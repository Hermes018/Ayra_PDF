//! Deterministic CycloneDX 1.5 SBOM generator and supply-chain re-vetting.
//! Strictly obeys `#![forbid(unsafe_code)]` and zero unwraps/panics in non-test code.

#![forbid(unsafe_code)]

use lpdf_core_types::LpdfError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// CycloneDX 1.5 JSON Bill of Materials document root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycloneDxBom {
    pub bom_format: String,
    pub spec_version: String,
    pub serial_number: String,
    pub version: u32,
    pub metadata: BomMetadata,
    pub components: Vec<BomComponent>,
}

/// Metadata section of the CycloneDX BOM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomMetadata {
    pub timestamp: String,
    pub tools: Vec<BomTool>,
    pub component: BomRootComponent,
}

/// Tool information that generated the BOM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomTool {
    pub vendor: String,
    pub name: String,
    pub version: String,
}

/// Root application component described by the BOM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomRootComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Individual software dependency component in the BOM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<Vec<BomHash>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<BomProperty>>,
}

/// Cryptographic hash entry for a component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomHash {
    pub alg: String,
    pub content: String,
}

/// Generic key-value property associated with a component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomProperty {
    pub name: String,
    pub value: String,
}

/// Parses workspace `Cargo.lock` content into a collection of `BomComponent` records.
pub fn parse_cargo_lock(content: &str) -> Vec<BomComponent> {
    let mut components = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut current_source: Option<String> = None;
    let mut current_checksum: Option<String> = None;

    let flush_package = |components: &mut Vec<BomComponent>,
                         name: &mut Option<String>,
                         version: &mut Option<String>,
                         source: &mut Option<String>,
                         checksum: &mut Option<String>| {
        if let (Some(n), Some(v)) = (name.take(), version.take()) {
            let src = source.take();
            let chk = checksum.take();

            let mut properties = Vec::new();
            if let Some(s) = src {
                properties.push(BomProperty {
                    name: "source".to_string(),
                    value: s,
                });
            }
            properties.push(BomProperty {
                name: "ecosystem".to_string(),
                value: "cargo".to_string(),
            });

            let hashes = chk.map(|c| {
                vec![BomHash {
                    alg: "SHA-256".to_string(),
                    content: c,
                }]
            });

            let purl = format!("pkg:cargo/{}@{}", n, v);

            components.push(BomComponent {
                component_type: "library".to_string(),
                name: n,
                version: v,
                purl: Some(purl),
                hashes,
                properties: if properties.is_empty() {
                    None
                } else {
                    Some(properties)
                },
            });
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            flush_package(
                &mut components,
                &mut current_name,
                &mut current_version,
                &mut current_source,
                &mut current_checksum,
            );
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("name = \"") {
            if let Some(end) = rest.strip_suffix('"') {
                current_name = Some(end.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("version = \"") {
            if let Some(end) = rest.strip_suffix('"') {
                current_version = Some(end.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("source = \"") {
            if let Some(end) = rest.strip_suffix('"') {
                current_source = Some(end.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("checksum = \"") {
            if let Some(end) = rest.strip_suffix('"') {
                current_checksum = Some(end.to_string());
            }
        }
    }

    flush_package(
        &mut components,
        &mut current_name,
        &mut current_version,
        &mut current_source,
        &mut current_checksum,
    );

    components
}

/// Minimal helper representation for parsing `package.json`.
#[derive(Debug, Deserialize)]
struct PackageJsonRaw {
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
}

/// Parses `package.json` into a collection of `BomComponent` records.
pub fn parse_package_json(content: &str) -> Result<Vec<BomComponent>, LpdfError> {
    let parsed: PackageJsonRaw =
        serde_json::from_str(content).map_err(|e| LpdfError::ConfigError(e.to_string()))?;

    let mut components = Vec::new();

    let mut add_deps = |deps: BTreeMap<String, String>, is_dev: bool| {
        for (name, version) in deps {
            let clean_ver = version.trim_start_matches(['^', '~', '=', 'v']).to_string();
            let purl = format!("pkg:npm/{}@{}", name, clean_ver);
            let mut props = vec![BomProperty {
                name: "ecosystem".to_string(),
                value: "npm".to_string(),
            }];
            if is_dev {
                props.push(BomProperty {
                    name: "scope".to_string(),
                    value: "devDependencies".to_string(),
                });
            } else {
                props.push(BomProperty {
                    name: "scope".to_string(),
                    value: "dependencies".to_string(),
                });
            }

            components.push(BomComponent {
                component_type: "library".to_string(),
                name,
                version: clean_ver,
                purl: Some(purl),
                hashes: None,
                properties: Some(props),
            });
        }
    };

    add_deps(parsed.dependencies, false);
    add_deps(parsed.dev_dependencies, true);

    Ok(components)
}

/// Generates a unified, deterministically sorted CycloneDX 1.5 JSON SBOM.
pub fn generate_cyclonedx_sbom(
    cargo_lock_path: &Path,
    package_json_path: &Path,
) -> Result<CycloneDxBom, LpdfError> {
    let cargo_lock_content = fs::read_to_string(cargo_lock_path).map_err(LpdfError::Io)?;
    let mut components = parse_cargo_lock(&cargo_lock_content);

    if package_json_path.exists() {
        let package_json_content = fs::read_to_string(package_json_path).map_err(LpdfError::Io)?;
        let npm_components = parse_package_json(&package_json_content)?;
        components.extend(npm_components);
    }

    // Sort components deterministically by name and version
    components.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));

    Ok(CycloneDxBom {
        bom_format: "CycloneDX".to_string(),
        spec_version: "1.5".to_string(),
        serial_number: "urn:uuid:68e82a93-5489-409c-9c02-e2d46e9dfef1".to_string(),
        version: 1,
        metadata: BomMetadata {
            timestamp: "2026-10-13T00:00:00Z".to_string(),
            tools: vec![BomTool {
                vendor: "LocalPDF Studio".to_string(),
                name: "lpdf-engine-switcher".to_string(),
                version: "0.1.0".to_string(),
            }],
            component: BomRootComponent {
                component_type: "application".to_string(),
                name: "localpdf-studio".to_string(),
                version: "0.1.0".to_string(),
                description: "Offline, local, privacy-first PDF utility suite".to_string(),
            },
        },
        components,
    })
}

/// Atomically writes the CycloneDX SBOM to the destination path.
pub fn write_sbom_to_file(bom: &CycloneDxBom, output_path: &Path) -> Result<(), LpdfError> {
    let json_bytes =
        serde_json::to_vec_pretty(bom).map_err(|e| LpdfError::ConfigError(e.to_string()))?;
    lpdf_fs_safety::atomic_write(output_path, &json_bytes)
}

/// Scans workspace crates to verify that all `lib.rs` and `main.rs` enforce `#![forbid(unsafe_code)]`.
pub fn verify_forbid_unsafe_in_workspace(
    crates_dir: &Path,
    app_tauri_dir: &Path,
) -> Result<Vec<PathBuf>, LpdfError> {
    let mut verified = Vec::new();
    let mut targets = Vec::new();

    // Collect crate entrypoints in crates/
    if crates_dir.exists() {
        let entries = fs::read_dir(crates_dir).map_err(LpdfError::Io)?;
        for entry in entries {
            let entry = entry.map_err(LpdfError::Io)?;
            let p = entry.path();
            if p.is_dir() {
                let lib_path = p.join("src").join("lib.rs");
                if lib_path.exists() {
                    targets.push(lib_path);
                }
                let main_path = p.join("src").join("main.rs");
                if main_path.exists() {
                    targets.push(main_path);
                }
            }
        }
    }

    // Collect app-tauri Rust entrypoint
    let tauri_src = app_tauri_dir.join("src-tauri").join("src");
    if tauri_src.exists() {
        let lib_path = tauri_src.join("lib.rs");
        if lib_path.exists() {
            targets.push(lib_path);
        }
        let main_path = tauri_src.join("main.rs");
        if main_path.exists() {
            targets.push(main_path);
        }
    }

    if targets.is_empty() {
        return Err(LpdfError::ConfigError(
            "No crate entrypoint source files discovered to verify".to_string(),
        ));
    }

    for target in targets {
        let content = fs::read_to_string(&target).map_err(LpdfError::Io)?;
        let has_forbid = content.lines().take(15).any(|line| {
            let trimmed = line.trim();
            trimmed == "#![forbid(unsafe_code)]"
        });

        if !has_forbid {
            return Err(LpdfError::ConfigError(format!(
                "Missing '#![forbid(unsafe_code)]' in entrypoint: {}",
                target.display()
            )));
        }

        verified.push(target);
    }

    Ok(verified)
}

/// Verifies `deny.toml` and `Cargo.lock` to ensure no copyleft licenses (GPL/AGPL) are linked.
pub fn verify_no_copyleft_dependencies(
    cargo_lock_path: &Path,
    deny_toml_path: &Path,
) -> Result<(), LpdfError> {
    let deny_content = fs::read_to_string(deny_toml_path).map_err(LpdfError::Io)?;

    // 1. Verify deny.toml does not permit GPL or AGPL licenses in allow list
    for line in deny_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('"') && (trimmed.contains("GPL") || trimmed.contains("AGPL")) {
            return Err(LpdfError::ConfigError(format!(
                "Forbidden copyleft license found in deny.toml allow list: {}",
                trimmed
            )));
        }
    }

    // 2. Scan Cargo.lock packages to ensure no known copyleft-only packages are linked
    let lock_content = fs::read_to_string(cargo_lock_path).map_err(LpdfError::Io)?;
    let banned_crates = ["pymupdf", "ghostscript", "mupdf", "itext"];
    for line in lock_content.lines() {
        let trimmed = line.trim();
        for banned in &banned_crates {
            if trimmed == format!("name = \"{}\"", banned) {
                return Err(LpdfError::ConfigError(format!(
                    "Banned copyleft crate '{}' linked in Cargo.lock",
                    banned
                )));
            }
        }
    }

    Ok(())
}

/// Verifies that `protocol/THREAT_MODEL.md` exists and documents threats T1 through T6.
pub fn verify_threat_model_coverage(threat_model_path: &Path) -> Result<(), LpdfError> {
    if !threat_model_path.exists() {
        return Err(LpdfError::ConfigError(format!(
            "Threat model document not found at: {}",
            threat_model_path.display()
        )));
    }

    let content = fs::read_to_string(threat_model_path).map_err(LpdfError::Io)?;
    let required_threats = ["T1", "T2", "T3", "T4", "T5", "T6"];

    for threat in &required_threats {
        if !content.contains(threat) {
            return Err(LpdfError::ConfigError(format!(
                "Threat model is missing required threat category '{}'",
                threat
            )));
        }
    }

    Ok(())
}

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable
)]
//! LocalPDF Studio geometry adapter (ADR-018 rev 3, R2 "strangler" step 2).
//!
//! App code (`app-tauri/src-tauri/src/commands/studio.rs`) calls only this crate for hatching.
//! Only our own types cross the boundary ([`PdfPt`], [`Loop`], [`Segment`], [`HatchPatternId`],
//! [`GeomError`]); no upstream engine type is ever exposed.
//!
//! Implementations are selected with [`HatchEngine`]:
//! - [`HatchEngine::Studio`] (default): our own pattern definitions (line families with angle,
//!   pitch, offset, dash) clipped to the real loops, even-odd. Fixes ERR-007 / BUG-015.
//! - [`HatchEngine::Legacy`]: the hand-written generator that used to live in `studio.rs`
//!   (`generate_hatch_strokes`, commit `562e4f9`), moved here unchanged. It hatches the
//!   **bounding box** of the loops, not the loops themselves (known defect ERR-007), and draws
//!   several patterns identically. Its output is pinned by `tests/hatch_golden.rs`.
//! - [`HatchEngine::Cadcraft`] (Cargo feature `cadcraft`, off by default until the R2 step 5
//!   switch): cadcraft@df65974 patterns (`cadcraft-doc`) swept across the real loops by the
//!   patched `cadcraft-render::pattern_lines`; fixes ERR-007. Parity: `tests/cadcraft_parity.rs`.
//!   The variant only exists when the feature is on, so no build can select an engine it lacks.

#[cfg(feature = "cadcraft")]
mod cadcraft;
mod fill;
mod legacy;
mod patterns;

#[cfg(feature = "cadcraft")]
pub use cadcraft::{
    display_scale as cadcraft_display_scale, AR_CONC_DISPLAY_SCALE, MAX_CADCRAFT_RUNS,
};
pub use fill::MAX_STUDIO_SEGMENTS;
pub use legacy::MAX_LEGACY_SEGMENTS;
pub use patterns::{
    pattern_families, pattern_tile, unit_pt, HatchFamily, SCALE_MAX, SCALE_MIN, TILE_CELLS,
    UNIT_BASE_PT, UNIT_MIN_PT,
};

/// A point in PDF user space (`pt`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfPt {
    pub x: f64,
    pub y: f64,
}

impl PdfPt {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// One boundary loop of a fill region, in PDF user space. The closing edge (last to first
/// vertex) is implicit.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Loop {
    pub points: Vec<PdfPt>,
}

impl Loop {
    pub fn new(points: Vec<PdfPt>) -> Self {
        Self { points }
    }

    /// The four corners of an axis-aligned rectangle `[x0, y0, x1, y1]`, counter-clockwise from
    /// `(x0, y0)`.
    pub fn from_rect(rect: [f64; 4]) -> Self {
        let [x0, y0, x1, y1] = rect;
        Self::new(vec![
            PdfPt::new(x0, y0),
            PdfPt::new(x1, y0),
            PdfPt::new(x1, y1),
            PdfPt::new(x0, y1),
        ])
    }
}

/// A straight hatch line segment from `a` to `b`, in PDF user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub a: PdfPt,
    pub b: PdfPt,
}

/// The line-hatch patterns the Studio offers (ids as used by the frontend `HatchPatternExtendedId`).
/// `solid` and `none` are fills, not line hatches, so they have no variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HatchPatternId {
    DiagonalSteel,
    Steel,
    Crosshatch,
    Tiling,
    Masonry,
    Timber,
    Earth,
    Insulation,
    Concrete,
}

impl HatchPatternId {
    /// Every line-hatch pattern, in a stable order.
    pub const ALL: [HatchPatternId; 9] = [
        HatchPatternId::DiagonalSteel,
        HatchPatternId::Steel,
        HatchPatternId::Crosshatch,
        HatchPatternId::Tiling,
        HatchPatternId::Masonry,
        HatchPatternId::Timber,
        HatchPatternId::Earth,
        HatchPatternId::Insulation,
        HatchPatternId::Concrete,
    ];

    /// Parse a frontend pattern id. `None` for `solid`, `none` and unknown ids (which the legacy
    /// generator also drew nothing for).
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "diagonal-steel" => Some(Self::DiagonalSteel),
            "steel" => Some(Self::Steel),
            "crosshatch" => Some(Self::Crosshatch),
            "tiling" => Some(Self::Tiling),
            "masonry" => Some(Self::Masonry),
            "timber" => Some(Self::Timber),
            "earth" => Some(Self::Earth),
            "insulation" => Some(Self::Insulation),
            "concrete" => Some(Self::Concrete),
            _ => None,
        }
    }

    /// The frontend id of this pattern.
    pub fn as_id(self) -> &'static str {
        match self {
            Self::DiagonalSteel => "diagonal-steel",
            Self::Steel => "steel",
            Self::Crosshatch => "crosshatch",
            Self::Tiling => "tiling",
            Self::Masonry => "masonry",
            Self::Timber => "timber",
            Self::Earth => "earth",
            Self::Insulation => "insulation",
            Self::Concrete => "concrete",
        }
    }
}

/// Which hatch implementation runs behind [`hatch_segments_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum HatchEngine {
    /// Our own clipped hatch (ERR-007 fix, owner decision 2026-11-06): pattern definitions in
    /// `patterns.rs`, even-odd clipping to the real loops in `fill.rs`. The default; no upstream
    /// code, so no licence question (R3).
    #[default]
    Studio,
    /// The pre-ADR-018 generator from `studio.rs` (bounding-box hatching, ERR-007). Kept only as
    /// the characterised oracle (goldens); removed one release after the Studio engine ships.
    Legacy,
    /// cadcraft@df65974 patterns swept across the real loops (even-odd, dashes): fixes ERR-007.
    /// Not the default until parity against the goldens passes (ADR-018 R2 step 5).
    #[cfg(feature = "cadcraft")]
    Cadcraft,
}

/// Geometry adapter errors, each with a stable code ([`GeomError::code`]).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum GeomError {
    /// No loop holds a point with finite coordinates, so there is no region to hatch.
    #[error("the hatch region has no finite boundary points")]
    NoFiniteGeometry,
    /// The pattern would need more than `limit` segments (huge extent, or coordinates so large
    /// that the line step no longer advances). Reported instead of hanging (DON'T-7: tell the user).
    #[error("the hatch needs more than {limit} segments; reduce the region or increase the pattern scale")]
    WorkLimitExceeded { limit: usize },
}

impl GeomError {
    /// Stable machine-readable code (protocol DO-7).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoFiniteGeometry => "GEOM_NO_FINITE_GEOMETRY",
            Self::WorkLimitExceeded { .. } => "GEOM_WORK_LIMIT",
        }
    }
}

/// Hatch line segments for `pattern` over the region bounded by `loops`, at pattern `scale`
/// (1.0 = default spacing), using the default [`HatchEngine`].
pub fn hatch_segments(
    pattern: HatchPatternId,
    loops: &[Loop],
    scale: f64,
) -> Result<Vec<Segment>, GeomError> {
    hatch_segments_with(HatchEngine::default(), pattern, loops, scale)
}

/// Square on-screen preview of `pattern` (`side` x `side` pt at the origin), from the cadcraft
/// engine's pattern data. Errors with [`GeomError::NoFiniteGeometry`] for a non-positive or
/// non-finite side (or one above 10,000 pt).
#[cfg(feature = "cadcraft")]
pub fn hatch_swatch(pattern: HatchPatternId, side: f64) -> Result<Vec<Segment>, GeomError> {
    cadcraft::hatch_swatch(pattern, side)
}

/// [`hatch_segments`] with an explicit implementation.
pub fn hatch_segments_with(
    engine: HatchEngine,
    pattern: HatchPatternId,
    loops: &[Loop],
    scale: f64,
) -> Result<Vec<Segment>, GeomError> {
    match engine {
        HatchEngine::Studio => fill::hatch_rings(
            patterns::pattern_families(pattern),
            &fill::finite_rings(loops),
            scale,
        ),
        HatchEngine::Legacy => legacy::hatch_segments(pattern, loops, scale),
        #[cfg(feature = "cadcraft")]
        HatchEngine::Cadcraft => cadcraft::hatch_segments(pattern, loops, scale),
    }
}

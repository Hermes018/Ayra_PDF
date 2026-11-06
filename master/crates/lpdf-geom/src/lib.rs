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
//! - [`HatchEngine::Legacy`]: the hand-written generator that used to live in `studio.rs`
//!   (`generate_hatch_strokes`, commit `562e4f9`), moved here unchanged. It hatches the
//!   **bounding box** of the loops, not the loops themselves (known defect ERR-007), and draws
//!   several patterns identically. Its output is pinned by `tests/hatch_golden.rs`.
//! - A cadcraft-backed implementation (`cadcraft-doc` patterns + `cadcraft-render`
//!   `pattern_lines`) is added behind this same switch in ADR-018 checkpoint 2. Until it exists
//!   the enum has only the legacy variant, so no caller can select an implementation that is not
//!   there.

#[cfg(feature = "cadcraft")]
mod cadcraft;
mod legacy;

#[cfg(feature = "cadcraft")]
pub use cadcraft::MAX_CADCRAFT_RUNS;
pub use legacy::MAX_LEGACY_SEGMENTS;

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
    /// The pre-ADR-018 generator from `studio.rs` (bounding-box hatching, ERR-007).
    #[default]
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

/// [`hatch_segments`] with an explicit implementation.
pub fn hatch_segments_with(
    engine: HatchEngine,
    pattern: HatchPatternId,
    loops: &[Loop],
    scale: f64,
) -> Result<Vec<Segment>, GeomError> {
    match engine {
        HatchEngine::Legacy => legacy::hatch_segments(pattern, loops, scale),
        #[cfg(feature = "cadcraft")]
        HatchEngine::Cadcraft => cadcraft::hatch_segments(pattern, loops, scale),
    }
}

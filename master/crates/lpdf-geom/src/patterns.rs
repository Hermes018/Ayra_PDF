//! Our own hatch pattern definitions (original work, no upstream; recorded in
//! `docs/UPSTREAM_MANIFEST.toml` as `lpdf-geom studio hatch patterns`).
//!
//! A pattern is a set of line families. One [`HatchFamily`] is a set of parallel lines:
//!
//! - `angle_deg`: direction of the lines, counter-clockwise from +x in PDF user space (y up).
//! - `pitch`: distance between neighbouring lines **measured along the axis that cuts them at the
//!   larger angle** (for 0 and 90 degrees the plain perpendicular spacing; for 45 and 135 degrees
//!   a pitch of 1 means the lines are one cell apart along x, i.e. `x - y = k` or `x + y = k`).
//! - `offset`: shift of the whole family across the lines, in cells.
//! - `phase`: shift of the dash pattern along the lines, in cells.
//! - `dash`: alternating on/off lengths along the lines, in cells, measured as the **projection on
//!   the main axis** (x for lines nearer to horizontal, y for lines nearer to vertical). Empty = solid.
//!
//! One *cell* is [`unit_pt`] points (8 pt at scale 1, the legacy line step). Every parameter is a
//! divisor of [`TILE_CELLS`], so each pattern repeats exactly every `TILE_CELLS` cells in x and y;
//! that tile is what the on-screen preview draws ([`pattern_tile`], exported to
//! `app-tauri/src/lib/core/hatchPatterns.json`), so screen and PDF come from the same numbers.

use crate::{GeomError, HatchPatternId, Segment};

/// A pattern repeats every this many cells in both directions.
pub const TILE_CELLS: f64 = 4.0;
/// Cell size at scale 1, in PDF points (the legacy hatch line step).
pub const UNIT_BASE_PT: f64 = 8.0;
/// Smallest cell size, in PDF points (legacy minimum step).
pub const UNIT_MIN_PT: f64 = 3.0;
/// User-facing scale range (legacy clamp).
pub const SCALE_MIN: f64 = 0.25;
pub const SCALE_MAX: f64 = 4.0;

/// One family of parallel (optionally dashed) hatch lines. See the module docs for the units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HatchFamily {
    pub angle_deg: f64,
    pub pitch: f64,
    pub offset: f64,
    pub phase: f64,
    pub dash: &'static [f64],
}

const fn fam(
    angle_deg: f64,
    pitch: f64,
    offset: f64,
    phase: f64,
    dash: &'static [f64],
) -> HatchFamily {
    HatchFamily {
        angle_deg,
        pitch,
        offset,
        phase,
        dash,
    }
}

const DIAGONAL_STEEL: [HatchFamily; 1] = [fam(45.0, 1.0, 0.0, 0.0, &[])];
// Steel section: solid line every 2 cells with a dashed line between.
const STEEL: [HatchFamily; 2] = [
    fam(45.0, 2.0, 0.0, 0.0, &[]),
    fam(45.0, 2.0, 1.0, 0.0, &[1.0, 1.0]),
];
// Diamond cross-hatch.
const CROSSHATCH: [HatchFamily; 2] = [
    fam(45.0, 1.0, 0.0, 0.0, &[]),
    fam(135.0, 1.0, 0.0, 0.0, &[]),
];
// Square tiles, 2 cells wide.
const TILING: [HatchFamily; 2] = [fam(0.0, 2.0, 0.0, 0.0, &[]), fam(90.0, 2.0, 0.0, 0.0, &[])];
// Running bond: a course line every cell, vertical joints every 2 cells, staggered by half a brick.
const MASONRY: [HatchFamily; 3] = [
    fam(0.0, 1.0, 0.0, 0.0, &[]),
    fam(90.0, 2.0, 0.0, 0.0, &[1.0, 1.0]),
    fam(90.0, 2.0, 1.0, 1.0, &[1.0, 1.0]),
];
// Wood grain: long staggered dashes on horizontal lines.
const TIMBER: [HatchFamily; 2] = [
    fam(0.0, 1.0, 0.0, 0.0, &[3.0, 1.0]),
    fam(0.0, 1.0, 0.5, 2.0, &[2.0, 2.0]),
];
// Earth: ground line every 2 cells with short oblique ticks.
const EARTH: [HatchFamily; 2] = [
    fam(0.0, 2.0, 0.0, 0.0, &[]),
    fam(45.0, 2.0, 1.0, 0.0, &[0.5, 3.5]),
];
// Insulation batt: zigzag made of two dashed diagonals.
const INSULATION: [HatchFamily; 2] = [
    fam(45.0, 2.0, 0.0, 0.0, &[1.0, 1.0]),
    fam(135.0, 2.0, 0.0, 1.0, &[1.0, 1.0]),
];
// Concrete: scattered short ticks (aggregate) on two diagonals.
const CONCRETE: [HatchFamily; 2] = [
    fam(45.0, 1.0, 0.0, 0.0, &[0.25, 1.75]),
    fam(135.0, 2.0, 1.0, 1.0, &[0.25, 3.75]),
];

/// The line families of `pattern`.
pub fn pattern_families(pattern: HatchPatternId) -> &'static [HatchFamily] {
    match pattern {
        HatchPatternId::DiagonalSteel => &DIAGONAL_STEEL,
        HatchPatternId::Steel => &STEEL,
        HatchPatternId::Crosshatch => &CROSSHATCH,
        HatchPatternId::Tiling => &TILING,
        HatchPatternId::Masonry => &MASONRY,
        HatchPatternId::Timber => &TIMBER,
        HatchPatternId::Earth => &EARTH,
        HatchPatternId::Insulation => &INSULATION,
        HatchPatternId::Concrete => &CONCRETE,
    }
}

/// The cell size in points for a user scale: `8 pt * clamp(scale, 0.25, 4)`, at least 3 pt (the
/// legacy step). A non-finite scale means 1.
pub fn unit_pt(scale: f64) -> f64 {
    let s = if scale.is_finite() { scale } else { 1.0 };
    (UNIT_BASE_PT * s.clamp(SCALE_MIN, SCALE_MAX)).max(UNIT_MIN_PT)
}

/// One repeat of `pattern` as segments in cell units: the pattern clipped to the square
/// `[0, TILE_CELLS]` x `[0, TILE_CELLS]`, by the same code that hatches real regions. Lines on the
/// lower and left tile edges are included and those on the upper and right edges are not, so
/// repeating the tile draws each line exactly once.
pub fn pattern_tile(pattern: HatchPatternId) -> Result<Vec<Segment>, GeomError> {
    let n = TILE_CELLS;
    let square = vec![(0.0, 0.0), (n, 0.0), (n, n), (0.0, n)];
    crate::fill::fill_rings(pattern_families(pattern), &[square], 1.0)
}

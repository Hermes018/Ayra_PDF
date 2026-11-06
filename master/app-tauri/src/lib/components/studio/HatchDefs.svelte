<script lang="ts">
  // Reusable, theme-contract-driven SVG <pattern> definitions for CAD/AEC procedural hatching.
  // Zero hardcoded hex literals: all strokes and fills read from currentColor / CSS variables.
  //
  // ERR-007 / BUG-015: the line hatches below are drawn from the SAME pattern parameters that
  // Studio save uses. `hatchPatterns.json` holds each pattern's families and one tile clipped by the
  // Rust hatch engine (crates/lpdf-geom); this component only scales that tile to CSS px. The tile is
  // authored in PDF orientation (y up), so it is mirrored here for the y-down canvas. The lattice is
  // anchored at the canvas origin, while a saved PDF anchors it at the page origin: spacing, angles
  // and dashes are identical, the phase can differ by less than one tile.
  import { hatchPatterns, hatchTileCells, hatchUnitPx, previewSegments } from '$lib/core/hatchPatterns';

  let { scale = 1.0 }: { scale?: number } = $props();

  let px = $derived(hatchUnitPx(scale));
  let tile = $derived(hatchTileCells * px);
</script>

<defs>
  {#each hatchPatterns as p (p.id)}
    <pattern
      id="hatch-{p.id}"
      width={tile}
      height={tile}
      patternUnits="userSpaceOnUse"
      patternTransform="scale(1 -1)"
    >
      {#each previewSegments(p) as seg, i (i)}
        <line
          x1={seg[0] * px}
          y1={seg[1] * px}
          x2={seg[2] * px}
          y2={seg[3] * px}
          stroke="var(--glass-text)"
          stroke-width="1"
          stroke-opacity="0.55"
        />
      {/each}
    </pattern>
  {/each}

  <!-- 9. Redaction Danger Crosshatch -->
  <pattern
    id="hatch-redact-danger"
    width="10"
    height="10"
    patternUnits="userSpaceOnUse"
    patternTransform="rotate(45)"
  >
    <line
      x1="0"
      y1="0"
      x2="0"
      y2="10"
      stroke="var(--danger)"
      stroke-width="2"
      stroke-opacity="0.65"
    />
  </pattern>
</defs>

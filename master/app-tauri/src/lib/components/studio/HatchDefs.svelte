<script lang="ts">
  // Reusable, theme-contract-driven SVG <pattern> definitions for CAD/AEC procedural hatching.
  // Zero hardcoded hex literals: all strokes and fills read from currentColor / CSS variables.
  let { scale = 1.0 }: { scale?: number } = $props();

  let unit = $derived(Math.max(4, Math.round(12 * scale)));
</script>

<defs>
  <!-- 1. Diagonal Steel (45° hatch lines) -->
  <pattern
    id="hatch-diagonal-steel"
    width={unit}
    height={unit}
    patternUnits="userSpaceOnUse"
    patternTransform="rotate(45)"
  >
    <line
      x1="0"
      y1="0"
      x2="0"
      y2={unit}
      stroke="var(--glass-text)"
      stroke-width="1.2"
      stroke-opacity="0.55"
    />
  </pattern>

  <!-- Alias for 'steel' -->
  <pattern
    id="hatch-steel"
    width={unit}
    height={unit}
    patternUnits="userSpaceOnUse"
    patternTransform="rotate(45)"
  >
    <line
      x1="0"
      y1="0"
      x2="0"
      y2={unit}
      stroke="var(--glass-text)"
      stroke-width="1.2"
      stroke-opacity="0.55"
    />
    <line
      x1={unit * 0.35}
      y1="0"
      x2={unit * 0.35}
      y2={unit}
      stroke="var(--glass-text)"
      stroke-width="0.8"
      stroke-opacity="0.4"
    />
  </pattern>

  <!-- 2. Crosshatch (orthogonal grid) -->
  <pattern
    id="hatch-crosshatch"
    width={unit}
    height={unit}
    patternUnits="userSpaceOnUse"
    patternTransform="rotate(45)"
  >
    <path
      d="M 0 0 L 0 {unit} M 0 0 L {unit} 0"
      stroke="var(--glass-text)"
      stroke-width="1"
      stroke-opacity="0.5"
      fill="none"
    />
  </pattern>

  <!-- 3. Concrete (aggregate triangles + stipple dots) -->
  <pattern
    id="hatch-concrete"
    width={unit * 2}
    height={unit * 2}
    patternUnits="userSpaceOnUse"
  >
    <polygon
      points="3,4 7,2 6,7"
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="0.9"
      stroke-opacity="0.55"
    />
    <polygon
      points="{unit + 2},{unit + 3} {unit + 7},{unit + 1} {unit + 5},{unit + 6}"
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="0.9"
      stroke-opacity="0.55"
    />
    <circle cx={unit * 0.8} cy={unit * 0.4} r="1" fill="var(--glass-text)" fill-opacity="0.5" />
    <circle cx={unit * 1.5} cy={unit * 0.8} r="1" fill="var(--glass-text)" fill-opacity="0.5" />
    <circle cx={unit * 0.4} cy={unit * 1.5} r="1" fill="var(--glass-text)" fill-opacity="0.5" />
  </pattern>

  <!-- 4. Timber (wood grain waves) -->
  <pattern
    id="hatch-timber"
    width={unit * 2}
    height={unit}
    patternUnits="userSpaceOnUse"
  >
    <path
      d="M 0 {unit * 0.3} Q {unit * 0.5} {unit * 0.1}, {unit} {unit * 0.3} T {unit * 2} {unit * 0.3}"
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="1"
      stroke-opacity="0.5"
    />
    <path
      d="M 0 {unit * 0.75} Q {unit * 0.5} {unit * 0.55}, {unit} {unit * 0.75} T {unit * 2} {unit * 0.75}"
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="0.8"
      stroke-opacity="0.4"
    />
  </pattern>

  <!-- 5. Tiling (orthogonal grid tiles) -->
  <pattern
    id="hatch-tiling"
    width={unit}
    height={unit}
    patternUnits="userSpaceOnUse"
  >
    <rect
      x="0"
      y="0"
      width={unit}
      height={unit}
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="0.9"
      stroke-opacity="0.45"
    />
  </pattern>

  <!-- 6. Masonry (running bond bricks) -->
  <pattern
    id="hatch-masonry"
    width={unit * 2}
    height={unit}
    patternUnits="userSpaceOnUse"
  >
    <path
      d="M 0 0 L {unit * 2} 0 M 0 {unit * 0.5} L {unit * 2} {unit * 0.5} M {unit * 0.5} 0 L {unit * 0.5} {unit * 0.5} M {unit * 1.5} {unit * 0.5} L {unit * 1.5} {unit}"
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="0.9"
      stroke-opacity="0.5"
    />
  </pattern>

  <!-- 7. Earth (compacted soil alternating hatch bundles) -->
  <pattern
    id="hatch-earth"
    width={unit * 2}
    height={unit * 2}
    patternUnits="userSpaceOnUse"
  >
    <path
      d="M 2 2 L {unit - 2} 2 M 2 5 L {unit - 2} 5 M {unit + 2} {unit + 2} L {unit + 2} {unit * 2 - 2} M {unit + 5} {unit + 2} L {unit + 5} {unit * 2 - 2}"
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="1"
      stroke-opacity="0.5"
    />
  </pattern>

  <!-- 8. Insulation (batt serpentine wave) -->
  <pattern
    id="hatch-insulation"
    width={unit * 2}
    height={unit}
    patternUnits="userSpaceOnUse"
  >
    <path
      d="M 0 {unit * 0.5} C {unit * 0.35} 0, {unit * 0.65} 0, {unit} {unit * 0.5} C {unit * 1.35} {unit}, {unit * 1.65} {unit}, {unit * 2} {unit * 0.5}"
      fill="none"
      stroke="var(--glass-text)"
      stroke-width="1"
      stroke-opacity="0.55"
    />
  </pattern>

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

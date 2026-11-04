<script lang="ts">
  export type StudioIconName =
    | 'pan'
    | 'select'
    | 'pen.a'
    | 'pen.b'
    | 'highlighter'
    | 'eraser'
    | 'eraser.partial'
    | 'eraser.whole'
    | 'laser'
    | 'text.review'
    | 'shape'
    | 'measure'
    | 'radial'
    | 'shortcuts'
    | 'undo'
    | 'redo'
    | 'grip'
    | 'close'
    | 'reset'
    | 'zoom.in'
    | 'zoom.out';

  let {
    name,
    size = 18
  }: {
    name: StudioIconName;
    size?: number;
  } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  class="studio-icon"
>
  {#if name === 'pan'}
    <path d="M18 11V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v4M14 10V4a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v6M10 10.5V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v8M6 14a4 4 0 0 0 8 0v-3a2 2 0 0 1 4 0v3a8 8 0 0 1-16 0v-2a2 2 0 0 1 4 0v2" />
  {:else if name === 'select'}
    <path d="M3 3l7 18 3-7 7-3L3 3z" />
  {:else if name === 'pen.a'}
    <path d="M12 19l7-7 3 3-7 7-3-3zM18 13l-1.5-1.5M4 20l4-1-3-3-1 4z" />
    <text x="18" y="8" font-size="7" font-weight="bold" fill="currentColor" stroke="none">A</text>
  {:else if name === 'pen.b'}
    <path d="M12 19l7-7 3 3-7 7-3-3zM18 13l-1.5-1.5M4 20l4-1-3-3-1 4z" />
    <text x="18" y="8" font-size="7" font-weight="bold" fill="currentColor" stroke="none">B</text>
  {:else if name === 'highlighter'}
    <path d="M9 11l-6 6v3h3l6-6M13.5 6.5l4 4M15 5l4 4-2 2-4-4 2-2zM4 20h4" />
  {:else if name === 'eraser' || name === 'eraser.partial'}
    <path d="M20 20H7L3 16c-1-1-1-2.5 0-3.5L12.5 3c1-1 2.5-1 3.5 0l5 5c1 1 1 2.5 0 3.5L14 18" />
    <path d="M7 16l6-6" />
  {:else if name === 'eraser.whole'}
    <path d="M20 20H7L3 16c-1-1-1-2.5 0-3.5L12.5 3c1-1 2.5-1 3.5 0l5 5c1 1 1 2.5 0 3.5L14 18" />
    <path d="M19 6l-3 3" />
  {:else if name === 'laser'}
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v4M12 18v4M2 12h4M18 12h4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
  {:else if name === 'text.review'}
    <path d="M4 7V4h16v3M12 4v16M4 18h16" />
  {:else if name === 'shape'}
    <rect x="3" y="3" width="11" height="11" rx="2" />
    <circle cx="15" cy="15" r="6" />
  {:else if name === 'measure'}
    <path d="M2 12h20M6 8v8M10 9v6M14 9v6M18 8v8" />
  {:else if name === 'radial'}
    <circle cx="12" cy="12" r="9" />
    <circle cx="12" cy="12" r="3" />
    <path d="M12 3v6M12 15v6M3 12h6M15 12h6M5.6 5.6l4.2 4.2M14.2 14.2l4.2 4.2M5.6 18.4l4.2-4.2M14.2 9.8l4.2-4.2" />
  {:else if name === 'shortcuts'}
    <rect x="2" y="4" width="20" height="16" rx="3" />
    <path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M6 12h.01M10 12h.01M14 12h.01M18 12h.01M8 16h8" />
  {:else if name === 'undo'}
    <path d="M3 7v6h6M3 13a9 9 0 0 1 15-4 9 9 0 0 1 0 12" />
  {:else if name === 'redo'}
    <path d="M21 7v6h-6M21 13a9 9 0 0 0-15-4 9 9 0 0 0 0 12" />
  {:else if name === 'grip'}
    <circle cx="9" cy="6" r="1.5" fill="currentColor" stroke="none" />
    <circle cx="15" cy="6" r="1.5" fill="currentColor" stroke="none" />
    <circle cx="9" cy="12" r="1.5" fill="currentColor" stroke="none" />
    <circle cx="15" cy="12" r="1.5" fill="currentColor" stroke="none" />
    <circle cx="9" cy="18" r="1.5" fill="currentColor" stroke="none" />
    <circle cx="15" cy="18" r="1.5" fill="currentColor" stroke="none" />
  {:else if name === 'close'}
    <path d="M18 6L6 18M6 6l12 12" />
  {:else if name === 'reset'}
    <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
    <path d="M3 3v5h5" />
  {:else if name === 'zoom.in'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'zoom.out'}
    <path d="M5 12h14" />
  {/if}
</svg>

<style>
  .studio-icon {
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
  }
</style>

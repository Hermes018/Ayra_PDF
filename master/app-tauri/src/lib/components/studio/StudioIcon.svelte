<script lang="ts">
  export type StudioIconName =
    | 'pan'
    | 'select'
    | 'lasso'
    | 'pen.a'
    | 'pen.b'
    | 'highlighter'
    | 'eraser'
    | 'eraser.partial'
    | 'eraser.whole'
    | 'laser'
    | 'text.review'
    | 'shape'
    | 'line'
    | 'arrow'
    | 'rect'
    | 'ellipse'
    | 'cloud'
    | 'hatch'
    | 'measure'
    | 'ruler'
    | 'tape'
    | 'protractor'
    | 'snap'
    | 'callout'
    | 'note'
    | 'stamp'
    | 'signature'
    | 'image'
    | 'latex'
    | 'audio'
    | 'radial'
    | 'shortcuts'
    | 'undo'
    | 'redo'
    | 'grip'
    | 'close'
    | 'reset'
    | 'zoom.in'
    | 'zoom.out'
    | 'pin'
    | 'star'
    | 'star.filled'
    | 'chevron.right'
    | 'chevron.left'
    | 'chevron.down'
    | 'check'
    | 'bolt'
    | 'chip'
    | 'globe'
    | 'file'
    | 'clock'
    | 'warning'
    | 'arrow.right';

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
  {:else if name === 'measure' || name === 'ruler'}
    <path d="M2 12h20M6 8v8M10 9v6M14 9v6M18 8v8" />
  {:else if name === 'snap'}
    <path d="M12 2v4M12 18v4M2 12h4M18 12h4" />
    <circle cx="12" cy="12" r="3" />
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
  {:else if name === 'pin'}
    <path d="M12 17v5M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
  {:else if name === 'lasso'}
    <path d="M7 22a5 5 0 0 1-2-4c0-3.87 3.85-7 8.6-7s8.6 3.13 8.6 7-3.85 7-8.6 7a9.3 9.3 0 0 1-4.6-1.2L3 25l1.6-4.4" />
  {:else if name === 'line'}
    <line x1="4" y1="20" x2="20" y2="4" />
  {:else if name === 'arrow'}
    <line x1="5" y1="19" x2="19" y2="5" />
    <polyline points="10 5 19 5 19 14" />
  {:else if name === 'rect'}
    <rect x="3" y="4" width="18" height="16" rx="2" />
  {:else if name === 'ellipse'}
    <ellipse cx="12" cy="12" rx="9" ry="7" />
  {:else if name === 'cloud'}
    <path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z" />
  {:else if name === 'hatch'}
    <rect x="3" y="3" width="18" height="18" rx="2" />
    <line x1="3" y1="9" x2="9" y2="3" />
    <line x1="3" y1="15" x2="15" y2="3" />
    <line x1="3" y1="21" x2="21" y2="3" />
    <line x1="9" y1="21" x2="21" y2="9" />
    <line x1="15" y1="21" x2="21" y2="15" />
  {:else if name === 'protractor'}
    <path d="M3 14a9 9 0 0 1 18 0H3z" />
    <line x1="12" y1="14" x2="12" y2="9" />
    <line x1="12" y1="14" x2="8.5" y2="10.5" />
    <line x1="12" y1="14" x2="15.5" y2="10.5" />
  {:else if name === 'tape'}
    <rect x="3" y="6" width="18" height="12" rx="2" />
    <line x1="7" y1="6" x2="7" y2="10" />
    <line x1="11" y1="6" x2="11" y2="10" />
    <line x1="15" y1="6" x2="15" y2="10" />
    <line x1="19" y1="6" x2="19" y2="10" />
  {:else if name === 'callout'}
    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
  {:else if name === 'note'}
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
    <polyline points="14 2 14 8 20 8" />
    <line x1="8" y1="13" x2="16" y2="13" />
    <line x1="8" y1="17" x2="12" y2="17" />
  {:else if name === 'stamp'}
    <path d="M5 22h14" />
    <path d="M19 18a2 2 0 0 0-2-2H7a2 2 0 0 0-2 2v4h14v-4z" />
    <path d="M12 16V6a3 3 0 1 0-6 0v10" />
  {:else if name === 'signature'}
    <path d="M3 21c3-2 5-6 6-9 1-3 0-5 2-5 3 0 2 6 4 6s3-4 6-4" />
    <path d="M18 19h4" />
  {:else if name === 'image'}
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
    <circle cx="8.5" cy="8.5" r="1.5" />
    <polyline points="21 15 16 10 5 21" />
  {:else if name === 'latex'}
    <path d="M4 19V5h4c2.5 0 4 1.5 4 4s-1.5 4-4 4H6" />
    <path d="M14 19l4-9 4 9" />
    <path d="M15.5 15.5h5" />
  {:else if name === 'audio'}
    <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
    <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
    <line x1="12" y1="19" x2="12" y2="22" />
  {:else if name === 'star'}
    <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
  {:else if name === 'star.filled'}
    <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" fill="currentColor" />
  {:else if name === 'chevron.right'}
    <polyline points="9 18 15 12 9 6" />
  {:else if name === 'chevron.left'}
    <polyline points="15 18 9 12 15 6" />
  {:else if name === 'chevron.down'}
    <polyline points="6 9 12 15 18 9" />
  {:else if name === 'check'}
    <polyline points="20 6 9 17 4 12" />
  {:else if name === 'bolt'}
    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
  {:else if name === 'chip'}
    <rect x="6" y="6" width="12" height="12" rx="2" />
    <path d="M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4" />
  {:else if name === 'globe'}
    <circle cx="12" cy="12" r="10" />
    <path d="M2 12h20M12 2a15 15 0 0 1 0 20M12 2a15 15 0 0 0 0 20" />
  {:else if name === 'file'}
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
    <polyline points="14 2 14 8 20 8" />
  {:else if name === 'clock'}
    <circle cx="12" cy="12" r="10" />
    <polyline points="12 6 12 12 16 14" />
  {:else if name === 'warning'}
    <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z" />
    <line x1="12" y1="9" x2="12" y2="13" />
    <line x1="12" y1="17" x2="12.01" y2="17" />
  {:else if name === 'arrow.right'}
    <line x1="5" y1="12" x2="19" y2="12" />
    <polyline points="12 5 19 12 12 19" />
  {/if}
</svg>

<style>
  .studio-icon {
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
  }
</style>

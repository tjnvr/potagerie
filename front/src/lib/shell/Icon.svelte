<script lang="ts" module>
  export type NameIcon =
    | "garden"
    | "edit"
    | "cal"
    | "check"
    | "settings"
    | "export"
    | "upload"
    | "trash"
    | "alert"
    | "close"
    | "plus"
    | "retry"
    | "play"
    | "pause"
    | "rotate"
    | "back";
</script>

<script lang="ts">
  // Icons of D01 and D02 (gen.py, icon()), 20 × 20 viewBox.
  let { name, size = 18 }: { name: NameIcon; size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 20 20"
  aria-hidden="true"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  {#if name === "play"}
    <path d="M7 4.5v11l9-5.5z" fill="currentColor" />
  {:else if name === "pause"}
    <path d="M6.5 4.5v11M13.5 4.5v11" stroke-width="3" />
  {:else if name === "rotate"}
    <path d="M15.5 8A6 6 0 1 0 16 12" stroke-width="2" />
    <path d="M16 3.5V8h-4.5" stroke-width="2" />
  {:else if name === "back"}
    <path d="M12 4.5L6.5 10l5.5 5.5" stroke-width="2" />
  {:else if name === "garden"}
    <path d="M3 12l7-4 7 4-7 4z" />
    <path d="M10 8V4.5M8 6l2-1.5L12 6" />
  {:else if name === "edit"}
    <path d="M4 16l1-4 8.5-8.5a2 2 0 0 1 3 3L8 15z" />
    <path d="M12 5l3 3" />
  {:else if name === "cal"}
    <rect x="3" y="4.5" width="14" height="12" rx="2.5" />
    <path d="M3 8.5h14M7 3v3M13 3v3" />
  {:else if name === "check"}
    <path d="M4.5 10.5l3.5 3.5 7.5-8" stroke-width="2" />
  {:else if name === "settings"}
    <circle cx="10" cy="10" r="2.6" />
    <path
      d="M10 3.5v2M10 14.5v2M3.5 10h2M14.5 10h2M5.4 5.4l1.4 1.4M13.2 13.2l1.4 1.4M5.4 14.6l1.4-1.4M13.2 6.8l1.4-1.4"
    />
  {:else if name === "export"}
    <path d="M10 3v9M6.5 6.5L10 3l3.5 3.5M4 12.5V16h12v-3.5" stroke-width="2" />
  {:else if name === "upload"}
    <path d="M10 16V7M6.5 10.5L10 7l3.5 3.5M4 4h12" stroke-width="2" />
  {:else if name === "trash"}
    <path d="M4.5 6h11M8 6V4h4v2M6 6l.8 10h6.4L14 6" />
  {:else if name === "alert"}
    <path d="M10 3.5l7 12.5H3z" />
    <path d="M10 8.5v3.5M10 14v.1" stroke-width="2" />
  {:else if name === "close"}
    <path d="M5 5l10 10M15 5L5 15" stroke-width="2" />
  {:else if name === "plus"}
    <path d="M10 4v12M4 10h12" stroke-width="2" />
  {:else if name === "retry"}
    <path d="M7 5L3.5 8.5 7 12M4 8.5h8a4 4 0 0 1 0 8H9" />
  {/if}
</svg>

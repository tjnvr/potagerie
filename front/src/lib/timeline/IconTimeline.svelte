<script lang="ts" module>
  export type NameIconTimeline = "book" | "before" | "after" | "plus" | "less" | "top" | "low" | "crayon" | "lock" | "check";
</script>

<script lang="ts">
  // Icons of the timeline, the planting form and the catalogue (D01, D02), in a
  // 20 × 20 box like `Icon.svelte`.
  let { name, size = 18 }: { name: NameIconTimeline; size?: number } = $props();
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
  {#if name === "book"}
    <path d="M4 4.5h4.5a1.5 1.5 0 0 1 1.5 1.5v10a1.5 1.5 0 0 0-1.5-1.5H4zM16 4.5h-4.5A1.5 1.5 0 0 0 10 6v10a1.5 1.5 0 0 1 1.5-1.5H16z" />
  {:else if name === "before"}
    <path d="M12 4.5L6.5 10l5.5 5.5" stroke-width="2" />
  {:else if name === "after"}
    <path d="M8 4.5l5.5 5.5L8 15.5" stroke-width="2" />
  {:else if name === "plus"}
    <path d="M10 4v12M4 10h12" stroke-width="2" />
  {:else if name === "less"}
    <path d="M4 10h12" stroke-width="2" />
  {:else if name === "top"}
    <path d="M4.5 12.5L10 7l5.5 5.5" stroke-width="2" />
  {:else if name === "low"}
    <path d="M4.5 7.5L10 13l5.5-5.5" stroke-width="2" />
  {:else if name === "crayon"}
    <path d="M4 16l1-4 8.5-8.5a2 2 0 0 1 3 3L8 15z" />
    <path d="M12 5l3 3" />
  {:else if name === "lock"}
    <rect x="4.5" y="9" width="11" height="7.5" rx="2" />
    <path d="M7 9V6.5a3 3 0 0 1 6 0V9" />
  {:else if name === "check"}
    <path d="M4.5 10.5l3.5 3.5 7.5-8" stroke-width="2" />
  {/if}
</svg>

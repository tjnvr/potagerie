<script lang="ts">
  import { localeTag } from "../locale";
  // The time cursor under the garden view (D01 scrubber, S01 F-VIS-3): play
  // button, a week back and forward, the slider over the year with its months.
  import Icon from "../shell/Icon.svelte";
  import { m } from "../paraglide/messages.js";
  import { daySince } from "./days";

  let {
    year,
    day,
    playback,
    speed,
    compact = false,
    onchoose,
    onplay,
    onspeed,
  }: {
    year: number;
    /** The day of the cursor, a day number (see days.ts). */
    day: number;
    playback: boolean;
    speed: number;
    compact?: boolean;
    onchoose: (day: number) => void;
    onplay: () => void;
    onspeed: (speed: number) => void;
  } = $props();

  const start = $derived(daySince(`${year}-01-01`));
  const end = $derived(daySince(`${year}-12-31`));
  const progression = $derived(Math.min(1, Math.max(0, (day - start) / (end - start))));

  /** The months of the year, with their length for the ticks. */
  const month = $derived(
    Array.from({ length: 12 }, (_, i) => {
      const premier = Date.UTC(year, i, 1);
      const next = Date.UTC(year, i + 1, 1);
      const name = new Date(premier).toLocaleDateString(localeTag(), { month: "short", timeZone: "UTC" });
      return { name, days: (next - premier) / 86_400_000 };
    }),
  );

  function week(direction: 1 | -1) {
    onchoose(Math.min(end, Math.max(start, Math.floor(day) + 7 * direction)));
  }
</script>

<div class="cursor" class:compact>
  <button
    class="play"
    type="button"
    data-play
    aria-pressed={playback}
    aria-label={playback ? m.diorama_pause() : m.diorama_play()}
    title={playback ? m.diorama_pause() : m.diorama_play()}
    onclick={onplay}
  >
    <Icon name={playback ? "pause" : "play"} size={22} />
  </button>
  <button class="step" type="button" aria-label={m.diorama_week_previous()} onclick={() => week(-1)}>
    <Icon name="back" size={16} />
  </button>
  <div class="lead">
    <input
      type="range"
      min={start}
      max={end}
      step="1"
      value={Math.floor(day)}
      style:--progression="{progression * 100}%"
      aria-label={m.diorama_cursor()}
      oninput={(e) => onchoose(Number(e.currentTarget.value))}
    />
    <div class="month" aria-hidden="true">
      {#each month as month_ (month_.name)}
        <span style:flex="{month_.days} 1 0">{compact ? month_.name.slice(0, 1) : month_.name}</span>
      {/each}
    </div>
  </div>
  <button class="step next" type="button" aria-label={m.diorama_week_next()} onclick={() => week(1)}>
    <Icon name="back" size={16} />
  </button>
  <div class="speed" role="group" aria-label={m.diorama_speed()}>
    {#each [1, 2] as v (v)}
      <button type="button" aria-pressed={speed === v} onclick={() => onspeed(v)}>×{v}</button>
    {/each}
  </div>
</div>

<style>
  .cursor {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .play {
    flex: none;
    display: grid;
    place-items: center;
    width: 56px;
    height: 56px;
    border: 0;
    border-radius: 50%;
    background: var(--accent);
    color: var(--accent-ink);
    box-shadow: 0 6px 16px rgba(242, 166, 90, 0.35);
  }

  .step {
    flex: none;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: 1px solid var(--line);
    border-radius: 50%;
    background: var(--white);
  }

  .next :global(svg) {
    transform: scaleX(-1);
  }

  .lead {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  input[type="range"] {
    width: 100%;
    height: 28px;
    margin: 0;
    background: transparent;
    appearance: none;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-runnable-track {
    height: 10px;
    border-radius: var(--pill);
    background: linear-gradient(to right, #f8d6b0 var(--progression), var(--white) var(--progression));
  }

  input[type="range"]::-moz-range-track {
    height: 10px;
    border-radius: var(--pill);
    background: linear-gradient(to right, #f8d6b0 var(--progression), var(--white) var(--progression));
  }

  input[type="range"]::-webkit-slider-thumb {
    width: 24px;
    height: 24px;
    margin-top: -7px;
    border: 4px solid var(--accent);
    border-radius: 50%;
    background: var(--white);
    appearance: none;
  }

  input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border: 4px solid var(--accent);
    border-radius: 50%;
    background: var(--white);
  }

  .month {
    display: flex;
    font-size: 11px;
    color: var(--muted);
  }

  .month span {
    min-width: 0;
    padding-left: 4px;
    border-left: 1px solid var(--line);
    overflow: hidden;
    white-space: nowrap;
  }

  .speed {
    flex: none;
    display: flex;
    padding: 3px;
    border-radius: var(--pill);
    background: var(--white);
  }

  .speed button {
    min-width: 40px;
    min-height: 36px;
    border: 0;
    border-radius: var(--pill);
    background: transparent;
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }

  .speed button[aria-pressed="true"] {
    background: var(--ink);
    color: var(--white);
  }

  .compact .play {
    width: 48px;
    height: 48px;
  }

  @media (max-width: 720px) {
    .cursor {
      flex-wrap: wrap;
      gap: 8px 10px;
    }
    .lead {
      order: 5;
      flex: 1 1 100%;
    }
    .speed {
      margin-left: auto;
    }
  }
</style>

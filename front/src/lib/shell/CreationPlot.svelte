<script lang="ts">
  import { localeTag } from "../locale";
  // First launch without a plot (ET01 ①), then the E2 form (F-TER-1).
  import { commands } from "../bindings";
  import { app } from "../state.svelte";
  import { m } from "../paraglide/messages.js";
  import Icon from "./Icon.svelte";
  import Logo from "./Logo.svelte";

  const CASES = [25, 50, 100];
  const AISLES = [30, 50, 80];

  // `?form` opens the form directly in dev, for screenshots.
  let form = $state(import.meta.env.DEV && new URLSearchParams(location.search).has("form"));
  let length = $state(9);
  let width = $state(6.5);
  let cellCm = $state(50);
  let aisleCm = $state(50);

  const lon = $derived(Math.round(Number(length) * 100));
  const lar = $derived(Math.round(Number(width) * 100));
  const valid = $derived(lon >= 100 && lar >= 100 && lon <= 10000 && lar <= 10000);
  const columns = $derived(valid ? Math.floor(lon / cellCm) : 0);
  const rows = $derived(valid ? Math.floor(lar / cellCm) : 0);
  const meters = (value: number) => value.toLocaleString(localeTag(), { maximumFractionDigits: 2 });

  async function create(event: SubmitEvent) {
    event.preventDefault();
    await app.modify((year) =>
      commands.createPlot(year, {
        length_cm: Math.round(length * 100),
        width_cm: Math.round(width * 100),
        cell_cm: cellCm,
        aisle_cm: aisleCm,
      }),
    );
  }
</script>

<section class="empty">
  {#if !form}
    <Logo size={40} />
    <p>{m.plot_absent()}</p>
    <button class="button principal" onclick={() => (form = true)}>
      <Icon name="plus" />{m.plot_create()}
    </button>
  {:else}
    <div class="two">
      <form class="form" onsubmit={create}>
        <div>
          <h2>{m.plot_title()}</h2>
          <p class="intro">{m.plot_intro()}</p>
        </div>
        <div class="dimensions">
          <label>
            <span>{m.plot_length()}</span>
            <span class="input">
              <input type="number" inputmode="decimal" min="1" max="100" step="0.5" required bind:value={length} />
              <abbr title={m.plot_unit()}>{m.plot_unit()}</abbr>
            </span>
          </label>
          <label>
            <span>{m.plot_width()}</span>
            <span class="input">
              <input type="number" inputmode="decimal" min="1" max="100" step="0.5" required bind:value={width} />
              <abbr title={m.plot_unit()}>{m.plot_unit()}</abbr>
            </span>
          </label>
        </div>
        {#if !valid}<p class="alert">{m.plot_dimensions_invalid()}</p>{/if}
        <fieldset>
          <legend>{m.plot_case()}</legend>
          <div class="choice">
            {#each CASES as value (value)}
              <label class:active={cellCm === value}>
                <input type="radio" name="case" value={value} bind:group={cellCm} />
                {m.cm({ value })}
              </label>
            {/each}
          </div>
          <small>{m.plot_case_lock()}</small>
        </fieldset>
        <fieldset>
          <legend>{m.plot_aisle()}</legend>
          <div class="choice">
            {#each AISLES as value (value)}
              <label class:active={aisleCm === value}>
                <input type="radio" name="aisle" value={value} bind:group={aisleCm} />
                {m.cm({ value })}
              </label>
            {/each}
          </div>
          <small>{m.plot_aisle_help()}</small>
        </fieldset>
        <div class="actions">
          <button type="button" class="button" onclick={() => (form = false)}>{m.cancel()}</button>
          <button type="submit" class="button principal" disabled={!valid}>
            <Icon name="check" />{m.plot_validate()}
          </button>
        </div>
      </form>

      <div class="preview">
        <div class="preview-title">
          <strong>{m.plot_preview()}</strong>
          {#if valid}
            <span>
              {m.plot_preview_summary({
                length: meters(lon / 100),
                width: meters(lar / 100),
                case: cellCm,
                columns,
                rows,
              })}
            </span>
          {/if}
        </div>
        {#if valid}
          <svg viewBox="0 0 {lon} {lar}" role="img" aria-label={m.plot_preview_aria()}>
            <rect width={lon} height={lar} rx={cellCm / 3} fill="var(--grass)" />
            {#each Array.from({ length: Math.max(columns - 1, 0) }, (_, i) => (i + 1) * cellCm) as x (x)}
              <line x1={x} x2={x} y1="0" y2={lar} class="grille" />
            {/each}
            {#each Array.from({ length: Math.max(rows - 1, 0) }, (_, i) => (i + 1) * cellCm) as y (y)}
              <line y1={y} y2={y} x1="0" x2={lon} class="grille" />
            {/each}
          </svg>
        {/if}
        <p class="note">{m.plot_preview_empty()}</p>
      </div>
    </div>
  {/if}
</section>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    min-height: 360px;
    padding: 24px;
    border-radius: 32px;
    background: var(--card);
    text-align: center;
    color: var(--muted);
  }

  .two {
    display: grid;
    grid-template-columns: minmax(0, 480px) minmax(0, 1fr);
    gap: 24px;
    width: 100%;
    align-items: start;
  }

  .intro {
    margin: 6px 0 0;
    font-size: 14px;
    color: var(--muted);
  }

  .alert {
    margin: 0;
    padding: 10px 14px;
    border-radius: 14px;
    background: var(--crit-bg);
    color: #5a1e1a;
    font-size: 13px;
  }

  .input {
    display: flex;
    align-items: center;
    height: 48px;
    padding: 0 14px;
    border-radius: 14px;
    background: var(--bg);
    margin: 0;
  }

  .input input {
    flex: 1;
    min-width: 0;
    height: 100%;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--ink);
    font-weight: 700;
    font-size: 16px;
  }

  .input abbr {
    text-decoration: none;
    font-weight: 700;
    color: var(--muted);
  }

  .preview {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px;
    border-radius: 28px;
    background: var(--card);
    text-align: left;
    color: var(--ink);
  }

  .preview-title {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 6px 16px;
    font-size: 13px;
  }

  .preview-title span,
  .note {
    color: var(--muted);
  }

  .preview svg {
    display: block;
    width: 100%;
    max-height: 340px;
  }

  .grille {
    stroke: #ffffff;
    stroke-opacity: 0.4;
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }

  .note {
    margin: 0;
    font-size: 13px;
  }

  @media (max-width: 900px) {
    .two {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  .form {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 24px;
    border-radius: 28px;
    background: var(--white);
    color: var(--ink);
    text-align: left;
  }

  .dimensions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  label > span,
  legend {
    display: block;
    font-size: 12px;
    font-weight: 700;
    color: var(--muted);
    margin-bottom: 6px;
  }

  fieldset {
    border: 0;
    margin: 0;
    padding: 0;
  }

  small {
    display: block;
    margin-top: 6px;
    color: var(--muted);
  }

  .choice {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: var(--pill);
    background: var(--bg);
  }

  .choice label {
    flex: 1;
    text-align: center;
    padding: 10px 0;
    border-radius: var(--pill);
    font-weight: 600;
    font-size: 14px;
    cursor: pointer;
  }

  .choice label.active {
    background: var(--ink);
    color: var(--white);
  }

  .choice input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .choice label:has(input:focus-visible) {
    outline: 3px solid var(--accent);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
  }
</style>

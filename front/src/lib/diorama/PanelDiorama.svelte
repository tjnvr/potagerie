<script lang="ts">
  // The bed panel (E06) opened from the garden view. Same panel as in the
  // editor; here it only edits through its fields (no drag), so the move and
  // resize go straight to the commands, with the same warning before a
  // shorter bed drops plantings (S03 §4).
  import { commands, type BedView, type YearView } from "../bindings";
  import PanelBed from "../editor/PanelBed.svelte";
  import type { Geo, Response } from "../editor/drag";
  import Confirmation from "../shell/Confirmation.svelte";
  import { app } from "../state.svelte";
  import { m } from "../paraglide/messages.js";

  let { view, bed, onclose }: { view: YearView; bed: BedView; onclose: () => void } = $props();

  let shortening = $state<{ count: number; answer: (ok: boolean) => void } | null>(null);

  async function modify(id: string, geo: Geo, nature: "move" | "resize"): Promise<Response> {
    if (nature === "resize" && geo.length_cm < bed.length_cm) {
      const result = await commands.plantingsExceeding(app.year, id, geo.length_cm);
      if (result.status === "ok" && result.data.length > 0) {
        const ok = await new Promise<boolean>((answer) => {
          shortening = { count: result.data.length, answer };
        });
        shortening = null;
        if (!ok) return "cancelled";
      }
    }
    const ok =
      nature === "move"
        ? await app.modify((year) => commands.moveBed(year, id, geo.x, geo.y))
        : await app.modify((year) =>
            commands.modifyBed(year, id, null, { bed_type: bed.bed_type, ...geo }),
          );
    return ok ? "ok" : "refusal";
  }
</script>

<PanelBed {view} {bed} {onclose} onmodify={modify} />

{#if shortening}
  {@const request = shortening}
  <Confirmation
    title={m.shorten_title({ name: bed.name })}
    confirm={m.shorten_button()}
    tone="alert"
    onconfirm={() => request.answer(true)}
    oncancel={() => request.answer(false)}
  >
    <p>{m.shorten_text({ count: request.count })}</p>
  </Confirmation>
{/if}

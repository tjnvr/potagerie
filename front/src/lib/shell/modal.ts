// Keyboard behaviour shared by the windows (Settings, Catalogue, Export, Note, Planting):
// focus goes into the window when it opens, Tab and Shift+Tab stay inside it, and focus
// goes back to the control that opened it when it closes.

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

function focusables(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
    (el) => !el.closest("[inert]") && el.getClientRects().length > 0,
  );
}

export function modal(root: HTMLElement) {
  const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  root.focus();

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Tab" || e.defaultPrevented) return;
    // A native <dialog> (confirmation) opened above this window traps focus by itself.
    if (root.querySelector("dialog[open]")) return;
    const items = focusables(root);
    if (items.length === 0) {
      e.preventDefault();
      root.focus();
      return;
    }
    const first = items[0]!;
    const last = items[items.length - 1]!;
    const active = document.activeElement;
    if (!(active instanceof Node) || !root.contains(active) || active === root) {
      e.preventDefault();
      (e.shiftKey ? last : first).focus();
    } else if (e.shiftKey && active === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && active === last) {
      e.preventDefault();
      first.focus();
    }
  }

  document.addEventListener("keydown", onkeydown);
  return {
    destroy() {
      document.removeEventListener("keydown", onkeydown);
      if (opener?.isConnected) opener.focus();
    },
  };
}

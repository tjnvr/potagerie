// Civil dates `YYYY-MM-DD` (S03 §1) as day numbers, for the layout of the
// timeline only: pixels to days and back. No rule of S03 lives here (no
// duration, no conflict, no slot): every date that matters comes from Rust.

const MS_PAR_JOUR = 86_400_000;

/** Days since 1970-01-01 of a civil date. Never goes through a time zone. */
export function jourDe(iso: string): number {
  const [a = 1970, m = 1, j = 1] = iso.split("-").map(Number);
  return Math.round(Date.UTC(a, m - 1, j) / MS_PAR_JOUR);
}

/** The civil date of a day number. */
export function dateDe(jour: number): string {
  return new Date(jour * MS_PAR_JOUR).toISOString().slice(0, 10);
}

export function anneeDe(iso: string): number {
  return Number(iso.slice(0, 4));
}

/** January 1st of `annee`, as a day number. */
export function premierJanvier(annee: number): number {
  return jourDe(`${String(annee).padStart(4, "0")}-01-01`);
}

/** The local date of the device (S03 §1: « aujourd'hui » is the local date). */
export function aujourdhuiLocal(maintenant: Date = new Date()): string {
  const m = String(maintenant.getMonth() + 1).padStart(2, "0");
  const j = String(maintenant.getDate()).padStart(2, "0");
  return `${maintenant.getFullYear()}-${m}-${j}`;
}

/** The civil date `n` days after `iso`: stepping the cursor by a week. */
export function decalerDate(iso: string, jours: number): string {
  return dateDe(jourDe(iso) + jours);
}

/** A civil date as « 2 août », with the year when it is not `annee`. */
export function dateAbregee(iso: string, annee?: number): string {
  const jour = new Date(`${iso}T12:00:00`);
  const options: Intl.DateTimeFormatOptions =
    jour.getFullYear() === annee ? { day: "numeric", month: "short" } : { day: "numeric", month: "short", year: "numeric" };
  return jour.toLocaleDateString("fr-FR", options);
}

/** « lundi 2 août 2027 ». */
export function dateComplete(iso: string): string {
  return new Date(`${iso}T12:00:00`).toLocaleDateString("fr-FR", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

/** The same month and day in another year, for moving the cursor from one
 *  season to another. February 29th becomes the 28th in a common year. */
export function memeJourAnnee(iso: string, annee: number): string {
  const [, m = 1, j = 1] = iso.split("-").map(Number);
  const jour = new Date(Date.UTC(annee, m - 1, j));
  const dernier = jour.getUTCMonth() !== m - 1 ? Date.UTC(annee, m, 0) : jour.getTime();
  return new Date(dernier).toISOString().slice(0, 10);
}

// Day numbers: days since 1970-01-01, UTC, so a civil date is an integer and
// the animation can move through the days with a fraction.

const MS_PAR_JOUR = 86_400_000;

/** `YYYY-MM-DD` as a day number. */
export function jourDepuis(iso: string): number {
  const [a = 1970, m = 1, j = 1] = iso.split("-").map(Number);
  return Math.round(Date.UTC(a, m - 1, j) / MS_PAR_JOUR);
}

/** A day number (a fraction is dropped) as `YYYY-MM-DD`. */
export function isoDepuis(jour: number): string {
  return new Date(Math.floor(jour) * MS_PAR_JOUR).toISOString().slice(0, 10);
}

/** First and last day numbers of a calendar year. */
export function bornesAnnee(annee: number): { debut: number; fin: number } {
  return { debut: jourDepuis(`${annee}-01-01`), fin: jourDepuis(`${annee}-12-31`) };
}

/** The month, 1 to 12, of a day number. */
export function moisDe(jour: number): number {
  return new Date(Math.floor(jour) * MS_PAR_JOUR).getUTCMonth() + 1;
}

/** A day number as « 20 juin 2027 ». */
export function dateEnToutesLettres(jour: number): string {
  return new Date(Math.floor(jour) * MS_PAR_JOUR).toLocaleDateString("fr-FR", {
    day: "numeric",
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  });
}

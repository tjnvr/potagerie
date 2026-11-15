// Frame-rate metrics of P01: mean images per second, the 1 % slowest frames,
// the 95th percentile frame time. Used by the dev counter (`?fps`) and by the
// headless measurement (front/scripts/mesurer.mjs).

export type Mesure = {
  images: number;
  /** Mean images per second over the run. */
  moyenne: number;
  /** Images per second of the slowest 1 % of the frames. */
  bas1pct: number;
  /** Frame time under which 95 % of the frames fall, in ms. */
  p95ms: number;
};

/** Frames dropped at the start of a run: the first ones pay for the set-up. */
export const IMAGES_IGNOREES = 5;
export const CRITERE_MOYENNE = 55;
export const CRITERE_BAS_1PCT = 30;

/** Metrics of a run from its frame durations in ms; null if too short. */
export function mesurer(durees: readonly number[]): Mesure | null {
  const ft = durees.slice(IMAGES_IGNOREES).sort((a, b) => a - b);
  if (ft.length < 20) return null;
  const moy = ft.reduce((a, b) => a + b, 0) / ft.length;
  const p99 = ft[Math.floor(ft.length * 0.99)] ?? 1;
  const p95 = ft[Math.floor(ft.length * 0.95)] ?? 0;
  return {
    images: ft.length,
    moyenne: 1000 / moy,
    bas1pct: 1000 / p99,
    p95ms: p95,
  };
}

/** The P01 criterion: mean at least 55 images/s and 1 % slowest at least 30. */
export function respecteLeCritere(m: Mesure): boolean {
  return m.moyenne >= CRITERE_MOYENNE && m.bas1pct >= CRITERE_BAS_1PCT;
}

/** Mean images per second over the last frames of a running playback. */
export function imagesParSeconde(recentes: readonly number[]): number {
  if (recentes.length === 0) return 0;
  return 1000 / (recentes.reduce((a, b) => a + b, 0) / recentes.length);
}

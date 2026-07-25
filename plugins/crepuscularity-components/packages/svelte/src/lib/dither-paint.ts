import { rgb, type Seed } from "./theme";

export type AreaVariant = "gradient" | "dotted" | "hatched" | "solid";

export const BAYER = [
  [0, 8, 2, 10],
  [12, 4, 14, 6],
  [3, 11, 1, 9],
  [15, 7, 13, 5],
].map((row) => row.map((v) => (v + 0.5) / 16));

export const CELL = 2;
export const MAX_COLS = 520;
export const MAX_ROWS = 200;
export const BORDER_ALPHA = 0.72;
export const OFF_TIER = 0.4;

export const clamp01 = (t: number) => (t < 0 ? 0 : t > 1 ? 1 : t);

export type PaintOpts = {
  variant: AreaVariant;
  intensity?: number;
  dim?: number;
  stacked?: boolean;
  sparse?: number;
};

export function paintColumn(
  octx: CanvasRenderingContext2D,
  x: number,
  top: number,
  floor: number,
  seed: Seed,
  {
    variant,
    intensity = 0,
    dim = 1,
    stacked = false,
    sparse = 0,
  }: PaintOpts,
) {
  const t = Math.round(top);
  const f = Math.round(floor);
  const depth = f - t;
  if (depth <= 0) {
    octx.fillStyle = rgb(seed.fill, 1, BORDER_ALPHA * dim);
    octx.fillRect(x, t, 1, 1);
    return;
  }
  const bias = (variant === "dotted" ? 0.12 : 0) + (stacked ? 0.2 : 0) - sparse;
  for (let y = t; y < f; y++) {
    let density = (y - t) / depth;
    if (stacked) density = 0.5 + 0.5 * density;
    if (variant === "hatched" && ((x + y) & 3) >= 2) continue;
    const lit =
      variant === "solid" ||
      density > BAYER[y & 3]![x & 3]! - 0.1 * intensity - bias;
    if (variant === "dotted" && !lit) continue;
    const k = (0.3 + density * 0.7) * (1 + 0.22 * intensity);
    const alpha = clamp01((lit ? k : k * OFF_TIER) * dim);
    octx.fillStyle = rgb(seed.fill, 1, alpha);
    octx.fillRect(x, y, 1, 1);
  }
  octx.fillStyle = rgb(seed.fill, 1, BORDER_ALPHA * dim);
  octx.fillRect(x, t, 1, 1);
  if (depth > 1) {
    octx.fillStyle = rgb(seed.fill, 1, BORDER_ALPHA * 0.5 * dim);
    octx.fillRect(x, t + 1, 1, 1);
  }
}

export function resample(src: number[], cols: number): number[] {
  const out = new Array<number>(cols);
  const last = Math.max(src.length - 1, 1);
  for (let c = 0; c < cols; c++) {
    const t = (c / Math.max(cols - 1, 1)) * last;
    const i = Math.floor(t);
    const f = t - i;
    const a = src[i] ?? 0;
    const b = src[Math.min(i + 1, src.length - 1)] ?? a;
    out[c] = a + (b - a) * f;
  }
  return out;
}

export function backingSize(width: number, height: number) {
  return {
    cols: Math.min(MAX_COLS, Math.max(8, Math.round(width / CELL))),
    rows: Math.min(MAX_ROWS, Math.max(8, Math.round(height / CELL))),
  };
}

export function sparklineColumnTops(
  values: number[],
  cols: number,
  rows: number,
): number[] {
  if (!values.length) return [];
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = Math.max(max - min, 1e-9);
  const resampled = resample(values, cols);
  return resampled.map((v) =>
    Math.round(rows - 1 - ((v - min) / span) * (rows - 1)),
  );
}

/** Paint a sparkline/area fill; scales low-res dither cells to CSS size. */
export function paintSparkline(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  values: number[],
  seed: Seed,
  variant: AreaVariant = "gradient",
  intensity = 0,
) {
  if (!values.length || width <= 0 || height <= 0) return;
  const { cols, rows } = backingSize(width, height);
  const tops = sparklineColumnTops(values, cols, rows);
  const floor = rows - 1;
  ctx.save();
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, width, height);
  ctx.scale(width / cols, height / rows);
  for (let x = 0; x < cols; x++) {
    paintColumn(ctx, x, tops[x]!, floor, seed, { variant, intensity });
  }
  ctx.restore();
}

export { default as Sparkline } from "./Sparkline.svelte";
export { default as DitherButton } from "./DitherButton.svelte";
export { default as DitherGradient } from "./DitherGradient.svelte";
export { default as Badge } from "./Badge.svelte";
export {
  PALETTE,
  themes,
  rgb,
  seedOfColor,
  isDitherColor,
  type DitherColor,
  type Seed,
  type Theme,
  type Rgb,
} from "./theme";
export {
  BAYER,
  paintColumn,
  paintSparkline,
  resample,
  backingSize,
  sparklineColumnTops,
  type AreaVariant,
} from "./dither-paint";

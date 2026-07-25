export type Rgb = [number, number, number];

export type DitherColor =
  | "green"
  | "blue"
  | "purple"
  | "pink"
  | "orange"
  | "red"
  | "grey";

export type Seed = { fill: Rgb; line: Rgb; star: Rgb };

export type Theme = {
  name: string;
  background: string;
  foreground: string;
  muted: string;
  accent: string;
  seeds: Record<string, Seed>;
};

export const PALETTE: Record<DitherColor, Seed> = {
  green: { fill: [40, 210, 110], line: [150, 255, 180], star: [200, 255, 220] },
  blue: { fill: [53, 143, 243], line: [150, 200, 255], star: [205, 228, 255] },
  purple: {
    fill: [150, 110, 255],
    line: [200, 175, 255],
    star: [225, 210, 255],
  },
  pink: { fill: [240, 90, 190], line: [255, 170, 220], star: [255, 205, 235] },
  orange: {
    fill: [255, 150, 50],
    line: [255, 195, 130],
    star: [255, 220, 175],
  },
  red: { fill: [240, 70, 70], line: [255, 150, 140], star: [255, 195, 185] },
  grey: { fill: [92, 92, 100], line: [140, 140, 150], star: [165, 165, 175] },
};

export const themes: Record<string, Theme> = {
  "dither-kit": {
    name: "dither-kit",
    background: "#0a0a0c",
    foreground: "#f2f2f5",
    muted: "#5c5c64",
    accent: "#358ff3",
    seeds: { ...PALETTE },
  },
  kumo: {
    name: "kumo",
    background: "#ffffff",
    foreground: "#1a1a1a",
    muted: "#6b7280",
    accent: "#4290f0",
    seeds: {
      blue: { fill: [66, 144, 240], line: [142, 188, 246], star: [200, 222, 251] },
      yellow: { fill: [245, 182, 71], line: [250, 210, 130], star: [255, 230, 180] },
      pink: { fill: [232, 100, 157], line: [245, 160, 195], star: [255, 205, 225] },
      purple: { fill: [141, 88, 238], line: [180, 145, 245], star: [210, 190, 255] },
      teal: { fill: [80, 195, 182], line: [140, 220, 210], star: [190, 235, 228] },
      orange: { fill: [211, 117, 54], line: [235, 165, 110], star: [250, 200, 160] },
      grey: { fill: [203, 203, 203], line: [160, 160, 160], star: [180, 180, 180] },
    },
  },
  night: {
    name: "night",
    background: "#0c0e14",
    foreground: "#e8eaf0",
    muted: "#6a7080",
    accent: "#7aa2ff",
    seeds: { ...PALETTE },
  },
  chalk: {
    name: "chalk",
    background: "#f7f5f0",
    foreground: "#1c1b19",
    muted: "#7a756c",
    accent: "#2f6fed",
    seeds: {
      green: { fill: [40, 210, 110], line: [30, 160, 85], star: [20, 120, 65] },
      blue: { fill: [53, 143, 243], line: [40, 110, 200], star: [30, 80, 160] },
      purple: { fill: [150, 110, 255], line: [110, 80, 200], star: [80, 55, 160] },
      pink: { fill: [240, 90, 190], line: [190, 60, 150], star: [150, 40, 120] },
      orange: { fill: [255, 150, 50], line: [210, 110, 30], star: [170, 80, 20] },
      red: { fill: [240, 70, 70], line: [190, 45, 45], star: [150, 30, 30] },
      grey: { fill: [120, 118, 112], line: [90, 88, 82], star: [70, 68, 62] },
    },
  },
  aurora: {
    name: "aurora",
    background: "#061018",
    foreground: "#e6fff8",
    muted: "#4a7a72",
    accent: "#28e0b0",
    seeds: {
      green: { fill: [40, 220, 140], line: [120, 255, 200], star: [180, 255, 230] },
      blue: { fill: [40, 160, 255], line: [120, 200, 255], star: [180, 225, 255] },
      purple: { fill: [160, 100, 255], line: [200, 160, 255], star: [230, 200, 255] },
      pink: { fill: [255, 80, 180], line: [255, 150, 210], star: [255, 200, 230] },
      orange: { fill: [255, 140, 60], line: [255, 190, 120], star: [255, 220, 170] },
      red: { fill: [255, 70, 90], line: [255, 140, 150], star: [255, 190, 195] },
      grey: { fill: [80, 100, 110], line: [120, 140, 150], star: [160, 175, 185] },
    },
  },
};

export const rgb = ([r, g, b]: Rgb, k = 1, a = 1) =>
  `rgba(${Math.round(r * k)},${Math.round(g * k)},${Math.round(b * k)},${a})`;

export const seedOfColor = (color: string, themeName = "dither-kit"): Seed => {
  const theme = themes[themeName] ?? themes["dither-kit"]!;
  return theme.seeds[color] ?? theme.seeds.blue ?? PALETTE.blue;
};

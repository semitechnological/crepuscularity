import { describe, expect, test } from "bun:test";
import {
  backingSize,
  resample,
  sparklineColumnTops,
  BAYER,
} from "./dither-paint";
import { PALETTE, seedOfColor, themes } from "./theme";

describe("theme", () => {
  test("dither-kit blue fill matches dither-kit", () => {
    expect(PALETTE.blue.fill).toEqual([53, 143, 243]);
  });

  test("kumo blue matches Cloudflare categorical", () => {
    expect(themes.kumo!.seeds.blue!.fill).toEqual([66, 144, 240]);
  });

  test("seedOfColor falls back to blue", () => {
    expect(seedOfColor("nope").fill).toEqual(PALETTE.blue.fill);
  });
});

describe("dither-paint", () => {
  test("Bayer matrix is 4x4 normalized", () => {
    expect(BAYER).toHaveLength(4);
    expect(BAYER[0]).toHaveLength(4);
    expect(BAYER[0]![0]).toBeCloseTo(0.5 / 16);
  });

  test("resample expands series", () => {
    const out = resample([0, 10], 5);
    expect(out).toHaveLength(5);
    expect(out[0]).toBe(0);
    expect(out[4]).toBe(10);
  });

  test("backingSize clamps", () => {
    const s = backingSize(200, 56);
    expect(s.cols).toBeGreaterThanOrEqual(8);
    expect(s.rows).toBeGreaterThanOrEqual(8);
  });

  test("sparklineColumnTops orders high values near top", () => {
    const tops = sparklineColumnTops([0, 10], 4, 20);
    expect(tops[0]!).toBeGreaterThan(tops[3]!);
  });
});

import { describe, expect, test } from "bun:test";
import {
  BAYER,
  resample,
  sparklineColumnTops,
  backingSize,
} from "./dither-paint";
import { PALETTE, seedOfColor, themes } from "./themes";

describe("themes", () => {
  test("dither-kit palette seeds", () => {
    expect(PALETTE.green.fill).toEqual([40, 210, 110]);
    expect(PALETTE.blue.fill).toEqual([53, 143, 243]);
  });

  test("kumo categorical blue", () => {
    expect(themes.kumo!.seeds.blue!.fill).toEqual([66, 144, 240]);
  });

  test("seedOfColor", () => {
    expect(seedOfColor("purple").fill).toEqual(PALETTE.purple.fill);
  });
});

describe("dither-paint", () => {
  test("Bayer thresholds", () => {
    expect(BAYER[0]![0]).toBeCloseTo(0.5 / 16);
    expect(BAYER[1]![0]).toBeCloseTo(12.5 / 16);
  });

  test("resample + tops", () => {
    expect(resample([1, 2, 3], 3)).toEqual([1, 2, 3]);
    const tops = sparklineColumnTops([1, 5, 1], 6, 24);
    expect(tops).toHaveLength(6);
    expect(Math.min(...tops)).toBeLessThan(Math.max(...tops));
  });

  test("backingSize", () => {
    const s = backingSize(100, 40);
    expect(s.cols).toBe(50);
    expect(s.rows).toBe(20);
  });
});

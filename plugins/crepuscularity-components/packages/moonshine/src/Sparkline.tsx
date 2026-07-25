import { useEffect, useRef } from "react";
import {
  paintSparkline,
  type AreaVariant,
} from "./dither-paint";
import { seedOfColor, type DitherColor } from "./themes";

export type SparklineProps = {
  values: number[];
  color?: DitherColor | string;
  variant?: AreaVariant;
  height?: number;
  theme?: string;
  intensity?: number;
  className?: string;
};

/**
 * Canvas Bayer-dither sparkline (dither-kit paint port).
 */
export function Sparkline({
  values,
  color = "blue",
  variant = "gradient",
  height = 56,
  theme = "dither-kit",
  intensity = 0,
  className,
}: SparklineProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (!canvas || !wrap || values.length < 2) return;

    const draw = () => {
      const width = wrap.clientWidth;
      if (width <= 0) return;
      const dpr = window.devicePixelRatio || 1;
      canvas.width = Math.max(1, Math.floor(width * dpr));
      canvas.height = Math.max(1, Math.floor(height * dpr));
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      paintSparkline(
        ctx,
        width,
        height,
        values,
        seedOfColor(color, theme),
        variant,
        intensity,
      );
    };

    draw();
    const ro = new ResizeObserver(draw);
    ro.observe(wrap);
    return () => ro.disconnect();
  }, [values, color, variant, height, theme, intensity]);

  return (
    <div
      ref={wrapRef}
      className={className}
      style={{ width: "100%", height, display: "block" }}
    >
      <canvas
        ref={canvasRef}
        aria-hidden
        style={{
          display: "block",
          width: "100%",
          height: "100%",
          imageRendering: "pixelated",
        }}
      />
    </div>
  );
}

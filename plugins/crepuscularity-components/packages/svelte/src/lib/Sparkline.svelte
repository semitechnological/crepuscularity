<script lang="ts">
  import { onMount } from "svelte";
  import { paintSparkline, type AreaVariant } from "./dither-paint";
  import { seedOfColor, type DitherColor } from "./theme";

  interface Props {
    values: number[];
    color?: DitherColor | string;
    variant?: AreaVariant;
    height?: number;
    theme?: string;
    intensity?: number;
    class?: string;
  }

  let {
    values,
    color = "blue",
    variant = "gradient",
    height = 56,
    theme = "dither-kit",
    intensity = 0,
    class: className = "",
  }: Props = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let width = $state(0);

  function redraw() {
    if (!canvas || width <= 0 || values.length < 2) return;
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
  }

  onMount(() => {
    const ro = new ResizeObserver((entries) => {
      width = entries[0]?.contentRect.width ?? 0;
      redraw();
    });
    if (canvas?.parentElement) ro.observe(canvas.parentElement);
    return () => ro.disconnect();
  });

  $effect(() => {
    values;
    color;
    variant;
    height;
    theme;
    intensity;
    width;
    redraw();
  });
</script>

<div class="crepus-sparkline {className}" style:height="{height}px">
  <canvas bind:this={canvas} aria-hidden="true"></canvas>
</div>

<style>
  .crepus-sparkline {
    width: 100%;
    display: block;
    position: relative;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
    image-rendering: pixelated;
  }
</style>

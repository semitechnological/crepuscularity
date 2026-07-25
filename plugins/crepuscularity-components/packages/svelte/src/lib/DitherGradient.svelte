<script lang="ts">
  import { onMount } from "svelte";
  import { backingSize, paintColumn, type AreaVariant } from "./dither-paint";
  import { seedOfColor, type DitherColor } from "./theme";

  interface Props {
    color?: DitherColor | string;
    variant?: AreaVariant;
    height?: number;
    theme?: string;
    class?: string;
  }

  let {
    color = "blue",
    variant = "gradient",
    height = 80,
    theme = "dither-kit",
    class: className = "",
  }: Props = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let width = $state(0);

  function redraw() {
    if (!canvas || width <= 0) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.floor(width * dpr));
    canvas.height = Math.max(1, Math.floor(height * dpr));
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const { cols, rows } = backingSize(width, height);
    const seed = seedOfColor(color, theme);
    ctx.save();
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, width, height);
    ctx.scale(width / cols, height / rows);
    for (let x = 0; x < cols; x++) {
      paintColumn(ctx, x, 0, rows - 1, seed, { variant });
    }
    ctx.restore();
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
    color;
    variant;
    height;
    theme;
    width;
    redraw();
  });
</script>

<div class="crepus-gradient {className}" style:height="{height}px">
  <canvas bind:this={canvas} aria-hidden="true"></canvas>
</div>

<style>
  .crepus-gradient {
    width: 100%;
    display: block;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
    image-rendering: pixelated;
  }
</style>

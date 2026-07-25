<script lang="ts">
  import { rgb, seedOfColor, type DitherColor } from "./theme";

  interface Props {
    label: string;
    color?: DitherColor | string;
    tone?: "solid" | "soft";
    theme?: string;
    class?: string;
  }

  let {
    label,
    color = "blue",
    tone = "soft",
    theme = "dither-kit",
    class: className = "",
  }: Props = $props();

  const seed = $derived(seedOfColor(color, theme));
  const fill = $derived(rgb(seed.fill));
  const soft = $derived(rgb(seed.fill, 1, 0.18));
</script>

<span
  class="crepus-badge crepus-badge--{tone} {className}"
  style:--crepus-fill={fill}
  style:--crepus-soft={soft}
>
  {label}
</span>

<style>
  .crepus-badge {
    display: inline-flex;
    align-items: center;
    border-radius: 4px;
    padding: 0.125rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 600;
  }
  .crepus-badge--solid {
    background: var(--crepus-fill);
    color: #fff;
  }
  .crepus-badge--soft {
    background: var(--crepus-soft);
    color: var(--crepus-fill);
  }
</style>

<script lang="ts">
  import { rgb, seedOfColor, type DitherColor } from "./theme";

  interface Props {
    label: string;
    color?: DitherColor | string;
    variant?: "solid" | "outline" | "ghost";
    disabled?: boolean;
    theme?: string;
    onclick?: (e: MouseEvent) => void;
    class?: string;
  }

  let {
    label,
    color = "blue",
    variant = "solid",
    disabled = false,
    theme = "dither-kit",
    onclick,
    class: className = "",
  }: Props = $props();

  const seed = $derived(seedOfColor(color, theme));
  const fill = $derived(rgb(seed.fill));
</script>

<button
  type="button"
  class="crepus-btn crepus-btn--{variant} {className}"
  style:--crepus-fill={fill}
  {disabled}
  {onclick}
>
  {label}
</button>

<style>
  .crepus-btn {
    font: inherit;
    cursor: pointer;
    border-radius: 6px;
    padding: 0.5rem 1rem;
    border: 1px solid transparent;
  }
  .crepus-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .crepus-btn--solid {
    background: var(--crepus-fill);
    color: #fff;
  }
  .crepus-btn--outline {
    background: transparent;
    color: var(--crepus-fill);
    border-color: var(--crepus-fill);
  }
  .crepus-btn--ghost {
    background: transparent;
    color: var(--crepus-fill);
  }
</style>

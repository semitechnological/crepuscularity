# UI Library

Reusable, generic Crepuscularity components for data-rich interfaces.

## Components

### `ui-library.crepus`

Multi-component file containing:

- **Grid** — responsive grid with `columns`, `gap`, and `aspectRatio` props.
- **Panel** — collapsible section with `title`, `collapsible`, and `defaultExpanded` props.
- **Wizard** — multi-step flow with `steps`, `currentStep`, `showProgress`, and `allowSkip` props.
- **List** — vertical list of items with `label`/`detail` fields and `selectable` prop.
- **Card** — bordered card with `title`, `subtitle`, and slot content.
- **Badge** — status badge with `label` and `color` props.
- **Button** — clickable button with `label` and `variant` props.
- **Alert** — banner alert with `message` and `kind` props.
- **Tabs** — horizontal tab list with `tabs` and `active` props.
- **EmptyState** — centered placeholder with `message` and optional `action`.
- **Avatar** — rounded image or text avatar with `src`, `label`, and `size` props.

### `transfer-progress.crepus`

Single-file component showing transfer progress with optional speed and ETA.

## Usage

```text
include ui-library/ui-library.crepus#Grid columns=4 gap=4
  for photo in {photos}
    img src={photo.thumb} w-full h-full object-cover
```

## Examples

See `examples/` for demo screens built from the library components.

## Compatibility

Components use Tailwind-style utility classes and are intended for `crepus web`, `crepus gpui`, and `crepus render` targets. Native IR and TUI targets may need fallbacks for grid/progress elements.

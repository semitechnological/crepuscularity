# Changelog

## Unreleased

### Behaviour changes

- `if` conditions now resolve a bare operand the same way as an operand on
  either side of a comparison. `if true`, `if false`, `if 1`, `if 0` and
  `if "text"` are literals; every other bare token is still a scope lookup.
  Previously a bare token was *always* a lookup, so `if true` was falsy while
  `x == true` worked — an inconsistency model-authored documents fell into.
- In `.crepus` source, `checked=false` (and `=0`, `=no`, `=off`, `=""`) now
  yields an unchecked control. Previously any `checked=` attribute counted as
  set regardless of value, so the source front-end disagreed with the IR
  decoder, which only honours JSON `true`.

### Fixed

- `listitem` / `li` inline `"..."` and `label=` text now render instead of
  producing empty bullets; empty list items are skipped entirely.
- `text bind=` resolves against the host `data` scope before interpolation.
- `listitem onclick=` dispatches through the same action seam as buttons.
- `progress` and `meter` always show a percentage beside the bar.
- A `progress` or `meter` inside a horizontal `stack`/`scroll` no longer fails
  layout: the bar falls back to a fixed width when its parent supplies
  unbounded width.
- A `list` inside a horizontal `stack`/`scroll` no longer fails layout: the
  marker row degrades to an intrinsically-sized row when width is unbounded.
- A negative `padding*` is clamped to zero instead of tripping the
  non-negative-insets assertion.
- Absurd but finite layout magnitudes (padding, spacing, spacer size, corner
  radius, border width, font size) are clamped to a sane maximum so a value
  like `1e9` cannot produce an unusable render.

## 0.1.0

- Initial release of the Flutter runtime renderer.
- `CrepusView.fromIr` renders decoded View IR JSON.
- `CrepusView.fromSource` renders `.crepus` source via a pure-Dart,
  indentation-based parser.
- Constrained node vocabulary: text, stack, scroll, button, toggle, checkbox,
  progress, meter, badge, divider, spacer, image, if, forEach, list, listItem.
  Unknown/disallowed kinds render nothing.
- Generic, action-agnostic dispatch via `onAction`; node/depth/source caps.
- Golden conformance against `examples/native-shells/fixture.json`.

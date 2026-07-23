# Changelog

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

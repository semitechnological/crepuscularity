# TODO

## Mobile native support

Goal: let `.crepus` stay the shared syntax while mobile becomes another backend family, not a fork of the language.

### Short-term

- Add a backend-neutral render IR in `crepuscularity-core`
- Keep HTML, React, and GPUI as separate backends consuming that IR
- Make backend selection explicit in the CLI

### Native mobile plan

1. Add `crepuscularity-mobile-core`
   - shared mobile view primitives
   - layout, text, input, image, scroll, stack, overlay, navigation nodes
   - event and state bridge independent of iOS/Android details

2. Add `crepuscularity-ios`
   - map `.crepus` + render IR to SwiftUI first
   - fall back to UIKit wrappers only where SwiftUI is insufficient
   - use Rust for parsing, evaluation, diffing, and data/state coordination

3. Add `crepuscularity-android`
   - map the same IR to Jetpack Compose first
   - use platform views only where Compose is insufficient

4. Keep `crepuscularity-react` and `crepuscularity-web` as the mobile-web path
   - immediate browser-based mobile support via WASM/webview
   - useful before native backends are mature

### Should we build our own renderer with Impeller?

Not first.

Reasons:

- Impeller is tightly associated with Flutter's engine stack and is not the lowest-friction Rust-native path
- building a full custom renderer before the render IR stabilizes would slow everything down
- native widget backends to SwiftUI and Compose get us mobile faster with better platform integration

### If we do build our own renderer later

Prefer this order:

1. backend-neutral render IR in core
2. platform widget backends first
3. experimental custom renderer only after the syntax and IR settle

If a custom renderer becomes necessary, a Rust-native path built on `wgpu` plus a scene/text stack such as `vello` is more realistic than betting directly on Impeller.

### Open questions

- how much of the DSL should map 1:1 across web, GPUI, and mobile?
- do we want a shared styling model or per-backend styling adapters?
- do we want native navigation/state lifecycles in the core IR or in backend adapters?

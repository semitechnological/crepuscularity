# React Native adaptation

Crepuscularity should not parse compiled React Native JavaScript and claim native parity. The bundle no longer contains the component, state, navigation, accessibility, or TurboModule contracts needed for a faithful conversion.

The viable boundary is source-level adaptation: consume TypeScript/TSX plus an explicit configuration file, preserve unsupported source untouched, and produce a Crepus project only for components and APIs with a tested mapping.

## Existing shared surface

| React Native | Crepuscularity |
|---|---|
| `View` / flexbox | `div flex` / `flex-row` / `flex-col` |
| `Text` | quoted text or `span` |
| `Pressable` / `Button` | `button @click` |
| `TextInput` | `input bind=` |
| `ScrollView` / `FlatList` | scroll container / `for` |
| props and static styles | template variables and utility classes |
| native module request | `NativePluginRequest` |

The generated target remains SwiftUI and Jetpack Compose through the existing View IR. It is not a React runtime.

## Adapter contract

`crepus react-native audit <project>` should scan source and produce one record per detected component, Expo module, React Native API, navigation dependency, and custom native module.

Each record is one of:

- `native`: a direct, tested Crepus mapping exists.
- `backend`: the UI can be converted but needs an explicit Rust/native handler.
- `unsupported`: conversion must stop before changing the project.

`crepus react-native convert` should only run after the audit contains no `unsupported` records. It writes a new native Crepus project rather than editing the React Native tree in place.

## Initial mapping boundary

The first conversion milestone should cover static and form-centric screens:

- layout, text, images, buttons, text inputs, toggles, sliders, scroll containers, and lists;
- static style objects whose values map to the existing native style registry;
- accessibility label, role, and hint when the equivalent View IR field exists;
- `Linking`, clipboard, share, haptics, document picking, and image picking through `NativePluginRequest`.

It should reject React hooks with side effects, arbitrary JavaScript expressions, Reanimated, Gesture Handler, custom Fabric components, direct native-module calls, platform-specific conditionals, navigation, and third-party UI packages until a real adapter exists.

## State and navigation

React component state should not be translated mechanically into generated Swift/Kotlin state. The target contract is a typed Rust action backend returning View IR state updates. A screen converts only when its state can be expressed as template bindings plus actions; otherwise the audit reports the hook or reducer as `backend`.

Navigation needs a first-class View IR route model before it can be converted. Until then, React Navigation and Expo Router are unsupported rather than flattened into one screen.

## Native modules

React Native TurboModules are source-defined TypeScript specifications plus generated Swift/Kotlin/C++ bindings. Crepuscularity should use the same explicitness: each supported module needs a named `NativeCapability`, a JSON request/response contract, native host implementations, permission handling, and platform smoke tests.

Expo modules belong in the same category. They are not generic JavaScript dependencies and must be mapped one at a time.

## Verification

Every mapped component needs:

1. a source fixture and expected audit result;
2. View IR and generated SwiftUI/Compose assertions;
3. iOS/macOS Swift build and Android debug-APK smoke proof;
4. an accessibility and interaction test when the mapping is interactive.

The authoritative external references are [React Native core components](https://reactnative.dev/docs/components-and-apis), [TurboModules](https://reactnative.dev/docs/turbo-native-modules-introduction), [React Native architecture](https://reactnative.dev/architecture/landing-page), and [Expo Modules](https://docs.expo.dev/modules/overview/).

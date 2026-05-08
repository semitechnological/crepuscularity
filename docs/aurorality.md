# Aurorality (SwiftUI Engine)

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [Native shells](native.md)

Aurorality is the SwiftUI-facing engine that consumes `.crepus` templates via `swiftgen` and renders them as native SwiftUI views in macOS/iOS host apps.

## What Aurorality Adds

- Native SwiftUI output from `.crepus` templates
- Semantic SwiftUI-style tags (`navigationstack`, `sidebar`, `item`, `menu`, `label`, `sf-*`, etc.)
- Utility-first layout fallback (`div flex`, `div flex-col`, spacing, sizing classes)
- Runtime bridge hooks (`eventSink`) for app state and transport actions
- Host-app integration with Rust/UniFFI backends

## Authoring Model

Aurorality supports a hybrid model:

- **Semantic components** for explicit native intent
- **Utility classes** for fast layout and spacing

```text
navigationsplitview
  sidebar
    section header="Conversations"
      button @click="newConversation"
        sf-plus
      for conv in {conversations}
        item @click="selectConversation:{conv.id}"
          span "{conv.title}"
  detail
    vstack gap-2 p-3
      textfield bind=draft placeholder="Message"
      button button-prominent @click="send"
        "Send"
```

## Supported Semantic Tags (swiftgen path)

- Navigation: `navigationstack`, `navigationsplitview`, `sidebar`, `detail`, `content`
- Layout: `vstack`, `hstack`, `zstack`, `group`, `form`, `section`, `list`, `item`, `scrollview`, `divider`, `spacer`
- Controls: `button`, `toggle`, `picker`, `textfield`, `input`, `securefield`, `menu`
- Content: `label`, `image`, `sf`, `sf-*`, `span`, `p`

See [DSL reference](dsl.md) for the living mapping details.

## Integration Notes

- `swiftgen` emits a generated SwiftUI view struct (for example `HyperChatGeneratedView`).
- Host apps pass a typed context object plus `eventSink`.
- Rust plugins/backends can be wired through UniFFI and invoked from host-side state models.

## Related Projects

- Aurorality repo: [semitechnological/aurorality](https://github.com/semitechnological/aurorality)
- Native shell examples in this repo: [native.md](native.md)

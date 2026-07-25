# @crepuscularity/moonshine

Thin bridge from crepuscularity JSON IR → [moonshine](../moonshine) React elements.

The crepuscularity CLI emit step imports this package so compiled `.crepus` modules depend on one entrypoint.

## Usage

```ts
import { renderCrepusIr, createMoonshineApp } from "@crepuscularity/moonshine";

const ir = {
  version: 1,
  root: [
    {
      kind: "stack",
      axis: "column",
      children: [
        { kind: "text", content: "Hello" },
        { kind: "button", label: "Go", onClick: "go" },
        { kind: "sparkline", values: [1, 3, 2, 5, 4] },
      ],
    },
  ],
};

function App() {
  return renderCrepusIr(ir, {
    onAction: (handler) => console.log(handler),
  });
}

createMoonshineApp({ root: App }).mount("#app");
```

Supported node kinds (MVP): `text`, `stack`, `button`, `sparkline`.

## Scripts

```bash
bun test
```

## License

ISC

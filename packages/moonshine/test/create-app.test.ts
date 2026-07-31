import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { describe, expect, test, beforeAll, afterAll, beforeEach } from "bun:test";
import { createMoonshineApp } from "../src/create-app";
import { createElement } from "react";

beforeAll(() => {
  GlobalRegistrator.register();
});

afterAll(() => {
  GlobalRegistrator.unregister();
});

describe("createMoonshineApp", () => {
  const DummyApp = () => createElement("div", null, "Hello Moonshine");

  beforeEach(() => {
    document.body.innerHTML = '<div id="app"></div>';
  });

  test("mounts app into container string and unmounts", () => {
    const app = createMoonshineApp({ root: DummyApp });
    expect(app.mounted).toBe(false);

    app.mount("#app");
    expect(app.mounted).toBe(true);

    // Test unmount
    app.unmount();
    expect(app.mounted).toBe(false);
  });

  test("mounts app into Element directly", () => {
    const app = createMoonshineApp({ root: DummyApp });
    const el = document.getElementById("app")!;

    app.mount(el);
    expect(app.mounted).toBe(true);
  });

  test("applies optional className during mount and removes it on unmount", () => {
    const app = createMoonshineApp({
      root: DummyApp,
      className: "moonshine-app dark-theme"
    });

    const el = document.getElementById("app")!;
    app.mount("#app");

    expect(el.classList.contains("moonshine-app")).toBe(true);
    expect(el.classList.contains("dark-theme")).toBe(true);

    app.unmount();

    expect(el.classList.contains("moonshine-app")).toBe(false);
    expect(el.classList.contains("dark-theme")).toBe(false);
  });

  test("throws if already mounted", () => {
    const app = createMoonshineApp({ root: DummyApp });
    app.mount("#app");

    expect(() => app.mount("#app")).toThrow("createMoonshineApp: already mounted");
  });

  test("throws if container not found", () => {
    const app = createMoonshineApp({ root: DummyApp });

    expect(() => app.mount("#missing")).toThrow("createMoonshineApp: mount target not found: #missing");
  });
});

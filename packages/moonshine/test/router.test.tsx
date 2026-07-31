import { describe, expect, test, afterEach } from "bun:test";
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import { matchPath, matchRoutes, useLocation, navigate } from "../src/router";

describe("router core", () => {
  test("matchPath extracts params", () => {
    const m = matchPath("/users/:id", "/users/42");
    expect(m).not.toBeNull();
    expect(m!.params.id).toBe("42");
  });

  test("matchPath rejects length mismatch", () => {
    expect(matchPath("/a/b", "/a")).toBeNull();
  });

  test("matchRoutes picks first hit", () => {
    const hit = matchRoutes(
      [
        { path: "/", element: "home" },
        { path: "/about", element: "about" },
        { path: "/users/:id", element: "user" },
      ],
      "/users/7",
    );
    expect(hit?.element).toBe("user");
    expect(hit?.params.id).toBe("7");
  });
});

describe("useLocation", () => {
  afterEach(() => {
    navigate("/");
  });

  test("returns initial location and updates when navigation happens", async () => {
    let currentLocation = "";
    let renderCount = 0;

    function TestComponent() {
      currentLocation = useLocation();
      renderCount++;
      return null;
    }

    const div = document.createElement("div");
    const root = createRoot(div);
    root.render(createElement(TestComponent));

    // Wait for initial render
    await new Promise(r => setTimeout(r, 10));
    expect(currentLocation).toBe("/");
    expect(renderCount).toBe(1);

    // Navigate and check if it updates
    navigate("/about");

    await new Promise(r => setTimeout(r, 10));
    expect(currentLocation).toBe("/about");
    expect(renderCount).toBe(2);

    // Navigate with replace
    navigate("/contact", { replace: true });

    await new Promise(r => setTimeout(r, 10));
    expect(currentLocation).toBe("/contact");
    expect(renderCount).toBe(3);
  });
});

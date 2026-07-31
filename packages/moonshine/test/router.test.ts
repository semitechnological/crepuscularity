import { describe, expect, test, mock, beforeEach, afterEach } from "bun:test";
import { Link, navigate } from "../src/router";

describe("Link", () => {
  let mockEvent: any;

  beforeEach(() => {
    mockEvent = {
      defaultPrevented: false,
      button: 0,
      metaKey: false,
      altKey: false,
      ctrlKey: false,
      shiftKey: false,
      preventDefault: mock(() => {}),
    };

    // Setup window and its origin for URL constructor
    globalThis.window = {
      location: {
        pathname: "/",
        origin: "http://localhost"
      },
      history: {
        pushState: mock(() => {}),
        replaceState: mock(() => {}),
      },
      dispatchEvent: mock(() => {}),
    } as any;
    globalThis.Event = class Event {
      constructor(type: string) {}
    } as any;
  });

  afterEach(() => {
    delete (globalThis as any).window;
    delete (globalThis as any).Event;
  });

  test("renders an anchor tag with correct properties", () => {
    const link = Link({ href: "/test", className: "my-link", children: "Test Link" });
    const linkObj = link as any;
    expect(linkObj.type).toBe("a");
    expect(linkObj.props.href).toBe("/test");
    expect(linkObj.props.className).toBe("my-link");
    expect(linkObj.props.children).toBe("Test Link");
    expect(typeof linkObj.props.onClick).toBe("function");
  });

  test("onClick prevents default and navigates", () => {
    const link = Link({ href: "/test" });
    const onClick = (link as any).props.onClick;

    onClick(mockEvent);

    expect(mockEvent.preventDefault).toHaveBeenCalled();
    expect(globalThis.window.history.pushState).toHaveBeenCalledWith({}, "", "/test");
  });

  test("onClick uses replaceState when replace prop is true", () => {
    const link = Link({ href: "/test-replace", replace: true });
    const onClick = (link as any).props.onClick;

    onClick(mockEvent);

    expect(mockEvent.preventDefault).toHaveBeenCalled();
    expect(globalThis.window.history.replaceState).toHaveBeenCalledWith({}, "", "/test-replace");
  });

  test("onClick does nothing if default is prevented", () => {
    const link = Link({ href: "/test" });
    const onClick = (link as any).props.onClick;

    mockEvent.defaultPrevented = true;
    onClick(mockEvent);

    expect(mockEvent.preventDefault).not.toHaveBeenCalled();
    expect(globalThis.window.history.pushState).not.toHaveBeenCalled();
  });

  test("onClick does nothing for non-left clicks", () => {
    const link = Link({ href: "/test" });
    const onClick = (link as any).props.onClick;

    mockEvent.button = 1; // Middle click
    onClick(mockEvent);

    expect(mockEvent.preventDefault).not.toHaveBeenCalled();
    expect(globalThis.window.history.pushState).not.toHaveBeenCalled();
  });

  test("onClick does nothing with modifier keys", () => {
    const link = Link({ href: "/test" });
    const onClick = (link as any).props.onClick;

    const modifiers = ['metaKey', 'altKey', 'ctrlKey', 'shiftKey'];

    modifiers.forEach(mod => {
      // Reset
      mockEvent = {
        defaultPrevented: false,
        button: 0,
        metaKey: false,
        altKey: false,
        ctrlKey: false,
        shiftKey: false,
        preventDefault: mock(() => {}),
      };

      mockEvent[mod] = true;
      onClick(mockEvent);

      expect(mockEvent.preventDefault).not.toHaveBeenCalled();
    });
  });
});

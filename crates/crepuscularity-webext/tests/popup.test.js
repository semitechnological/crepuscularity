import { describe, it, expect, vi, beforeEach, afterEach } from "bun:test";
import { browserApi } from "../assets/browser-shim.js";

describe("popup.js", () => {
  let changeListener;
  let clickListener;

  beforeEach(async () => {
    document.body.innerHTML = `
      <div id="view-main"></div>
      <div id="view-help"></div>
      <div id="view-crepus"></div>
      <input type="checkbox" id="enabled" />
      <input type="checkbox" id="autoRender" />
      <input type="checkbox" data-action="set-enabled" id="action-enabled" />
      <input type="checkbox" data-action="set-auto-render" id="action-auto-render" />
      <div data-action="copy-prompt" id="action-copy-prompt">
        <span class="material-symbols-outlined">content_copy</span>
      </div>
      <pre id="system-prompt">Test prompt</pre>
      <div data-action="hide-help" id="action-hide-help"></div>
    `;

    const originalAddEventListener = document.addEventListener;
    vi.spyOn(document, 'addEventListener').mockImplementation((event, handler) => {
      if (event === "change") changeListener = handler;
      if (event === "click") clickListener = handler;
      originalAddEventListener.call(document, event, handler);
    });

    vi.spyOn(browserApi.storage.sync, 'set').mockResolvedValue(undefined);
    vi.spyOn(browserApi.storage.sync, 'get').mockResolvedValue({});

    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText: vi.fn().mockResolvedValue() },
      writable: true,
      configurable: true,
    });

    const cacheBuster = `?t=${Date.now()}_${Math.random()}`;
    await import(`../assets/popup.js${cacheBuster}`);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("error paths", () => {
    it("handles storage sync set errors for set-enabled", async () => {
      const mockError = new Error("Storage quota exceeded");
      browserApi.storage.sync.set.mockRejectedValueOnce(mockError);

      const actionEl = document.getElementById("action-enabled");
      actionEl.checked = true;

      const event = {
        target: {
          closest: (selector) => selector === "input[data-action]" ? actionEl : null
        }
      };

      await changeListener(event);
      expect(browserApi.storage.sync.set).toHaveBeenCalledWith({ enabled: true });
    });

    it("handles storage sync set errors for set-auto-render", async () => {
      const mockError = new Error("Storage quota exceeded");
      browserApi.storage.sync.set.mockRejectedValueOnce(mockError);

      const actionEl = document.getElementById("action-auto-render");
      actionEl.checked = true;

      const event = {
        target: {
          closest: (selector) => selector === "input[data-action]" ? actionEl : null
        }
      };

      await changeListener(event);
      expect(browserApi.storage.sync.set).toHaveBeenCalledWith({ autoRender: true });
    });

    it("handles copy-prompt clipboard error", async () => {
      navigator.clipboard.writeText.mockRejectedValueOnce(new Error("Clipboard access denied"));

      const actionEl = document.getElementById("action-copy-prompt");

      const event = {
        target: {
          closest: (selector) => selector === "[data-action]" ? actionEl : null
        }
      };

      await clickListener(event);
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith("Test prompt");
    });

    it("handles storage sync get errors in syncState", async () => {
      const mockError = new Error("Storage unavailable");
      browserApi.storage.sync.get.mockRejectedValueOnce(mockError);

      const actionEl = document.getElementById("action-hide-help");
      const event = {
        target: {
          closest: (selector) => selector === "[data-action]" ? actionEl : null
        }
      };

      // Trigger syncState via hide-help action
      await clickListener(event);

      // Fallback defaults should be applied
      expect(document.getElementById("enabled").checked).toBe(true);
      expect(document.getElementById("autoRender").checked).toBe(false);
    });
  });
});

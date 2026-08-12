import { describe, expect, it, mock, beforeEach, afterEach } from "bun:test";
import { GlobalRegistrator } from "@happy-dom/global-registrator";

const mockGet = mock();
const mockSet = mock();

mock.module("../browser-shim.js", () => {
  return {
    browserApi: {
      storage: {
        sync: {
          get: mockGet,
          set: mockSet,
        },
      },
    },
  };
});

GlobalRegistrator.register();

// We must bypass Bun's module caching if we want to run the top-level script again
// or we can just trigger the action that calls it.

describe("popup.js", () => {
    beforeEach(async () => {
        document.body.innerHTML = `
            <div id="view-main" hidden="true"></div>
            <div id="view-help" hidden="true"></div>
            <div id="view-crepus" hidden="true"></div>
            <input type="checkbox" id="enabled" />
            <input type="checkbox" id="autoRender" />
            <button data-action="hide-help"></button>
            <input type="checkbox" id="set-enabled" data-action="set-enabled" />
            <input type="checkbox" id="set-auto-render" data-action="set-auto-render" />
        `;
        mockGet.mockClear();
        mockSet.mockClear();
    });

    afterEach(() => {
        document.body.innerHTML = "";
    });

    it("should handle error in syncState via hide-help action", async () => {
        // Since the script evaluates on import, we need to load it once.
        mockGet.mockReturnValue(Promise.resolve({ enabled: false, autoRender: true }));
        await import("../popup.js");
        await new Promise(resolve => setTimeout(resolve, 10)); // let it settle

        const enabledCb = document.getElementById("enabled");
        const autoRenderCb = document.getElementById("autoRender");

        expect(enabledCb.checked).toBe(false);
        expect(autoRenderCb.checked).toBe(true);

        // Now trigger the error case
        mockGet.mockReturnValue(Promise.reject(new Error("Storage failed")));

        // Trigger hide-help to invoke syncState again
        const hideHelpBtn = document.querySelector('[data-action="hide-help"]');
        hideHelpBtn.click();

        // Wait for next tick to let async syncState complete
        await new Promise(resolve => setTimeout(resolve, 10));

        expect(enabledCb.checked).toBe(true); // DEFAULTS.enabled
        expect(autoRenderCb.checked).toBe(false); // DEFAULTS.autoRender
    });

    it("should handle error in set-enabled action", async () => {
        mockSet.mockReturnValue(Promise.reject(new Error("Set failed")));

        const setEnabled = document.getElementById("set-enabled");
        setEnabled.checked = true;
        // manually dispatch change event as click/setting checked might not bubble it in happy-dom exactly as needed
        setEnabled.dispatchEvent(new Event('change', { bubbles: true }));

        await new Promise(resolve => setTimeout(resolve, 10));

        expect(mockSet).toHaveBeenCalledWith({ enabled: true });
        // Just verify no unhandled rejection crashes the test
    });

    it("should handle error in set-auto-render action", async () => {
        mockSet.mockReturnValue(Promise.reject(new Error("Set failed")));

        const setAutoRender = document.getElementById("set-auto-render");
        setAutoRender.checked = true;
        setAutoRender.dispatchEvent(new Event('change', { bubbles: true }));

        await new Promise(resolve => setTimeout(resolve, 10));

        expect(mockSet).toHaveBeenCalledWith({ autoRender: true });
        // Just verify no unhandled rejection crashes the test
    });
});

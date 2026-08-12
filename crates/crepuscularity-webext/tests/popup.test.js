import { test, expect, describe, mock, beforeEach, afterEach } from "bun:test";
import { GlobalRegistrator } from "@happy-dom/global-registrator";

let mockBrowserApi;

describe("popup.js tests", () => {
    let originalConsoleError;
    let originalClipboard;

    beforeEach(async () => {
        GlobalRegistrator.register();

        originalConsoleError = console.error;
        console.error = mock();
        originalClipboard = navigator.clipboard;

        // Setup DOM for popup.js
        document.body.innerHTML = `
            <div id="view-main"></div>
            <div id="view-help"></div>
            <div id="view-crepus"></div>

            <input type="checkbox" id="enabled" />
            <input type="checkbox" id="autoRender" />

            <div data-action="show-help">Show Help</div>
            <div data-action="hide-help">Hide Help</div>

            <div id="system-prompt">Prompt text</div>
            <div data-action="copy-prompt"><span class="material-symbols-outlined">content_copy</span></div>

            <input type="checkbox" data-action="set-enabled" />
            <input type="checkbox" data-action="set-auto-render" />
        `;

        mockBrowserApi = {
            storage: {
                sync: {
                    get: mock(() => Promise.resolve({})),
                    set: mock(() => Promise.resolve())
                }
            }
        };

        mock.module("../assets/browser-shim.js", () => {
            return {
                browserApi: mockBrowserApi
            };
        });

        // We need a unique query string so the module re-evaluates
        const nonce = Math.random().toString(36).substring(7);
        await import(`../assets/popup.js?nonce=${nonce}`);
        // Allow the syncState to finish
        await new Promise(r => setTimeout(r, 10));
    });

    afterEach(() => {
        console.error = originalConsoleError;
        Object.defineProperty(navigator, 'clipboard', {
            value: originalClipboard,
            configurable: true,
        });
        GlobalRegistrator.unregister();
        mock.restore();
    });

    test("handles errors when toggling set-enabled", async () => {
        const error = new Error("Storage failure");
        mockBrowserApi.storage.sync.set.mockImplementation(() => Promise.reject(error));

        const input = document.querySelector('input[data-action="set-enabled"]');
        input.checked = true;

        const event = new Event("change", { bubbles: true });
        input.dispatchEvent(event);

        await new Promise(r => setTimeout(r, 10));

        expect(mockBrowserApi.storage.sync.set).toHaveBeenCalledWith({ enabled: true });
        expect(console.error).not.toHaveBeenCalled();
    });

    test("handles errors when toggling set-auto-render", async () => {
        const error = new Error("Storage failure");
        mockBrowserApi.storage.sync.set.mockImplementation(() => Promise.reject(error));

        const input = document.querySelector('input[data-action="set-auto-render"]');
        input.checked = true;

        const event = new Event("change", { bubbles: true });
        input.dispatchEvent(event);

        await new Promise(r => setTimeout(r, 10));

        expect(mockBrowserApi.storage.sync.set).toHaveBeenCalledWith({ autoRender: true });
        expect(console.error).not.toHaveBeenCalled();
    });

    test("handles errors when copying prompt", async () => {
        const error = new Error("Clipboard failure");
        Object.defineProperty(navigator, 'clipboard', {
            value: { writeText: mock(() => Promise.reject(error)) },
            configurable: true,
        });

        const btn = document.querySelector('div[data-action="copy-prompt"]');

        const event = new Event("click", { bubbles: true });
        btn.dispatchEvent(event);

        await new Promise(r => setTimeout(r, 10));

        expect(navigator.clipboard.writeText).toHaveBeenCalledWith("Prompt text");
        expect(console.error).not.toHaveBeenCalled();
    });
});

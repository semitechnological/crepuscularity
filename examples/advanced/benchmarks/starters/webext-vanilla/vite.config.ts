import { defineConfig } from "vite";
import { crx } from "@crxjs/vite-plugin";

export default defineConfig({
  plugins: [
    crx({
      manifest: {
        manifest_version: 3,
        name: "bench-vanilla-ext",
        version: "0.0.1",
        action: {
          default_popup: "src/popup.html",
        },
      },
    }),
  ],
});

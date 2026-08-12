import { GlobalRegistrator } from "@happy-dom/global-registrator";
GlobalRegistrator.register();

globalThis.browser = {
  runtime: {
    getURL: () => {},
    onMessage: {}
  },
  storage: {
    sync: {
      get: () => Promise.resolve({}),
      set: () => Promise.resolve()
    }
  }
};

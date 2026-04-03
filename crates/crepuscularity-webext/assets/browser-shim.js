const nativeApi = globalThis.browser ?? globalThis.chrome;

function promisifyChrome(fn, ctx, ...args) {
  return new Promise((resolve, reject) => {
    fn.call(ctx, ...args, (result) => {
      const error = globalThis.chrome?.runtime?.lastError;
      if (error) {
        reject(new Error(error.message));
        return;
      }
      resolve(result);
    });
  });
}

function storageArea(area) {
  const api = nativeApi.storage[area];
  return {
    async get(keys) {
      if (api.get.length <= 1) {
        return api.get(keys);
      }
      return promisifyChrome(api.get, api, keys);
    },
    async set(value) {
      if (api.set.length <= 1) {
        return api.set(value);
      }
      return promisifyChrome(api.set, api, value);
    }
  };
}

export const browserApi = {
  runtime: {
    getURL: (path) => nativeApi.runtime.getURL(path),
    onMessage: nativeApi.runtime.onMessage,
    async sendMessage(payload) {
      if (nativeApi.runtime.sendMessage.length <= 1) {
        return nativeApi.runtime.sendMessage(payload);
      }
      return promisifyChrome(nativeApi.runtime.sendMessage, nativeApi.runtime, payload);
    }
  },
  storage: {
    local: storageArea("local"),
    sync: storageArea("sync")
  }
};

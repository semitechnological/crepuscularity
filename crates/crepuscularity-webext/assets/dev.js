(function () {
  var api = globalThis.browser ?? globalThis.chrome;
  if (!api || !api.runtime) return;
  var lastId = 0;
  setInterval(function () {
    fetch(api.runtime.getURL("src/.reload-id"), { cache: "no-store" })
      .then(function (r) { return r.text(); })
      .then(function (text) {
        var id = parseInt(text, 10) || 0;
        if (id > 0 && lastId > 0 && id !== lastId) {
          location.reload();
        }
        lastId = id;
      })
      .catch(function () {});
  }, 1500);
})();

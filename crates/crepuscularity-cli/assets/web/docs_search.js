(function () {
  var overlay = document.getElementById("doc-search-overlay");
  var input = document.getElementById("doc-search-input");
  var list = document.getElementById("doc-search-results");
  var openBtn = document.getElementById("doc-search-open");
  if (!overlay || !input || !list) return;

  var index = [];
  var loaded = false;
  var loadError = "";

  // Resolve against the docs *directory* (not the HTML filename) so GitHub Pages
  // paths like /repo/docs/cli.html load /repo/docs/docs-search-index.json.
  var INDEX_URLS = (function () {
    try {
      var page = new URL(window.location.href);
      var path = page.pathname;
      var dir;
      var urls = [];
      var inDocs = path.indexOf("/docs/") >= 0;
      if (path === "" || path.endsWith("/")) {
        dir = path;
      } else if (/\.html?$/i.test(path)) {
        dir = path.replace(/[^/]+$/, "");
      } else {
        dir = path.endsWith("/") ? path : path + "/";
      }
      var base = new URL(dir || "/", page.origin);
      if (inDocs) {
        urls.push(new URL("docs-search-index.json", base).href);
        urls.push(new URL("../docs-search-index.json", base).href);
        urls.push(new URL("/docs/docs-search-index.json", page.origin).href);
      } else {
        urls.push(new URL("docs/docs-search-index.json", base).href);
        urls.push(new URL("/docs/docs-search-index.json", page.origin).href);
        urls.push(new URL("docs-search-index.json", base).href);
      }
      return urls;
    } catch (err) {
      return ["docs-search-index.json", "docs/docs-search-index.json"];
    }
  })();

  var HIDDEN_CLASS = "doc-search-overlay--hidden";

  function setOverlayVisible(visible) {
    if (visible) {
      overlay.classList.remove(HIDDEN_CLASS);
      overlay.setAttribute("aria-hidden", "false");
    } else {
      overlay.classList.add(HIDDEN_CLASS);
      overlay.setAttribute("aria-hidden", "true");
    }
  }

  function load() {
    if (loaded) return Promise.resolve();
    loadError = "";
    function tryNext(i) {
      if (i >= INDEX_URLS.length) {
        return Promise.reject(new Error("search index not found"));
      }
      return fetch(INDEX_URLS[i], { cache: "no-store" })
        .then(function (r) {
          if (!r.ok) {
            throw new Error("HTTP " + r.status);
          }
          return r.json();
        })
        .then(function (d) {
          index = Array.isArray(d.entries) ? d.entries : [];
          loaded = true;
        })
        .catch(function (e) {
          if (i + 1 < INDEX_URLS.length) {
            return tryNext(i + 1);
          }
          loadError = e.message || "fetch failed";
          index = [];
          loaded = true;
          console.warn("[crepus docs] search index:", loadError, INDEX_URLS[0]);
        });
    }

    return tryNext(0);
  }

  function score(q, s) {
    if (!q) return 0;
    q = q.toLowerCase();
    s = (s || "").toLowerCase();
    var qi = 0;
    var run = 0;
    var bonus = 0;
    for (var i = 0; i < s.length && qi < q.length; i++) {
      if (s.charCodeAt(i) === q.charCodeAt(qi)) {
        qi++;
        run++;
        bonus += run * 3;
      } else {
        run = 0;
      }
    }
    if (qi < q.length) return 0;
    return 100 + bonus;
  }

  function rank(q, entries) {
    if (!entries.length) return [];
    if (!q.trim()) return entries.slice(0, 14);
    var parts = q
      .toLowerCase()
      .split(/\s+/)
      .filter(Boolean);
    return entries
      .map(function (e) {
        var hay = (e.title + " " + e.text).toLowerCase();
        var ok = parts.every(function (p) {
          return hay.indexOf(p) >= 0 || score(p, hay) > 0;
        });
        if (!ok) return { e: e, s: 0 };
        var s =
          score(q, e.title) * 4 +
          score(q, e.text) +
          parts.reduce(function (a, p) {
            return (
              a +
              (e.title.toLowerCase().indexOf(p) >= 0 ? 40 : 0) +
              (e.text.toLowerCase().indexOf(p) >= 0 ? 12 : 0)
            );
          }, 0);
        return { e: e, s: s };
      })
      .filter(function (x) {
        return x.s > 0;
      })
      .sort(function (a, b) {
        return b.s - a.s;
      })
      .slice(0, 24)
      .map(function (x) {
        return x.e;
      });
  }

  function render(results) {
    list.innerHTML = "";
    if (loadError && !index.length) {
      var err = document.createElement("li");
      err.className = "doc-search-error";
      err.textContent =
        "Could not load search index (" +
        loadError +
        "). Check that docs-search-index.json exists next to this page.";
      list.appendChild(err);
      return;
    }
    for (var i = 0; i < results.length; i++) {
      var e = results[i];
      var li = document.createElement("li");
      var a = document.createElement("a");
      a.href = e.href;
      a.textContent = e.title;
      var sn = document.createElement("span");
      sn.className = "doc-search-snippet";
      sn.textContent = (e.text || "").replace(/\s+/g, " ").slice(0, 120);
      li.appendChild(a);
      li.appendChild(sn);
      list.appendChild(li);
    }
    if (!results.length && index.length) {
      var empty = document.createElement("li");
      empty.className = "doc-search-empty";
      empty.textContent = "No matches — try different keywords.";
      list.appendChild(empty);
    }
  }

  function openPalette() {
    setOverlayVisible(true);
    document.documentElement.style.overflow = "hidden";
    load().then(function () {
      input.focus();
      input.select();
      render(rank(input.value, index));
    });
  }

  function closePalette() {
    setOverlayVisible(false);
    document.documentElement.style.overflow = "";
  }

  function isOpen() {
    return !overlay.classList.contains(HIDDEN_CLASS);
  }

  if (openBtn) {
    openBtn.addEventListener("click", function (ev) {
      ev.preventDefault();
      openPalette();
    });
  }

  document.addEventListener("keydown", function (ev) {
    if ((ev.metaKey || ev.ctrlKey) && String(ev.key).toLowerCase() === "k") {
      ev.preventDefault();
      if (isOpen()) closePalette();
      else openPalette();
    }
    if (ev.key === "Escape" && isOpen()) {
      ev.preventDefault();
      closePalette();
    }
  });

  overlay.addEventListener("click", function (ev) {
    if (ev.target === overlay) closePalette();
  });

  input.addEventListener("input", function () {
    if (!loaded) return;
    render(rank(input.value, index));
  });

  list.addEventListener("click", function (ev) {
    var a = ev.target.closest("a");
    if (a) closePalette();
  });

  setOverlayVisible(false);
})();

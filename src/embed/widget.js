// Beerview tap list embed widget v1
(function () {
  "use strict";

  var script = document.currentScript;
  if (!script) {
    console.error("Beerview: cannot find script element");
    return;
  }

  var pubSlug = script.getAttribute("data-pub");
  if (!pubSlug) {
    console.error("Beerview: missing data-pub attribute on script tag");
    return;
  }

  var apiBase = (new URL(script.src)).origin;
  var apiUrl = apiBase + "/v1/pubs/" + encodeURIComponent(pubSlug) + "/taps";
  var refreshMs = 5 * 60 * 1000; // 5 minutes

  var container = document.createElement("div");
  container.className = "beerview-widget";
  container.setAttribute("aria-live", "polite");
  script.parentNode.insertBefore(container, script.nextSibling);

  function esc(str) {
    if (str == null) return "";
    var d = document.createElement("div");
    d.appendChild(document.createTextNode(String(str)));
    return d.innerHTML;
  }

  function renderTap(tap) {
    var b = tap.beer;
    if (!b) return "";
    var parts = [];
    if (b.style) parts.push(esc(b.style));
    if (b.abv != null) parts.push(esc(b.abv.toFixed(1)) + "\u00a0%");
    var meta = parts.length ? ' <span class="bw-meta">(' + parts.join(", ") + ")</span>" : "";

    var prices = "";
    if (tap.prices && tap.prices.length) {
      prices = ' <span class="bw-prices">' +
        tap.prices.map(function (p) {
          return esc(p.size) + "\u00a0" + esc(p.price) + "\u00a0K\u010d";
        }).join(", ") + "</span>";
    }

    return '<li class="bw-tap">' +
      '<strong class="bw-beer-name">' + esc(b.name) + "</strong>" +
      ' <span class="bw-brewery">' + esc(b.brewery) + "</span>" +
      meta + prices +
      "</li>";
  }

  function render(data) {
    var activeTaps = (data.taps || []).filter(function (t) { return t.beer != null; });
    if (activeTaps.length === 0) {
      container.innerHTML = '<p class="bw-empty">No beers currently on tap.</p>';
      return;
    }
    var html = '<div class="bw-header"><strong>' + esc(data.pub_name) + "</strong> &mdash; On Tap</div>" +
      '<ul class="bw-tap-list">' +
      activeTaps.map(renderTap).join("") +
      "</ul>";
    container.innerHTML = html;
  }

  function load() {
    fetch(apiUrl)
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.json();
      })
      .then(render)
      .catch(function (err) {
        console.warn("Beerview: could not load tap list", err);
        if (!container.querySelector(".bw-tap-list")) {
          container.innerHTML = '<p class="bw-empty">Could not load tap list.</p>';
        }
      });
  }

  load();
  setInterval(load, refreshMs);
})();

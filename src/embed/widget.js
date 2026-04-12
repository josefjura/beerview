// Beerview embed widget
// Full implementation: see docs/technical-guidelines.md Section 9
(function () {
    "use strict";
    var script = document.currentScript;
    var pubSlug = script.getAttribute("data-pub");
    if (!pubSlug) {
        console.error("Beerview widget: missing data-pub attribute");
        return;
    }
    var apiBase = new URL(script.src).origin;
    var apiUrl = apiBase + "/v1/pubs/" + encodeURIComponent(pubSlug) + "/taps";
    var container = document.createElement("div");
    container.className = "beerview-widget";
    container.innerHTML = "<p>Loading tap list...</p>";
    script.parentNode.insertBefore(container, script.nextSibling);
    fetch(apiUrl)
        .then(function (r) { if (!r.ok) throw new Error("HTTP " + r.status); return r.json(); })
        .then(function (data) {
            if (!data.taps || data.taps.length === 0) {
                container.innerHTML = "<p>No beers currently on tap.</p>";
                return;
            }
            var html = "<div class=\"beerview-taps\"><h3>" + esc(data.pub_name) + " — On Tap</h3><ul>";
            data.taps.forEach(function (t) {
                html += "<li><strong>" + esc(t.beer.name) + "</strong> by " + esc(t.beer.brewery) + "</li>";
            });
            html += "</ul></div>";
            container.innerHTML = html;
        })
        .catch(function () { container.innerHTML = "<p>Could not load tap list.</p>"; });
    function esc(s) {
        if (!s) return "";
        var d = document.createElement("div");
        d.appendChild(document.createTextNode(s));
        return d.innerHTML;
    }
})();

/* eslint-disable */
/**
 * Browser-side test transport. Executed inside the page by Playwright with the
 * recorded fixture passed in as `fixture`.
 *
 * Read commands are answered from recordings made by the REAL Rust core
 * (`cargo run -p pw-cli --example dev_fixtures`), so no query logic is
 * reimplemented here. Write commands patch those recordings just enough to
 * exercise the UI's optimistic updates.
 *
 * Test scaffolding only — never bundled into the desktop application.
 */
(function () {
  var responses = fixture.responses;

  // Recorded keys were built from serde_json::Value, which (without the
  // preserve_order feature) stores object fields in a BTreeMap and therefore
  // serializes every object — including nested ones like `filters` — with
  // alphabetically sorted keys. Mirror that here so a runtime call's argument
  // object hashes to the same key regardless of the property order it was
  // constructed in.
  function sortKeysDeep(value) {
    if (Array.isArray(value)) return value.map(sortKeysDeep);
    if (value && typeof value === 'object') {
      var sorted = {};
      Object.keys(value)
        .sort()
        .forEach(function (key) {
          sorted[key] = sortKeysDeep(value[key]);
        });
      return sorted;
    }
    return value;
  }

  function canonical(args) {
    return JSON.stringify(sortKeysDeep(args || {}));
  }

  function lookup(cmd, args) {
    var direct = responses[cmd + '|' + canonical(args)];
    if (direct !== undefined) return direct;
    return responses[cmd + '|{}'];
  }

  var clipboard = [];
  var opened = [];
  window.__PW_CLIPBOARD__ = clipboard;
  window.__PW_OPENED__ = opened;

  function patchFavorite(itemId, favorite) {
    function walk(node) {
      if (Array.isArray(node)) {
        node.forEach(walk);
        return;
      }
      if (node && typeof node === 'object') {
        if (node.id === itemId && 'isFavorite' in node) node.isFavorite = favorite;
        Object.keys(node).forEach(function (key) {
          walk(node[key]);
        });
      }
    }
    Object.keys(responses).forEach(function (key) {
      walk(responses[key]);
    });
  }

  window.__PW_FIXTURE_INVOKE__ = function (cmd, args) {
    args = args || {};
    switch (cmd) {
      case 'set_favorite':
        patchFavorite(args.itemId, Boolean(args.favorite));
        return Promise.resolve(Boolean(args.favorite));
      case 'set_pin':
        return Promise.resolve(Boolean(args.pinned));
      case 'record_open':
      case 'set_setting':
      case 'clear_search_history':
      case 'delete_saved_search':
      case 'delete_note':
      case 'delete_link':
      case 'rebuild_search_index':
        return Promise.resolve(null);
      case 'copy_text':
        clipboard.push(String(args.text == null ? '' : args.text));
        return Promise.resolve(null);
      case 'open_external': {
        var url = String(args.url == null ? '' : args.url);
        // The real command validates the URL in Rust; mirror only the refusal
        // so a test can prove the UI surfaces the error.
        if (!/^https?:\/\//i.test(url)) {
          return Promise.reject(new Error('unsupported url scheme in "' + url + '"'));
        }
        opened.push(url);
        return Promise.resolve(null);
      }
      case 'pick_pack_file':
        return Promise.resolve(null);
      default: {
        var value = lookup(cmd, args);
        if (value === undefined) {
          return Promise.reject(
            new Error('no recorded response for ' + cmd + ' ' + JSON.stringify(args))
          );
        }
        return Promise.resolve(value);
      }
    }
  };
})();

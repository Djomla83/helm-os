/* HELM G2 — non-product interaction prototype.
 *
 * A faithful port of the owner-approved G2-D8 designer final to plain
 * DOM + vanilla JavaScript. No framework, no build step, no network.
 *
 * NOTHING HERE TOUCHES THIS COMPUTER. No file is opened, no process is
 * created, no receipt is produced by HELM, nothing is persisted. Every
 * path, byte count, digest, exit code and receipt value below is a fixed
 * constant written into this file.
 */
(function () {
  "use strict";

  /* The deadline the designer final exposes as a plan value. Fixed here;
     the prototype selects no bounds and asks HELM for nothing. */
  var DEADLINE_SECONDS = 30;

  /* Mock receipt bytes, exactly as shown on the Evidence screen. */
  var RECEIPT_BYTES =
    '{"receipt_version":"0.1","backend":"linux_x86_64_clone3_pidfd_execveat",' +
    '"exec_status":{"indeterminate":"status_eof_without_record"},' +
    '"child_end":{"exited":{"code":0}},"sigterm_sent":false,' +
    '"sigkill_sent":false,"group_sweep":"issued",' +
    '"stdout":{"drained":1284,"completeness":"complete_at_eof"},' +
    '"stderr":{"drained":0,"completeness":"complete_at_eof"},' +
    '"pre_exec":{"size":41984,"sha256":"9d41…c7e0","mode_bits":493,' +
    '"elf_type":"EXEC"},"plan_sha256":"4f1c…9ab2"}';

  var REFUSALS = {
    SET_ID_BITS_PRESENT: [
      "Refused — the file has set-id bits",
      "HELM does not admit a file that would change user or group identity when it runs. This is a property of the file, not a judgement about it.",
      "set_id_bits_present"
    ],
    NOT_ELF: [
      "Refused — the file is not an ELF executable",
      "HELM admits only ELF executables on this computer. This file is something else, so there is nothing to launch.",
      "not_elf"
    ]
  };

  var state = {
    screen: "library",
    entry: "known", // none | known | available | attempt | ended
    program: "admitted", // none | admitted | refused
    refusal: null,
    folder: true,
    chooserOpen: false,
    outputOpen: false,
    advanced: true,
    copyLabel: "Copy the exact receipt bytes",
    elapsed: 0
  };

  var tick = null;
  var done = null;
  var copyReset = null;

  function clearTimers() {
    if (tick) { clearInterval(tick); tick = null; }
    if (done) { clearTimeout(done); done = null; }
  }

  function setState(patch) {
    for (var key in patch) {
      if (Object.prototype.hasOwnProperty.call(patch, key)) state[key] = patch[key];
    }
    render();
  }

  /* ---------- derived values (the designer final's renderVals) ---------- */

  function stateWord() {
    switch (state.entry) {
      case "available": return "Launch available";
      case "attempt": return "Launch attempt in progress";
      case "ended": return "Ended";
      default: return "Known";
    }
  }

  function stateNote() {
    switch (state.entry) {
      case "available": return "A single-use authorisation exists.";
      case "attempt": return "HELM's launch call has not returned. Not a fact about the program.";
      case "ended": return "The direct child was observed to end, reporting 0.";
      default: return "Program and folder admitted. No attempt made.";
    }
  }

  function shapeFor(entry) {
    switch (entry) {
      case "available": return "shape--available";
      case "attempt": return "shape--attempt";
      case "ended": return "shape--ended";
      default: return "shape--known";
    }
  }

  function values() {
    var r = state.refusal ? REFUSALS[state.refusal] : null;
    var elapsed = Math.min(state.elapsed, DEADLINE_SECONDS);
    var chosen = state.program === "admitted" && state.folder;

    return {
      advanced: state.advanced,

      onLibrary: state.screen === "library",
      onChoose: state.screen === "choose",
      onProgram: state.screen === "program",
      onAuthority: state.screen === "authority",
      onAttempt: state.screen === "attempt",
      onResult: state.screen === "result",
      onEvidence: state.screen === "evidence",

      railLabel: state.entry === "none" ? "Nothing open" : "Held in memory",
      railNote: state.entry === "none"
        ? "HELM knows nothing about any program yet."
        : "Entries live until HELM closes. Nothing was written to disk.",

      libraryEmpty: state.entry === "none",
      libraryHasEntry: state.entry !== "none",
      entryStateWord: stateWord(),
      entryStateNote: stateNote(),
      entryShape: shapeFor(state.entry),

      noProgram: state.program === "none",
      programAdmitted: state.program === "admitted",
      programRefused: state.program === "refused",
      programPhase: state.program === "admitted"
        ? "Admitted"
        : state.program === "refused" ? "Refused" : "Not checked yet",
      refusalTitle: r ? r[0] : "",
      refusalText: r ? r[1] : "",
      refusalCode: r ? r[2] : "",

      folderAdmitted: chosen,
      folderPending: !chosen,
      folderPhase: chosen ? "Admitted" : "Not checked yet",
      chooseComplete: chosen,
      chooserOpen: state.chooserOpen,

      canAuthorise: state.entry === "known" || state.entry === "ended",
      canAttempt: state.entry === "available",
      hasResult: state.entry === "ended",
      noResult: state.entry !== "ended",

      footerSubject: state.entry === "none"
        ? "No program open"
        : "helm-probe — " + stateWord(),
      footerBound: "Run deadline " + DEADLINE_SECONDS + " s",
      footerMode: state.advanced ? "Advanced disclosure" : "Normal disclosure",

      elapsedLabel: elapsed.toFixed(1) + " s of at most " + DEADLINE_SECONDS + " s",
      elapsedPct: Math.min(100, (elapsed / DEADLINE_SECONDS) * 100).toFixed(1) + "%",

      outputOpen: state.outputOpen,
      outputToggleLabel: state.outputOpen ? "Hide what was printed" : "Show what was printed",
      copyLabel: state.copyLabel
    };
  }

  /* ---------- rendering ---------- */

  var screens = null;
  var navItems = null;

  function render() {
    var v = values();

    screens.forEach(function (section) {
      section.hidden = state.screen !== section.getAttribute("data-screen");
    });

    navItems.forEach(function (button) {
      button.classList.toggle("is-active", button.getAttribute("data-go") === state.screen);
      button.setAttribute("aria-current", button.getAttribute("data-go") === state.screen ? "true" : "false");
    });

    Array.prototype.forEach.call(document.querySelectorAll("[data-if]"), function (node) {
      node.hidden = !v[node.getAttribute("data-if")];
    });

    Array.prototype.forEach.call(document.querySelectorAll("[data-bind]"), function (node) {
      node.textContent = v[node.getAttribute("data-bind")];
    });

    Array.prototype.forEach.call(document.querySelectorAll("[data-bind-shape]"), function (node) {
      node.classList.remove("shape--known", "shape--available", "shape--attempt", "shape--ended");
      node.classList.add(v[node.getAttribute("data-bind-shape") + "Shape"]);
    });

    Array.prototype.forEach.call(document.querySelectorAll("[data-bind-width]"), function (node) {
      node.style.width = v[node.getAttribute("data-bind-width")];
    });

    var toggle = document.querySelector('[data-act="toggleOutput"]');
    if (toggle) toggle.setAttribute("aria-expanded", String(state.outputOpen));
  }

  /* ---------- actions ---------- */

  function go(screen) {
    setState({ screen: screen });
  }

  var actions = {
    openChooser: function () { setState({ chooserOpen: true }); },
    closeChooser: function () { setState({ chooserOpen: false }); },
    pickGood: function () {
      setState({ chooserOpen: false, program: "admitted", refusal: null, folder: true });
    },
    pickSetId: function () {
      setState({ chooserOpen: false, program: "refused", refusal: "SET_ID_BITS_PRESENT" });
    },
    pickNotElf: function () {
      setState({ chooserOpen: false, program: "refused", refusal: "NOT_ELF" });
    },
    chooseFolder: function () { setState({ folder: true }); },

    authorise: function () { setState({ screen: "program", entry: "available" }); },
    refuseAuthority: function () { setState({ screen: "program", entry: "known" }); },

    /* A mock attempt. HELM has no cancel and this prototype offers none:
       the attempt runs to its mock conclusion exactly as the approved
       final does. */
    startAttempt: function () {
      clearTimers();
      setState({ screen: "attempt", entry: "attempt", elapsed: 0 });
      var start = Date.now();
      tick = setInterval(function () {
        setState({ elapsed: (Date.now() - start) / 1000 });
      }, 100);
      done = setTimeout(function () {
        clearTimers();
        setState({ screen: "result", entry: "ended" });
      }, 2600);
    },

    closeEntry: function () {
      clearTimers();
      setState({ screen: "library", entry: "none", program: "none", folder: false, refusal: null });
    },

    toggleOutput: function () { setState({ outputOpen: !state.outputOpen }); },
    toggleMode: function () { setState({ advanced: !state.advanced }); },

    /* Copies the exact mock receipt bytes. Where the clipboard is
       unavailable — which it may be under file:// — it says so instead of
       reporting a copy that did not happen. */
    copyBytes: function () {
      if (copyReset) { clearTimeout(copyReset); copyReset = null; }
      var restore = function () {
        copyReset = setTimeout(function () {
          setState({ copyLabel: "Copy the exact receipt bytes" });
        }, 2200);
      };
      var count = typeof TextEncoder === "function"
        ? new TextEncoder().encode(RECEIPT_BYTES).length
        : RECEIPT_BYTES.length;

      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(RECEIPT_BYTES).then(function () {
          setState({ copyLabel: "Copied " + count + " exact bytes" });
          restore();
        }, function () {
          setState({ copyLabel: "Clipboard unavailable — select the bytes below" });
          restore();
        });
      } else {
        setState({ copyLabel: "Clipboard unavailable — select the bytes below" });
        restore();
      }
    }
  };

  /* ---------- wiring ---------- */

  function init() {
    screens = Array.prototype.slice.call(document.querySelectorAll("[data-screen]"));
    navItems = Array.prototype.slice.call(document.querySelectorAll(".navitem"));

    var bytes = document.querySelector("[data-bytes]");
    if (bytes) bytes.textContent = RECEIPT_BYTES;

    document.addEventListener("click", function (event) {
      var node = event.target;
      var target = node && node.closest ? node.closest("[data-go],[data-act]") : null;
      if (!target) return;
      var screen = target.getAttribute("data-go");
      var act = target.getAttribute("data-act");
      if (screen) {
        event.preventDefault();
        if (screen === "choose") {
          setState({ screen: "choose", program: "none", folder: false, refusal: null });
        } else if (screen === "program") {
          setState({ screen: "program", entry: state.entry === "none" ? "known" : state.entry });
        } else {
          go(screen);
        }
      } else if (act && actions[act]) {
        event.preventDefault();
        actions[act]();
      }
    });

    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape" && state.chooserOpen) actions.closeChooser();
    });

    render();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();

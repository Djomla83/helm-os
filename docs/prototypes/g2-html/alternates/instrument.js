/* HELM G2 — REFERENCE / NON-CANONICAL alternate: "Instrument" / panel direction.
   The canonical direction accepted at G2-D8 is ../index.html (Record / graphite frame).
   Mock state transitions only. No backend, no network, no persistence, no process.
   Every string here is prototype copy, not accepted product copy. */

(function () {
  'use strict';

  // ---------------------------------------------------------------- mock data

  var PROGRAM_FILES = [
    {
      name: 'hello-cli',
      meta: 'regular file · 18 456 bytes',
      outcome: 'admitted'
    },
    {
      name: 'report-tool',
      meta: 'regular file · 2 210 304 bytes',
      outcome: 'refused',
      code: 'SET_ID_BITS_PRESENT',
      heading: 'HELM does not admit this file',
      body: 'This file carries set-user-ID or set-group-ID bits. HELM does not admit programs ' +
            'carrying these bits, because it cannot reason about what they change. This is not a ' +
            'statement that the program is dangerous.'
    },
    {
      name: 'notes.txt',
      meta: 'regular file · 1 284 bytes',
      outcome: 'refused',
      code: 'NOT_ELF',
      heading: 'HELM does not admit this file',
      body: 'The first bytes of this file are not an ELF header. HELM runs exact Linux executables ' +
            'and cannot start this object.'
    }
  ];

  var FOLDERS = [
    { name: 'work', meta: 'directory', kind: 'dir', outcome: 'admitted' },
    { name: 'reports', meta: 'directory', kind: 'dir', outcome: 'admitted' },
    {
      name: 'hello-cli',
      meta: 'regular file · 18 456 bytes',
      kind: 'file',
      outcome: 'refused',
      code: 'NOT_A_DIRECTORY',
      heading: 'HELM does not admit this as a working folder',
      body: 'A working folder must be a directory. HELM opened this object and found a regular ' +
            'file, so it has no directory to start the program in.'
    }
  ];

  var RECEIPT_BYTES =
    '{"schema":"helm-launch-receipt","version":"0.1","backend":"linux_x86_64_clone3_pidfd_execveat",' +
    '"plan_sha256":"4b7e9a21c0d83f56ae1b2c9d7048f3e5a6b1c2d093847fe5ab0c1d2e3f405162",' +
    '"working_directory_id":"workdir-01",' +
    '"executable":{"pre_exec_body_size":18456,' +
    '"pre_exec_body_sha256":"9f2c41d7b8e0a35c6d14f8027be59a3c1d0e7f48ab2c9d5e6f10a3b7c8d29e45",' +
    '"pre_exec_mode_bits":493,"elf_type":"et_dyn"},' +
    '"argument_count":1,"environment_mode":"empty",' +
    '"exec_status":{"kind":"indeterminate","reason":"status_eof_without_record"},' +
    '"child_end":{"kind":"exited","code":0},"run_deadline_expired":false,' +
    '"termination":{"sigterm_sent":false,"sigkill_sent":false,"group_sweep":"issued"},' +
    '"stdout":{"bytes_drained":16,' +
    '"drained_sha256":"2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",' +
    '"completeness":"complete_at_eof"},' +
    '"stderr":{"bytes_drained":0,' +
    '"drained_sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",' +
    '"completeness":"complete_at_eof"}}';

  var RUN_BOUND_MS = 30000;      // the plan's timeout_ms, shown as a bound
  var SIMULATED_ATTEMPT_MS = 2183; // prototype only: how long the mock attempt takes

  // ---------------------------------------------------------------- state

  var state = {
    screen: 'library',
    programChosen: false,
    folderChosen: false,
    entry: false,
    authorised: false,
    attempted: false,
    chooserMode: 'program',
    chooserPick: null,
    lastFocus: null
  };

  var $ = function (sel, root) { return (root || document).querySelector(sel); };
  var $$ = function (sel, root) {
    return Array.prototype.slice.call((root || document).querySelectorAll(sel));
  };

  var announcer = $('#announcer');
  function announce(text) {
    announcer.textContent = '';
    window.setTimeout(function () { announcer.textContent = text; }, 40);
  }

  function icon(id, size) {
    return '<svg class="ico" width="' + (size || 15) + '" height="' + (size || 15) +
           '" aria-hidden="true"><use href="#' + id + '"></use></svg>';
  }

  // ---------------------------------------------------------------- routing

  function show(screen) {
    state.screen = screen;
    $$('.screen').forEach(function (el) {
      el.hidden = el.getAttribute('data-screen') !== screen;
    });

    var navTarget = screen;
    if (screen === 'choose') { navTarget = 'library'; }
    if (screen === 'attempt') { navTarget = 'authority'; }
    $$('.nav-item').forEach(function (btn) {
      if (btn.getAttribute('data-goto') === navTarget) {
        btn.setAttribute('aria-current', 'true');
      } else {
        btn.removeAttribute('aria-current');
      }
    });

    $('#main').scrollTop = 0;
  }

  function renderLibrary() {
    $('#library-empty').hidden = state.entry;
    $('#library-list').hidden = !state.entry;
    $('#nav-program').hidden = !state.entry;
    $('#nav-result').disabled = !state.attempted;
    $('#nav-evidence').disabled = !state.attempted;

    var chip = $('#entry-chip');
    var progChip = $('#program-chip');
    var label, ico;

    if (state.attempted) {
      label = 'Ended'; ico = 'i-stop';
      $('#entry-last').textContent = 'Last attempt: a moment ago, this session';
      $('#program-open-result').hidden = false;
      $('#program-result-line').textContent =
        'The program ended and reported 0. HELM did not establish that execution began.';
    } else if (state.authorised) {
      label = 'Launch available'; ico = 'i-disc';
    } else {
      label = 'Known'; ico = 'i-circle';
    }

    [chip, progChip].forEach(function (el) {
      if (!el) { return; }
      el.innerHTML = icon(ico) + ' ' + label;
    });
  }

  // ---------------------------------------------------------------- choose flow

  function setPhase(which, stateName, iconId, statusHtml) {
    var el = $('#phase-' + which);
    el.setAttribute('data-state', stateName);
    $('.ico', el).innerHTML = '<use href="#' + iconId + '"></use>';
    $('#phase-' + which + '-status').innerHTML = statusHtml;
  }

  function clearRefusal(which) { $('#' + which + '-refusal').innerHTML = ''; }

  function showRefusal(which, item) {
    var again = which === 'program' ? 'Choose another file…' : 'Choose another folder…';
    $('#' + which + '-refusal').innerHTML =
      '<div class="refusal">' +
        '<h4>' + icon('i-tri', 16) + item.heading + '</h4>' +
        '<p>' + item.body + '</p>' +
        '<span class="code">' + item.code + '</span>' +
        '<div class="actions" style="margin-top:14px">' +
          '<button type="button" class="btn" data-action="open-chooser" data-mode="' + which + '">' +
            again +
          '</button>' +
          '<span class="small muted">Nothing was kept. HELM closed what it opened.</span>' +
        '</div>' +
      '</div>';
    announce(item.heading + '. ' + item.code + '. Choose another one to continue.');
  }

  function setOpenState() {
    var ready = state.programChosen && state.folderChosen;
    var hint = $('#open-hint');
    $('#btn-open-program').disabled = !ready;
    if (ready) {
      hint.textContent = 'Opening keeps the choice in memory. Nothing is installed, added or written.';
    } else if (!state.programChosen) {
      hint.textContent = 'Waiting for a program file HELM can admit.';
    } else {
      hint.textContent = 'Waiting for a working folder HELM can admit.';
    }
  }

  function resetChoose() {
    state.programChosen = false;
    state.folderChosen = false;
    clearRefusal('program');
    clearRefusal('folder');
    setPhase('program', 'active', 'i-half', 'Nothing chosen yet.');
    setPhase('folder', 'pending', 'i-circle', 'Not checked yet.');
    $('#btn-choose-folder').disabled = true;
    $('#disclose-admission').hidden = true;
    setOpenState();
  }

  function chooseFile(item) {
    clearRefusal('program');
    setPhase('program', 'active', 'i-half', 'Checking the program…');
    window.setTimeout(function () {
      if (item.outcome === 'refused') {
        state.programChosen = false;
        setPhase('program', 'active', 'i-half',
          'HELM opened <span class="mono">/home/you/apps/' + item.name +
          '</span> and did not admit it.');        showRefusal('program', item);
        $('#btn-choose-folder').disabled = true;
        setOpenState();
        return;
      }
      state.programChosen = true;
      setPhase('program', 'done', 'i-disc',
        'Regular file, 18 456 bytes, ELF type <span class="mono">et_dyn</span>, measurement stable.' +
        '<span class="mono path">/home/you/apps/' + item.name + '</span>');
      $('#btn-choose-program').textContent = 'Choose a different file…';
      $('#btn-choose-folder').disabled = false;
      setPhase('folder', 'active', 'i-half', 'Not checked yet.');
      $('#disclose-admission').hidden = false;
      setOpenState();
      announce('Program accepted. Choose a working folder next.');
    }, 520);
  }

  function chooseFolder(item) {
    clearRefusal('folder');
    setPhase('folder', 'active', 'i-half', 'Checking the folder…');
    window.setTimeout(function () {
      if (item.outcome === 'refused') {
        state.folderChosen = false;
        setPhase('folder', 'active', 'i-half',
          'HELM opened <span class="mono">/home/you/' + item.name + '</span> and did not admit it.');
        showRefusal('folder', item);
        setOpenState();
        return;
      }
      state.folderChosen = true;
      setPhase('folder', 'done', 'i-disc',
        'Directory, identifier <span class="mono">workdir-01</span>.' +
        '<span class="mono path">/home/you/' + item.name + '</span>');
      $('#btn-choose-folder').textContent = 'Choose a different folder…';
      setOpenState();
      announce('Working folder accepted.');
    }, 460);
  }

  // ---------------------------------------------------------------- chooser dialog

  var dialog = $('#chooser');

  function openChooser(mode) {
    state.chooserMode = mode;
    state.chooserPick = null;
    var list = mode === 'program' ? PROGRAM_FILES : FOLDERS;
    $('#chooser-title').textContent =
      mode === 'program' ? 'Choose a program file' : 'Choose a working folder';
    $('#chooser-path').textContent = mode === 'program' ? '/home/you/apps' : '/home/you';
    $('#chooser-accept').disabled = true;

    $('#chooser-files').innerHTML = list.map(function (item, i) {
      var glyph = (mode === 'folder' && item.kind === 'dir') ? 'i-folder' : 'i-file';
      return '<button type="button" class="file-row" data-index="' + i + '" aria-pressed="false">' +
               icon(glyph, 16) +
               '<span class="nm">' + item.name + '</span>' +
               '<span class="mt">' + item.meta + '</span>' +
             '</button>';
    }).join('');

    if (typeof dialog.showModal === 'function') { dialog.showModal(); }
    else { dialog.setAttribute('open', ''); }
  }

  function closeChooser() {
    if (typeof dialog.close === 'function' && dialog.open) { dialog.close(); }
    else { dialog.removeAttribute('open'); }
  }

  function acceptChooser() {
    if (state.chooserPick === null) { return; }
    var list = state.chooserMode === 'program' ? PROGRAM_FILES : FOLDERS;
    var item = list[state.chooserPick];
    closeChooser();
    if (state.chooserMode === 'program') { chooseFile(item); } else { chooseFolder(item); }
  }

  $('#chooser-files').addEventListener('click', function (ev) {
    var row = ev.target.closest('.file-row');
    if (!row) { return; }
    $$('.file-row', dialog).forEach(function (r) { r.setAttribute('aria-pressed', 'false'); });
    row.setAttribute('aria-pressed', 'true');
    state.chooserPick = Number(row.getAttribute('data-index'));
    $('#chooser-accept').disabled = false;
  });
  $('#chooser-files').addEventListener('dblclick', function (ev) {
    if (ev.target.closest('.file-row')) { acceptChooser(); }
  });
  $('#chooser-accept').addEventListener('click', acceptChooser);

  // ---------------------------------------------------------------- attempt

  var attemptTimer = null;
  var attemptStart = 0;

  function runAttempt() {
    show('attempt');
    announce('Launch attempt in progress. This attempt cannot be cancelled from HELM.');
    attemptStart = Date.now();
    var bar = $('#bound-elapsed');
    var label = $('#bound-elapsed-label');
    bar.style.width = '0%';
    label.textContent = 'elapsed 0.0 s';

    window.clearInterval(attemptTimer);
    attemptTimer = window.setInterval(function () {
      var elapsed = Date.now() - attemptStart;
      if (elapsed >= SIMULATED_ATTEMPT_MS) {
        window.clearInterval(attemptTimer);
        finishAttempt();
        return;
      }
      bar.style.width = Math.min(100, (elapsed / RUN_BOUND_MS) * 100) + '%';
      label.textContent = 'elapsed ' + (elapsed / 1000).toFixed(1) + ' s';
    }, 90);
  }

  function finishAttempt() {
    state.attempted = true;
    state.authorised = false;
    renderLibrary();
    show('result');
    announce('Result: the program ended and reported 0. HELM did not establish that execution began.');
  }

  // ---------------------------------------------------------------- authority

  function setAuthorised(on) {
    state.authorised = on;
    $('#authorise-card').hidden = on;
    $('#launch-card').hidden = !on;
    var chip = $('#authority-chip');
    chip.innerHTML = on
      ? icon('i-disc') + ' Launch available'
      : icon('i-circle') + ' Under review';
    renderLibrary();
  }

  // ---------------------------------------------------------------- receipt

  function renderReceipt() {
    $('#receipt-bytes').textContent = RECEIPT_BYTES;
    var out = $('#receipt-digest');
    var src = $('#receipt-digest-src');

    var subtle = window.crypto && window.crypto.subtle;
    if (!subtle || typeof TextEncoder === 'undefined') {
      out.textContent = 'not computed in this browser context';
      src.textContent = 'A digest function was not available here. In the product the digest comes ' +
                        'from the receipt itself.';
      return;
    }
    subtle.digest('SHA-256', new TextEncoder().encode(RECEIPT_BYTES)).then(function (buf) {
      var hex = Array.prototype.map.call(new Uint8Array(buf), function (b) {
        return ('0' + b.toString(16)).slice(-2);
      }).join('');
      out.textContent = hex;
      src.textContent = 'Computed in this page from exactly the bytes above.';
    }).catch(function () {
      out.textContent = 'not computed in this browser context';
      src.textContent = 'A digest function was not available here.';
    });
  }

  function copyReceipt() {
    var confirmLine = $('#copy-confirm');
    var done = function (ok) {
      confirmLine.hidden = false;
      confirmLine.textContent = ok
        ? 'Copied ' + RECEIPT_BYTES.length + ' bytes, exactly as shown.'
        : 'Copy was not available here. Select the bytes above and copy them directly.';
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(RECEIPT_BYTES).then(function () { done(true); },
                                                        function () { done(false); });
    } else {
      done(false);
    }
  }

  // ---------------------------------------------------------------- actions

  var actions = {
    'choose-start': function () { resetChoose(); show('choose'); },
    'cancel-choose': function () {
      resetChoose();
      show('library');
      announce('Cancelled. HELM closed everything it opened and kept nothing.');
    },
    'open-chooser': function (btn) { openChooser(btn.getAttribute('data-mode')); },
    'chooser-cancel': closeChooser,
    'open-entry': function () {
      state.entry = true;
      state.authorised = false;
      state.attempted = false;
      renderLibrary();
      setAuthorised(false);
      show('program');
      announce('hello-cli is open in HELM for this session.');
    },
    'authorise': function () {
      setAuthorised(true);
      announce('Authorised for exactly one attempt. Launch available.');
    },
    'discard-authorisation': function () {
      setAuthorised(false);
      announce('Authorisation discarded. Nothing ran.');
    },
    'attempt-launch': runAttempt,
    'attempt-again': function () {
      setAuthorised(false);
      show('authority');
      announce('Every attempt is authorised again. Review the authority for this attempt.');
    },
    'close-program': function () {
      var ok = window.confirm(
        'Close hello-cli?\n\n' +
        'This discards the session entry for hello-cli, its authorisation and its result. ' +
        'It deletes nothing on disk: the program file and the working folder are untouched.'
      );
      if (!ok) { return; }
      state.entry = false;
      state.attempted = false;
      state.authorised = false;
      $('#program-open-result').hidden = true;
      $('#program-result-line').textContent = 'No launch has been attempted.';
      $('#entry-last').textContent = 'No launch attempted in this session';
      resetChoose();
      renderLibrary();
      setAuthorised(false);
      show('library');
      announce('hello-cli was closed. Nothing on disk was changed.');
    },
    'copy-receipt': copyReceipt
  };

  // ---------------------------------------------------------------- wiring

  document.addEventListener('click', function (ev) {
    var goto = ev.target.closest('[data-goto]');
    if (goto && !goto.disabled) {
      show(goto.getAttribute('data-goto'));
      return;
    }
    var act = ev.target.closest('[data-action]');
    if (act && !act.disabled) {
      var fn = actions[act.getAttribute('data-action')];
      if (fn) { fn(act); }
      return;
    }
    var disc = ev.target.closest('.disclose > button');
    if (disc) {
      var open = disc.getAttribute('aria-expanded') === 'true';
      disc.setAttribute('aria-expanded', open ? 'false' : 'true');
      var region = document.getElementById(disc.getAttribute('aria-controls'));
      region.hidden = open;
      if (!open) {
        region.setAttribute('tabindex', '-1');
        region.focus({ preventScroll: true });
      } else {
        disc.focus({ preventScroll: true });
      }
    }
  });

  document.addEventListener('keydown', function (ev) {
    if (ev.key === 'Escape' && dialog.open) { closeChooser(); }
  });

  // The theme switcher and its three exploratory colour directions were removed at
  // integration: the owner selected the direction at G2-D8. This page is pinned to
  // the accepted working palette in the stylesheet's :root.

  // ---------------------------------------------------------------- deep links

  function bootstrapFromHash() {
    var target = (window.location.hash || '').replace('#', '');
    if (!target) { return false; }

    if (['program', 'authority', 'authorised', 'result', 'evidence', 'attempt', 'library-entry'].indexOf(target) !== -1) {
      state.entry = true;
      state.programChosen = true;
      state.folderChosen = true;
    }
    if (target === 'result' || target === 'evidence') { state.attempted = true; }

    if (target === 'choose') {
      // show the step mid-flow: program admitted, folder still to choose
      setPhase('program', 'done', 'i-disc',
        'Regular file, 18 456 bytes, ELF type <span class="mono">et_dyn</span>, measurement stable.' +
        '<span class="mono path">/home/you/apps/hello-cli</span>');
      setPhase('folder', 'active', 'i-half', 'Not checked yet.');
      $('#btn-choose-folder').disabled = false;
      $('#disclose-admission').hidden = false;
      setOpenState();
    }

    renderLibrary();
    setAuthorised(target === 'authorised');

    if (target === 'attempt') {
      $('#bound-elapsed').style.width = (SIMULATED_ATTEMPT_MS / RUN_BOUND_MS) * 100 + '%';
      $('#bound-elapsed-label').textContent = 'elapsed 2.1 s';
    }

    show(target === 'library-entry' ? 'library' : (target === 'authorised' ? 'authority' : target));
    return true;
  }

  // ---------------------------------------------------------------- boot

  resetChoose();
  renderLibrary();
  setAuthorised(false);
  renderReceipt();
  if (!bootstrapFromHash()) { show('library'); }
})();

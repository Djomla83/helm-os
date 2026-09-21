/* HELM G2 — canonical interaction prototype, "Record" / graphite frame.
   Mock state transitions only. No backend, no network, no persistence, no process.
   Every value shown here is a fixed mock constant; nothing is read from this computer. */

(function () {
  'use strict';

  var PROGRAM_FILES = [
    { name: 'hello-cli', meta: 'regular file · 18 456 bytes', outcome: 'admitted' },
    {
      name: 'report-tool', meta: 'regular file · 2 210 304 bytes', outcome: 'refused',
      code: 'SET_ID_BITS_PRESENT',
      heading: 'HELM does not admit this file',
      body: 'This file carries set-user-ID or set-group-ID bits. HELM does not admit programs ' +
            'carrying these bits, because it cannot reason about what they change. This is not a ' +
            'statement that the program is dangerous.'
    },
    {
      name: 'notes.txt', meta: 'regular file · 1 284 bytes', outcome: 'refused',
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
      name: 'hello-cli', meta: 'regular file · 18 456 bytes', kind: 'file', outcome: 'refused',
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

  var RUN_BOUND_MS = 30000;
  var SIMULATED_ATTEMPT_MS = 2183;

  var state = {
    screen: 'home',
    programChosen: false,
    folderChosen: false,
    entry: false,
    authorised: false,
    attempted: false,
    chooserMode: 'program',
    chooserPick: null
  };

  var $ = function (s, r) { return (r || document).querySelector(s); };
  var $$ = function (s, r) { return Array.prototype.slice.call((r || document).querySelectorAll(s)); };

  var announcer = $('#announcer');
  function announce(t) {
    announcer.textContent = '';
    window.setTimeout(function () { announcer.textContent = t; }, 40);
  }

  // ------------------------------------------------ the append-only record

  var record = [];

  function append(key, value) {
    record.push({ key: key, value: value });
    renderRecord();
  }

  function renderRecord() {
    var list = $('#record-list');
    if (!record.length) {
      list.innerHTML = '<p class="record-empty">Nothing established yet.</p>';
      return;
    }
    list.innerHTML = record.map(function (r, i) {
      return '<div class="record-item">' +
        '<span class="i">' + String(i + 1).padStart(2, '0') + '</span>' +
        '<span><span class="k">' + r.key + '</span><span class="v">' + r.value + '</span></span>' +
        '</div>';
    }).join('');
    list.scrollTop = list.scrollHeight;
  }

  // ------------------------------------------------ routing

  var RAIL = { choose: 1, authority: 2, attempt: 3, result: 4, evidence: 5 };

  function railReachable(target) {
    if (target === 'choose') { return true; }
    if (target === 'attempt') { return false; }
    if (!state.entry) { return false; }
    if (target === 'result' || target === 'evidence') { return state.attempted; }
    return true;
  }

  function show(screen) {
    state.screen = screen;
    $$('.screen').forEach(function (el) {
      el.hidden = el.getAttribute('data-screen') !== screen;
    });

    var here = RAIL[screen] || 0;
    $$('.rail-step').forEach(function (btn) {
      var step = btn.getAttribute('data-step');
      var n = RAIL[step];
      btn.setAttribute('data-state', n === here ? 'current' : (n < here ? 'done' : 'pending'));
      btn.disabled = n !== here && !railReachable(step);
    });

    $('#work').scrollTop = 0;
  }

  function renderHome() {
    $('#home-open').hidden = !state.entry;
    $('#home-title').textContent = state.entry
      ? 'One program is open for this session.'
      : 'Nothing is open in HELM.';
    $('#home-standfirst').textContent = state.entry
      ? 'HELM holds the program and the folder you chose. It changed nothing on disk, and it keeps ' +
        'no library between runs.'
      : 'Choose a program file and a working folder on this computer. Choosing is not installing ' +
        'and not adding. HELM opens the file, measures it, and changes nothing on disk.';

    var glyph = '○', label = 'Known', cls = 'stamp';
    if (state.attempted) { glyph = '■'; label = 'Ended'; }
    else if (state.authorised) { glyph = '◉'; label = 'Launch available'; cls = 'stamp stamp--open'; }
    var stamp = $('#home-stamp');
    stamp.className = cls;
    stamp.innerHTML = '<span class="glyph">' + glyph + '</span> ' + label;

    $('#home-result-btn').hidden = !state.attempted;
    $('#home-last').textContent = state.attempted
      ? 'A moment ago, this session — ended, reported 0'
      : 'None in this session';
  }

  // ------------------------------------------------ choose

  function setOp(which, st, mark, html) {
    var el = $('#op-' + which);
    el.setAttribute('data-state', st);
    $('.mark', el).textContent = mark;
    $('#op-' + which + '-status').innerHTML = html;
  }

  function clearRefusal(w) { $('#' + w + '-refusal').innerHTML = ''; }

  function showRefusal(which, item) {
    var again = which === 'program' ? 'Choose another file' : 'Choose another folder';
    $('#' + which + '-refusal').innerHTML =
      '<div class="refusal">' +
        '<h4>' + item.heading + '</h4>' +
        '<p>' + item.body + '</p>' +
        '<span class="code">' + item.code + '</span>' +
        '<div class="acts" style="margin-top:14px">' +
          '<button type="button" class="btn btn--sm" data-action="open-chooser" data-mode="' +
            which + '">' + again + '</button>' +
          '<span class="small muted">Nothing was kept. HELM closed what it opened.</span>' +
        '</div>' +
      '</div>';
    append('Refused · ' + item.code, which === 'program' ? 'program file not admitted' : 'folder not admitted');
    announce(item.heading + '. ' + item.code + '. Choose another one to continue.');
  }

  function setOpenState() {
    var ready = state.programChosen && state.folderChosen;
    $('#btn-open-program').disabled = !ready;
    $('#open-hint').textContent = ready
      ? 'Nothing is installed, added or written.'
      : (!state.programChosen ? 'Waiting for a program file HELM can admit.'
                              : 'Waiting for a working folder HELM can admit.');
  }

  function resetChoose() {
    state.programChosen = false;
    state.folderChosen = false;
    clearRefusal('program');
    clearRefusal('folder');
    setOp('program', 'active', '▸', 'Nothing chosen yet.');
    setOp('folder', 'pending', '○', 'Not checked yet.');
    $('#btn-choose-folder').disabled = true;
    $('#btn-choose-program').textContent = 'Choose file';
    $('#btn-choose-folder').textContent = 'Choose folder';
    $('#disclose-admission').hidden = true;
    setOpenState();
  }

  function chooseFile(item) {
    clearRefusal('program');
    setOp('program', 'active', '▸', 'Checking the program…');
    window.setTimeout(function () {
      if (item.outcome === 'refused') {
        state.programChosen = false;
        setOp('program', 'active', '▸',
          'HELM opened <span class="mono">/home/you/apps/' + item.name + '</span> and did not admit it.');
        showRefusal('program', item);
        $('#btn-choose-folder').disabled = true;
        setOpenState();
        return;
      }
      state.programChosen = true;
      setOp('program', 'done', '■',
        'Regular file, 18 456 bytes, ELF type <span class="mono">et_dyn</span>, measurement stable.' +
        '<span class="path">/home/you/apps/' + item.name + '</span>');
      $('#btn-choose-program').textContent = 'Choose another';
      $('#btn-choose-folder').disabled = false;
      setOp('folder', 'active', '▸', 'Not checked yet.');
      $('#disclose-admission').hidden = false;
      setOpenState();
      append('Program admitted', 'hello-cli · 18 456 B · et_dyn · 9f2c41d7…');
      announce('Program accepted. Choose a working folder next.');
    }, 520);
  }

  function chooseFolder(item) {
    clearRefusal('folder');
    setOp('folder', 'active', '▸', 'Checking the folder…');
    window.setTimeout(function () {
      if (item.outcome === 'refused') {
        state.folderChosen = false;
        setOp('folder', 'active', '▸',
          'HELM opened <span class="mono">/home/you/' + item.name + '</span> and did not admit it.');
        showRefusal('folder', item);
        setOpenState();
        return;
      }
      state.folderChosen = true;
      setOp('folder', 'done', '■',
        'Directory, identifier <span class="mono">workdir-01</span>.' +
        '<span class="path">/home/you/' + item.name + '</span>');
      $('#btn-choose-folder').textContent = 'Choose another';
      setOpenState();
      append('Folder admitted', '/home/you/' + item.name + ' · workdir-01');
      announce('Working folder accepted.');
    }, 460);
  }

  // ------------------------------------------------ chooser dialog

  var dialog = $('#chooser');

  function openChooser(mode) {
    state.chooserMode = mode;
    state.chooserPick = null;
    var list = mode === 'program' ? PROGRAM_FILES : FOLDERS;
    $('#chooser-title').textContent = mode === 'program' ? 'Choose a program file' : 'Choose a working folder';
    $('#chooser-path').textContent = mode === 'program' ? '/home/you/apps' : '/home/you';
    $('#chooser-accept').disabled = true;
    $('#chooser-files').innerHTML = list.map(function (item, i) {
      var g = (mode === 'folder' && item.kind === 'dir') ? '▣' : '▢';
      return '<button type="button" class="file-row" data-index="' + i + '" aria-pressed="false">' +
        '<span class="g">' + g + '</span>' +
        '<span class="nm">' + item.name + '</span>' +
        '<span class="mt">' + item.meta + '</span></button>';
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

  // ------------------------------------------------ authority + attempt

  function setAuthorised(on) {
    state.authorised = on;
    $('#authorise-block').hidden = on;
    $('#launch-block').hidden = !on;
    renderHome();
    show(state.screen);
  }

  var timer = null;

  function runAttempt() {
    show('attempt');
    append('Attempt opened', 'run bound 30 s · no cancel available');
    announce('Launch attempt in progress. This attempt cannot be cancelled from HELM.');
    var start = Date.now();
    var fill = $('#scale-fill');
    var label = $('#scale-label');
    fill.style.width = '0%';
    label.textContent = 'elapsed 0.0 s';
    window.clearInterval(timer);
    timer = window.setInterval(function () {
      var e = Date.now() - start;
      if (e >= SIMULATED_ATTEMPT_MS) {
        window.clearInterval(timer);
        finish();
        return;
      }
      fill.style.width = Math.min(100, (e / RUN_BOUND_MS) * 100) + '%';
      label.textContent = 'elapsed ' + (e / 1000).toFixed(1) + ' s';
    }, 90);
  }

  function finish() {
    state.attempted = true;
    setAuthorised(false);
    append('Child ended', 'exited · code 0 · 2183 ms');
    append('Exec status', 'indeterminate · status_eof_without_record');
    append('Receipt produced', 'digest below · not signed, no proof of origin');
    show('result');
    announce('Result: the program ended and reported 0. HELM did not establish that execution began.');
  }

  // ------------------------------------------------ receipt

  function renderReceipt() {
    $('#receipt-bytes').textContent = RECEIPT_BYTES;
    var out = $('#receipt-digest');
    var src = $('#receipt-digest-src');
    var subtle = window.crypto && window.crypto.subtle;
    if (!subtle || typeof TextEncoder === 'undefined') {
      out.textContent = 'not computed in this browser context';
      src.textContent = 'A digest function was not available here.';
      return;
    }
    subtle.digest('SHA-256', new TextEncoder().encode(RECEIPT_BYTES)).then(function (buf) {
      out.textContent = Array.prototype.map.call(new Uint8Array(buf), function (b) {
        return ('0' + b.toString(16)).slice(-2);
      }).join('');
      src.textContent = 'Computed in this page from exactly the bytes above.';
    }).catch(function () {
      out.textContent = 'not computed in this browser context';
      src.textContent = 'A digest function was not available here.';
    });
  }

  function copyReceipt() {
    var line = $('#copy-confirm');
    var done = function (ok) {
      line.textContent = ok
        ? 'Copied ' + RECEIPT_BYTES.length + ' bytes, exactly as shown.'
        : 'Copy was not available here. Select the bytes above and copy them directly.';
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(RECEIPT_BYTES).then(function () { done(true); }, function () { done(false); });
    } else { done(false); }
  }

  // ------------------------------------------------ actions

  var actions = {
    'choose-start': function () { resetChoose(); show('choose'); },
    'cancel-choose': function () {
      resetChoose();
      show('home');
      announce('Cancelled. HELM closed everything it opened and kept nothing.');
    },
    'open-chooser': function (btn) { openChooser(btn.getAttribute('data-mode')); },
    'chooser-cancel': closeChooser,
    'open-entry': function () {
      state.entry = true;
      renderHome();
      setAuthorised(false);
      show('home');
      announce('hello-cli is open in HELM for this session.');
    },
    'authorise': function () {
      setAuthorised(true);
      append('Authorised', 'single use · plan 4b7e9a21…');
      announce('Authorised for exactly one attempt. Launch available.');
    },
    'discard-authorisation': function () {
      setAuthorised(false);
      append('Authorisation discarded', 'nothing ran');
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
        'This discards the session entry, its authorisation, its result and the session record. ' +
        'It deletes nothing on disk: the program file and the working folder are untouched.'
      );
      if (!ok) { return; }
      state.entry = false;
      state.attempted = false;
      record = [];
      renderRecord();
      resetChoose();
      setAuthorised(false);
      renderHome();
      show('home');
      announce('hello-cli was closed. Nothing on disk was changed.');
    },
    'copy-receipt': copyReceipt
  };

  document.addEventListener('click', function (ev) {
    var go = ev.target.closest('[data-goto]');
    if (go && !go.disabled) { show(go.getAttribute('data-goto')); return; }

    var act = ev.target.closest('[data-action]');
    if (act && !act.disabled) {
      var fn = actions[act.getAttribute('data-action')];
      if (fn) { fn(act); }
      return;
    }

    var step = ev.target.closest('.rail-step');
    if (step) {
      if (step.disabled) { return; }
      var target = step.getAttribute('data-step');
      if (target === 'choose') { actions['choose-start'](); return; }
      if (!railReachable(target)) { return; }
      show(target);
      return;
    }

    var disc = ev.target.closest('.disclose > button');
    if (disc) {
      var open = disc.getAttribute('aria-expanded') === 'true';
      disc.setAttribute('aria-expanded', open ? 'false' : 'true');
      $('.pm', disc).textContent = open ? '+' : '−';
      var region = document.getElementById(disc.getAttribute('aria-controls'));
      region.hidden = open;
      if (!open) { region.setAttribute('tabindex', '-1'); region.focus({ preventScroll: true }); }
      else { disc.focus({ preventScroll: true }); }
    }
  });

  document.addEventListener('keydown', function (ev) {
    if (ev.key === 'Escape' && dialog.open) { closeChooser(); }
  });

  // ------------------------------------------------ deep links (screens.html)

  // A hash selects one screen with the state it would have been reached in.
  // This exists so the contact sheet can show every screen; it is not a product
  // route and HELM has no URLs.
  var DEEP = ['home', 'session', 'choose', 'authority', 'authorised', 'attempt', 'result', 'evidence'];

  function seedRecord(target) {
    if (!state.entry) { return; }
    append('Program admitted', 'hello-cli · 18 456 B · et_dyn · 9f2c41d7…');
    append('Folder admitted', '/home/you/work · workdir-01');
    if (target === 'authorised' || state.attempted) {
      append('Authorised', 'single use · plan 4b7e9a21…');
    }
    if (state.attempted) {
      append('Attempt opened', 'run bound 30 s · no cancel available');
      append('Child ended', 'exited · code 0 · 2183 ms');
      append('Exec status', 'indeterminate · status_eof_without_record');
      append('Receipt produced', 'digest below · not signed, no proof of origin');
    }
  }

  function bootstrapFromHash() {
    var target = (window.location.hash || '').replace('#', '');
    if (DEEP.indexOf(target) === -1) { return false; }

    if (['session', 'authority', 'authorised', 'attempt', 'result', 'evidence'].indexOf(target) !== -1) {
      state.entry = true;
      state.programChosen = true;
      state.folderChosen = true;
    }
    if (target === 'result' || target === 'evidence') { state.attempted = true; }

    if (target === 'choose') {
      // mid-flow: the program was admitted, the working folder is still to choose
      state.programChosen = true;
      setOp('program', 'done', '■',
        'Regular file, 18 456 bytes, ELF type <span class="mono">et_dyn</span>, measurement stable.' +
        '<span class="path">/home/you/apps/hello-cli</span>');
      setOp('folder', 'active', '▸', 'Not checked yet.');
      $('#btn-choose-program').textContent = 'Choose another';
      $('#btn-choose-folder').disabled = false;
      $('#disclose-admission').hidden = false;
      setOpenState();
    }

    seedRecord(target);
    renderHome();
    setAuthorised(target === 'authorised');

    if (target === 'attempt') {
      $('#scale-fill').style.width = (SIMULATED_ATTEMPT_MS / RUN_BOUND_MS) * 100 + '%';
      $('#scale-label').textContent = 'elapsed 2.2 s';
    }

    show(target === 'session' ? 'home' : (target === 'authorised' ? 'authority' : target));
    return true;
  }

  // ------------------------------------------------ boot

  resetChoose();
  renderHome();
  renderRecord();
  setAuthorised(false);
  renderReceipt();
  if (!bootstrapFromHash()) { show('home'); }
})();

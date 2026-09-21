//! The seven accepted D8 surfaces, built natively.
//!
//! Translated from `docs/prototypes/g2-html/index.html`. Copy is reproduced
//! verbatim from the prototype, because the wording *is* the product decision:
//! `docs/implementation/HELM-G2-VISUAL-KICKOFF.md` governs what may be said,
//! and nothing here says more than the prototype already says.
//!
//! Each builder returns its root widget plus the handles that `refresh()` in
//! `main.rs` needs. Buttons report through a dispatcher rather than returning
//! handles, which keeps the wiring in one place.

use std::rc::Rc;

use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use crate::state::{self, Candidate, Screen, State};
use crate::widgets::{
    BODY_MEASURE, ITEM_GAP, LABEL_COL, ORDINAL_COL, ROW_GAP, column, fact, fact_advanced,
    field_body, grouphead, hbox, labelled_row, line, line_numeric, link_button, mark, numbered,
    numbered_field, primary_button, quiet_button, section_row, selectable_mono, vbox, wrapped,
};

/// Everything the spike lets a person do. All of it is presentation.
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Go(Screen),
    GoChoose,
    GoProgram,
    Pick(Candidate),
    ChooseFolder,
    Authorise,
    RefuseAuthority,
    StartAttempt,
    CloseEntry,
    ToggleOutput,
    ToggleDisclosure,
    CopyBytes,
}

pub type Dispatch = Rc<dyn Fn(Action)>;

fn on(button: &gtk::Button, dispatch: &Dispatch, action: Action) {
    let dispatch = Rc::clone(dispatch);
    button.connect_clicked(move |_| dispatch(action));
}

/// The outer padding of every screen: 38 top, 48 sides, 44 bottom.
fn screen_page() -> gtk::Box {
    let page = vbox(0);
    page.set_margin_top(38);
    page.set_margin_bottom(44);
    page.set_margin_start(48);
    page.set_margin_end(48);
    page
}

fn spaced<W: IsA<gtk::Widget>>(widget: W, top: i32) -> W {
    widget.as_ref().set_margin_top(top);
    widget
}

fn heading(text: &str, large: bool) -> gtk::Label {
    let label = wrapped(text, "helm-h1", 700);
    if large {
        label.add_css_class("helm-h1-lg");
    }
    label
}

// ---------------------------------------------------------------- library

pub struct LibraryUi {
    pub root: gtk::Widget,
    pub empty: gtk::Widget,
    pub entry: gtk::Widget,
    pub mark: gtk::Box,
    pub word: gtk::Label,
    pub note: gtk::Label,
}

pub fn library(dispatch: &Dispatch) -> LibraryUi {
    let page = screen_page();

    let head = hbox(ROW_GAP);
    let intro = vbox(10);
    intro.set_hexpand(true);
    intro.append(&heading("Library", true));
    intro.append(&wrapped(
        "What HELM currently has open, and the way to choose another program.",
        "helm-lede",
        540,
    ));
    let choose = primary_button("Choose local program");
    choose.set_valign(gtk::Align::Start);
    on(&choose, dispatch, Action::GoChoose);
    head.append(&intro);
    head.append(&choose);
    page.append(&head);

    // Nothing open.
    let empty = vbox(0);
    empty.add_css_class("rule-strong-top");
    empty.set_margin_top(34);
    let empty_row = labelled_row(
        "Nothing is open",
        &wrapped(
            "HELM knows nothing about any program yet. Choosing one opens it, measures it and \
             keeps it here for this session. Nothing is installed, nothing is added and nothing \
             is written to disk.",
            "helm-rowbody",
            560,
        ),
    );
    empty_row.set_margin_top(28);
    empty.append(&empty_row);
    page.append(&empty);

    // One session entry, as a ledger.
    let entry = vbox(0);

    let header = hbox(20);
    header.add_css_class("rule-strong-bottom");
    header.set_margin_top(34);
    header.set_margin_bottom(9);
    let h_program = line("Program", "helm-micro");
    h_program.set_hexpand(true);
    let h_state = column("State", "helm-micro", 520);
    let h_opened = line("Opened", "helm-micro");
    h_opened.set_size_request(84, -1);
    h_opened.set_xalign(1.0);
    h_opened.set_halign(gtk::Align::End);
    header.append(&h_program);
    header.append(&h_state);
    header.append(&h_opened);
    entry.append(&header);

    let row = hbox(20);
    row.add_css_class("rule-bottom");
    row.set_margin_top(20);
    row.set_margin_bottom(20);
    row.set_valign(gtk::Align::Start);

    let name_col = vbox(5);
    name_col.set_hexpand(true);
    name_col.append(&line(state::PROGRAM_NAME, "helm-entryname"));
    name_col.append(&line_numeric(state::PROGRAM_PATH, "helm-entrypath"));

    let state_col = vbox(6);
    state_col.set_size_request(520, -1);
    let state_line = hbox(9);
    let entry_mark = mark("mark-known", false);
    let word = line("Known", "helm-stateword");
    state_line.append(&entry_mark);
    state_line.append(&word);
    let note = wrapped("", "helm-statenote", 520);
    state_col.append(&state_line);
    state_col.append(&note);

    let opened = line_numeric(state::OPENED_AT, "helm-entrypath");
    opened.set_size_request(84, -1);
    opened.set_xalign(1.0);
    opened.set_halign(gtk::Align::End);
    opened.set_valign(gtk::Align::Start);
    opened.set_margin_top(3);

    row.append(&name_col);
    row.append(&state_col);
    row.append(&opened);
    entry.append(&row);

    let actions = hbox(22);
    actions.set_margin_top(18);
    let open = quiet_button("Open program", true, false);
    on(&open, dispatch, Action::GoProgram);
    let review = link_button("Review authority", false);
    on(&review, dispatch, Action::Go(Screen::Authority));
    let close = link_button("Close program", true);
    on(&close, dispatch, Action::CloseEntry);
    actions.append(&open);
    actions.append(&review);
    actions.append(&close);
    entry.append(&actions);

    page.append(&entry);

    // The standing non-claim.
    let not_installing = vbox(0);
    not_installing.add_css_class("rule-top");
    not_installing.set_margin_top(40);
    let label = column("Not installing", "helm-rowlabel-accent", LABEL_COL);
    let body = wrapped(
        "Choosing is not installing and not adding. HELM opens the file to inspect it, keeps the \
         choice for this session, and changes nothing on disk. HELM does not install anything \
         yet; doing so would need a source of record, an acquisition path and a replacement \
         transaction.",
        "helm-rowbody-sm",
        600,
    );
    let row = section_row(label.upcast_ref::<gtk::Widget>(), &body);
    row.set_margin_top(22);
    not_installing.append(&row);
    page.append(&not_installing);

    LibraryUi {
        root: page.upcast(),
        empty: empty.upcast(),
        entry: entry.upcast(),
        mark: entry_mark,
        word,
        note,
    }
}

// ----------------------------------------------------------------- choose

pub struct ChooseUi {
    pub root: gtk::Widget,
    pub program_phase: gtk::Label,
    pub folder_phase: gtk::Label,
    pub none: gtk::Widget,
    pub admitted: gtk::Widget,
    pub refused: gtk::Widget,
    pub refusal_title: gtk::Label,
    pub refusal_text: gtk::Label,
    pub refusal_code: gtk::Label,
    pub advanced: gtk::Widget,
    pub folder_admitted: gtk::Widget,
    pub folder_pending: gtk::Widget,
    pub continue_button: gtk::Button,
}

pub fn choose(dispatch: &Dispatch) -> ChooseUi {
    let page = screen_page();
    page.append(&line("Library · Choose", "helm-crumb"));

    let title = heading("Which program, and where should it run?", false);
    title.set_margin_top(12);
    page.append(&title);
    page.append(&spaced(
        wrapped(
            "HELM opens the file you choose and measures it. It does not run, copy, modify or \
             install anything at this step.",
            "helm-lede",
            600,
        ),
        10,
    ));

    // --- the program file -------------------------------------------------
    let program_section = vbox(0);
    program_section.add_css_class("rule-strong-top");
    program_section.set_margin_top(30);

    let program_label = vbox(4);
    program_label.set_size_request(LABEL_COL, -1);
    program_label.append(&column("The program file", "helm-rowlabel", LABEL_COL));
    let program_phase = line("Not checked yet", "helm-rowphase");
    program_label.append(&program_phase);

    let program_body = field_body();

    // Not chosen yet. The three candidates stand in for the desktop file
    // chooser: this spike opens no dialog and reads no filesystem. A later,
    // separately authorised backend-connected spike uses GtkFileDialog, which
    // routes through the desktop portal.
    let none = vbox(0);
    none.append(&wrapped(
        "Nothing is chosen. HELM knows nothing about this until a file is opened.",
        "helm-note",
        BODY_MEASURE,
    ));
    let standin = line(
        "Stand-in for the desktop file chooser — no dialog is opened and no filesystem is read.",
        "helm-micro",
    );
    standin.set_margin_top(14);
    none.append(&standin);
    let picks = vbox(0);
    picks.set_margin_top(10);
    for candidate in Candidate::all() {
        let pick = gtk::Button::new();
        pick.add_css_class("helm-chooser-item");
        pick.add_css_class("rule-soft-top");
        let inner = hbox(16);
        let name = line(candidate.name(), "helm-chooser-name");
        name.set_hexpand(true);
        let size = line_numeric(candidate.size(), "helm-chooser-size");
        inner.append(&name);
        inner.append(&size);
        pick.set_child(Some(&inner));
        pick.update_property(&[gtk::accessible::Property::Label(&format!(
            "Choose {}, {}",
            candidate.name(),
            candidate.size()
        ))]);
        on(&pick, dispatch, Action::Pick(candidate));
        picks.append(&pick);
    }
    picks.add_css_class("rule-soft-bottom");
    none.append(&picks);

    // Admitted.
    let admitted = vbox(0);
    admitted.append(&line(state::PROGRAM_NAME, "helm-fieldtitle"));
    admitted.append(&spaced(
        line_numeric(state::PROGRAM_PATH, "helm-entrypath"),
        5,
    ));
    let facts = vbox(0);
    facts.set_margin_top(16);
    facts.append(&fact("Kind", "Regular file"));
    facts.append(&fact("Size", state::SIZE_BYTES));
    facts.append(&fact("File mode bits", state::MODE_BITS));
    facts.append(&fact("ELF type", state::ELF_TYPE));
    admitted.append(&facts);
    admitted.append(&spaced(
        wrapped(
            "HELM opened and measured this file. It did not run, copy, modify or install \
             anything. This is a pre-execution measurement of the pinned object, never the \
             identity of bytes that executed.",
            "helm-micro",
            560,
        ),
        12,
    ));
    let choose_advanced = vbox(0);
    choose_advanced.set_margin_top(12);
    choose_advanced.append(&fact_advanced(
        "Measurement digest",
        "pre_exec_sha256",
        state::PRE_EXEC_DIGEST,
    ));
    admitted.append(&choose_advanced);

    // Refused.
    let refused = vbox(0);
    let refusal_head = hbox(9);
    let refusal_mark = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    refusal_mark.add_css_class("helm-mark-refused");
    refusal_mark.set_valign(gtk::Align::Center);
    refusal_mark.update_state(&[gtk::accessible::State::Hidden(true)]);
    let refusal_title = line("", "helm-refusal-title");
    refusal_head.append(&refusal_mark);
    refusal_head.append(&refusal_title);
    refused.append(&refusal_head);
    let refusal_text = wrapped("", "helm-refusal-text", 560);
    refusal_text.set_margin_top(10);
    refused.append(&refusal_text);
    let refusal_code = line_numeric("", "helm-refusal-code");
    refusal_code.set_margin_top(8);
    refused.append(&refusal_code);
    refused.append(&spaced(
        wrapped(
            "Nothing was admitted. HELM closed the descriptor it opened and kept nothing.",
            "helm-micro",
            560,
        ),
        10,
    ));

    program_body.append(&none);
    program_body.append(&admitted);
    program_body.append(&refused);

    let program_row = section_row(program_label.upcast_ref::<gtk::Widget>(), &program_body);
    program_row.set_margin_top(22);
    program_section.append(&program_row);
    page.append(&program_section);

    // --- the working folder ----------------------------------------------
    let folder_section = vbox(0);
    folder_section.add_css_class("rule-top");
    folder_section.set_margin_top(26);

    let folder_label = vbox(4);
    folder_label.set_size_request(LABEL_COL, -1);
    folder_label.append(&column("The working folder", "helm-rowlabel", LABEL_COL));
    let folder_phase = line("Not checked yet", "helm-rowphase");
    folder_label.append(&folder_phase);

    let folder_body = field_body();

    let folder_admitted = vbox(0);
    folder_admitted.append(&line_numeric(state::PROGRAM_PATH, "helm-fieldpath"));
    folder_admitted.append(&spaced(
        wrapped(
            &format!(
                "Directory, identifier {}. The program will start here.",
                state::WORKDIR_ID
            ),
            "helm-note",
            BODY_MEASURE,
        ),
        8,
    ));

    let folder_pending = vbox(0);
    folder_pending.append(&wrapped("Not chosen yet.", "helm-note", BODY_MEASURE));
    let pick_folder = quiet_button("Choose a working folder…", true, false);
    pick_folder.set_margin_top(14);
    pick_folder.set_halign(gtk::Align::Start);
    on(&pick_folder, dispatch, Action::ChooseFolder);
    folder_pending.append(&pick_folder);

    folder_body.append(&folder_admitted);
    folder_body.append(&folder_pending);

    let folder_row = section_row(folder_label.upcast_ref::<gtk::Widget>(), &folder_body);
    folder_row.set_margin_top(22);
    folder_section.append(&folder_row);
    page.append(&folder_section);

    // --- closing row ------------------------------------------------------
    let footer = hbox(ROW_GAP);
    footer.add_css_class("rule-strong-top");
    footer.set_margin_top(32);
    let note = wrapped(
        "Cancel closes every descriptor HELM opened and leaves nothing behind.",
        "helm-micro",
        460,
    );
    note.set_hexpand(true);
    note.set_margin_top(22);
    note.set_valign(gtk::Align::Center);
    let actions = hbox(22);
    actions.set_margin_top(22);
    actions.set_valign(gtk::Align::Center);
    let cancel = link_button("Cancel", true);
    on(&cancel, dispatch, Action::Go(Screen::Library));
    let continue_button = primary_button("Continue");
    on(&continue_button, dispatch, Action::GoProgram);
    actions.append(&cancel);
    actions.append(&continue_button);
    footer.append(&note);
    footer.append(&actions);
    page.append(&footer);

    ChooseUi {
        root: page.upcast(),
        program_phase,
        folder_phase,
        none: none.upcast(),
        admitted: admitted.upcast(),
        refused: refused.upcast(),
        refusal_title,
        refusal_text,
        refusal_code,
        advanced: choose_advanced.upcast(),
        folder_admitted: folder_admitted.upcast(),
        folder_pending: folder_pending.upcast(),
        continue_button,
    }
}

// ---------------------------------------------------------------- program

pub struct ProgramUi {
    pub root: gtk::Widget,
    pub mark: gtk::Box,
    pub word: gtk::Label,
    pub note: gtk::Label,
    pub authorise: gtk::Button,
    pub attempt: gtk::Button,
    pub available: gtk::Widget,
    pub has_result: gtk::Widget,
    pub no_result: gtk::Widget,
    pub advanced: gtk::Widget,
}

pub fn program(dispatch: &Dispatch) -> ProgramUi {
    let page = screen_page();
    page.append(&line("Library · Chosen program", "helm-crumb"));

    let head = hbox(32);
    head.add_css_class("helm-pagehead");
    head.set_margin_top(12);
    head.set_valign(gtk::Align::End);

    let head_left = vbox(12);
    head_left.set_hexpand(true);
    head_left.append(&heading(state::PROGRAM_NAME, false));
    let state_line = hbox(9);
    let program_mark = mark("mark-known", false);
    let word = line("Known", "helm-stateword-lg");
    let note = wrapped("", "helm-statedash", 420);
    state_line.append(&program_mark);
    state_line.append(&word);
    state_line.append(&note);
    head_left.append(&state_line);

    let head_actions = hbox(20);
    head_actions.set_valign(gtk::Align::End);
    let authorise = primary_button("Review what this will be permitted to use");
    on(&authorise, dispatch, Action::Go(Screen::Authority));
    let attempt = primary_button("Attempt launch");
    on(&attempt, dispatch, Action::StartAttempt);
    head_actions.append(&authorise);
    head_actions.append(&attempt);

    head.append(&head_left);
    head.append(&head_actions);
    page.append(&head);

    // Launch available.
    let available = vbox(0);
    available.add_css_class("rule-bottom");
    let available_row = section_row(
        column("Launch available", "helm-rowlabel-accent", LABEL_COL).upcast_ref::<gtk::Widget>(),
        &wrapped(
            "A single-use authorisation exists. It is consumed by one attempt; after that, \
             authorising happens again. Going back now discards it and requires choosing the file \
             and folder again.",
            "helm-rowbody-ink",
            600,
        ),
    );
    available_row.set_margin_top(22);
    available_row.set_margin_bottom(22);
    available.append(&available_row);
    page.append(&available);

    // Overview.
    let overview_body = field_body();
    let facts = vbox(0);
    facts.append(&fact("Program file", state::PROGRAM_PATH));
    facts.append(&fact("Working folder", state::PROGRAM_PATH));
    facts.append(&fact("Measured", state::MEASURED_SUMMARY));
    overview_body.append(&facts);
    overview_body.append(&spaced(
        wrapped(
            "HELM has no application identity, version or icon for this program. Those would come \
             from a specification HELM cannot read yet.",
            "helm-micro",
            BODY_MEASURE,
        ),
        10,
    ));
    let overview = labelled_row("Overview", &overview_body);
    overview.add_css_class("helm-sec-bottom");
    page.append(&overview);

    // Runtime.
    let runtime_body = field_body();
    runtime_body.append(&wrapped(
        "HELM runs this program directly on this computer.",
        "helm-note-ink",
        BODY_MEASURE,
    ));
    let runtime_advanced = vbox(0);
    runtime_advanced.set_margin_top(10);
    runtime_advanced.append(&fact_advanced(
        "Backend identity",
        "backend",
        state::BACKEND_IDENTITY,
    ));
    runtime_body.append(&runtime_advanced);
    runtime_body.append(&spaced(
        wrapped(
            "There is no runtime selection. Wine, Proton, a PWA runtime and a MicroVM do not exist \
             in HELM and are not offered here.",
            "helm-micro",
            BODY_MEASURE,
        ),
        10,
    ));
    let runtime = labelled_row("Runtime", &runtime_body);
    runtime.add_css_class("helm-sec-bottom");
    page.append(&runtime);

    // Result.
    let result_body = field_body();
    let has_result = vbox(0);
    has_result.append(&wrapped(
        "The program ended and reported 0. HELM does not interpret what that number means.",
        "helm-note-ink",
        BODY_MEASURE,
    ));
    let result_actions = hbox(20);
    result_actions.set_margin_top(12);
    let open_result = quiet_button("Open last result", true, false);
    on(&open_result, dispatch, Action::Go(Screen::Result));
    let details = link_button("Launch details", false);
    on(&details, dispatch, Action::Go(Screen::Evidence));
    result_actions.append(&open_result);
    result_actions.append(&details);
    has_result.append(&result_actions);
    let no_result = wrapped("No launch has been attempted.", "helm-note", BODY_MEASURE);
    result_body.append(&has_result);
    result_body.append(&no_result);
    let result = labelled_row("Result", &result_body);
    result.add_css_class("helm-sec-bottom");
    page.append(&result);

    // Updates.
    let updates = labelled_row(
        "Updates",
        &wrapped(
            "HELM cannot check for updates. It has no updater, no source to check against and no \
             record of what it previously had.",
            "helm-note",
            BODY_MEASURE,
        ),
    );
    updates.add_css_class("helm-sec-bottom");
    page.append(&updates);

    // Recovery. Only what HELM can actually do.
    let recovery_body = field_body();
    let chips = hbox(10);
    let review = quiet_button("Review authority", false, true);
    on(&review, dispatch, Action::Go(Screen::Authority));
    let rechoose = quiet_button("Choose a different program or folder", false, true);
    on(&rechoose, dispatch, Action::GoChoose);
    let close = quiet_button("Close program", false, true);
    on(&close, dispatch, Action::CloseEntry);
    chips.append(&review);
    chips.append(&rechoose);
    chips.append(&close);
    recovery_body.append(&chips);
    recovery_body.append(&spaced(
        wrapped(
            "Repair, rollback, reset and re-prepare are not shown as controls: nothing in HELM \
             prepares an environment, so there is nothing to repair or return to a known state. \
             Closing deletes nothing on disk.",
            "helm-micro",
            BODY_MEASURE,
        ),
        12,
    ));
    let recovery = labelled_row("Recovery", &recovery_body);
    recovery.set_margin_top(22);
    recovery.set_margin_bottom(22);
    page.append(&recovery);

    ProgramUi {
        root: page.upcast(),
        mark: program_mark,
        word,
        note,
        authorise,
        attempt,
        available: available.upcast(),
        has_result: has_result.upcast(),
        no_result: no_result.upcast(),
        advanced: runtime_advanced.upcast(),
    }
}

// -------------------------------------------------------------- authority

pub struct AuthorityUi {
    pub root: gtk::Widget,
    pub advanced: gtk::Widget,
}

pub fn authority(dispatch: &Dispatch) -> AuthorityUi {
    let page = screen_page();
    page.set_margin_bottom(40);

    let head = hbox(32);
    head.add_css_class("helm-pagehead");
    head.set_valign(gtk::Align::End);
    let title = wrapped(
        &format!("Authorise one launch of {}?", state::PROGRAM_NAME),
        "helm-h1",
        620,
    );
    title.set_hexpand(true);
    let meta = line("Authority review\nSingle use", "helm-pagemeta");
    meta.set_xalign(1.0);
    meta.set_halign(gtk::Align::End);
    meta.set_justify(gtk::Justification::Right);
    head.append(&title);
    head.append(&meta);
    page.append(&head);

    let permits = grouphead("What HELM will permit for this one attempt");
    permits.set_margin_top(20);
    permits.set_margin_bottom(8);
    page.append(&permits);

    let grants = vbox(0);
    grants.append(&numbered(
        "01",
        "The program",
        "HELM will run exactly this file — the one it inspected, not a file found by name later.",
        150,
        false,
        false,
    ));
    grants.append(&numbered(
        "02",
        "The folder",
        &format!("It will start in {}.", state::PROGRAM_PATH),
        150,
        false,
        false,
    ));
    grants.append(&numbered(
        "03",
        "The input",
        "It will receive no input. Anything it reads from input ends immediately.",
        150,
        false,
        false,
    ));
    grants.append(&numbered(
        "04",
        "The time",
        "It may run for at most 30 seconds. After that HELM asks it to stop, waits 5 seconds, \
         then forces it.",
        150,
        false,
        false,
    ));
    page.append(&grants);

    let refuses = grouphead("What HELM does not do");
    refuses.set_margin_top(24);
    refuses.set_margin_bottom(8);
    page.append(&refuses);

    let non_grants = vbox(0);
    non_grants.append(&numbered(
        "05",
        "No settings",
        "HELM passes no environment settings at all — not your home folder, not your display, not \
         your language.",
        150,
        false,
        false,
    ));
    non_grants.append(&numbered(
        "06",
        "No inheritance",
        "It inherits no open files or connections from HELM beyond its own input and output.",
        150,
        false,
        false,
    ));
    non_grants.append(&numbered(
        "07",
        "No new privileges",
        "It cannot gain additional privileges while starting.",
        150,
        false,
        false,
    ));
    let last = numbered(
        "08",
        "Everything else",
        "Beyond those, this program runs with the same access you have. HELM does not restrict \
         what it can read or change on this computer.",
        150,
        false,
        false,
    );
    last.add_css_class("rule-bottom");
    non_grants.append(&last);
    page.append(&non_grants);

    // The non-containment statement, in plain language.
    let pull = wrapped(
        "HELM does not contain this program. It starts it and watches it. It does not restrict \
         which files it can open, what it can change or what it can reach on the network.",
        "helm-pull",
        680,
    );
    pull.set_margin_start(ORDINAL_COL + ITEM_GAP);
    pull.set_margin_top(26);
    pull.set_margin_bottom(24);
    page.append(&pull);

    // Technical detail, one layer deep.
    let advanced = vbox(0);
    advanced.set_margin_start(ORDINAL_COL + ITEM_GAP);
    advanced.set_margin_bottom(24);
    let detail_head = line("Technical detail", "helm-grouphead");
    detail_head.set_margin_bottom(8);
    advanced.append(&detail_head);
    let grid = gtk::Grid::new();
    grid.set_column_spacing(44);
    grid.set_column_homogeneous(true);
    let details = [
        ("Environment settings", "environment_mode", "empty"),
        ("Input", "stdin_mode", "closed_pipe_eof"),
        ("Run deadline", "timeout_ms", "30000 ms"),
        ("Grace period", "grace_ms", "5000 ms"),
        ("Arguments", "argv_len", "1"),
        ("Plan identity", "plan_sha256", state::PLAN_DIGEST),
    ];
    let mut row = 0_i32;
    let mut col = 0_i32;
    for (label, field, value) in details {
        let cell = fact_advanced(label, field, value);
        cell.remove_css_class("rule-soft-top");
        cell.add_css_class("rule-soft-bottom");
        grid.attach(&cell, col, row, 1, 1);
        col += 1;
        if col == 2 {
            col = 0;
            row += 1;
        }
    }
    advanced.append(&grid);
    page.append(&advanced);

    let footer = hbox(ROW_GAP);
    footer.add_css_class("rule-strong-top");
    let note = wrapped(
        "An authorisation is single-use. Every attempt authorises again — nothing stands after \
         this one launch.",
        "helm-micro",
        430,
    );
    note.set_hexpand(true);
    note.set_margin_top(20);
    note.set_valign(gtk::Align::Center);
    let actions = hbox(22);
    actions.set_margin_top(20);
    actions.set_valign(gtk::Align::Center);
    let refuse = link_button("Do not authorise", true);
    on(&refuse, dispatch, Action::RefuseAuthority);
    let authorise = primary_button("Authorise this one launch");
    on(&authorise, dispatch, Action::Authorise);
    actions.append(&refuse);
    actions.append(&authorise);
    footer.append(&note);
    footer.append(&actions);
    page.append(&footer);

    AuthorityUi {
        root: page.upcast(),
        advanced: advanced.upcast(),
    }
}

// ---------------------------------------------------------------- attempt

pub struct AttemptUi {
    pub root: gtk::Widget,
    pub elapsed: gtk::Label,
    pub meter: gtk::ProgressBar,
}

/// The attempt screen carries **no controls at all**. `launch` is synchronous
/// and returns no handle, so there is nothing to cancel and nothing to observe
/// live. The screen says so instead of offering a button that would lie.
pub fn attempt() -> AttemptUi {
    let page = screen_page();
    page.append(&line(
        &format!("{} · Attempt", state::PROGRAM_NAME),
        "helm-crumb",
    ));

    let title_row = hbox(12);
    title_row.set_margin_top(14);
    let spinner_mark = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spinner_mark.add_css_class("helm-mark-attempt-lg");
    spinner_mark.set_valign(gtk::Align::Center);
    spinner_mark.update_state(&[gtk::accessible::State::Hidden(true)]);
    title_row.append(&spinner_mark);
    title_row.append(&heading("Launch attempt in progress", false));
    page.append(&title_row);

    page.append(&spaced(
        wrapped(
            "HELM called launch and the call has not returned. This is a fact about HELM's own \
             call. HELM has not established that the program is running.",
            "helm-lede-ink",
            640,
        ),
        12,
    ));

    let elapsed_body = field_body();
    let elapsed = line_numeric("0.0 s of at most 30 s", "helm-elapsed");
    elapsed_body.append(&elapsed);
    let meter = gtk::ProgressBar::new();
    meter.add_css_class("helm-meter");
    meter.set_margin_top(14);
    meter.set_fraction(0.0);
    // The bound is announced as text beside it; the bar is its picture.
    meter.update_state(&[gtk::accessible::State::Hidden(true)]);
    elapsed_body.append(&meter);
    elapsed_body.append(&spaced(
        wrapped(
            "This is a bound, not progress towards success. Reaching the deadline is a described \
             outcome, not a failure to finish in time.",
            "helm-micro",
            BODY_MEASURE,
        ),
        10,
    ));
    let elapsed_row = labelled_row("Elapsed against the bound", &elapsed_body);
    elapsed_row.add_css_class("helm-sec-top-strong");
    elapsed_row.set_margin_top(30);
    page.append(&elapsed_row);

    let steps_body = field_body();
    for (ordinal, text) in [
        ("01", "HELM asks the program to stop."),
        ("02", "It waits 5 seconds."),
        ("03", "It forces the program to stop."),
    ] {
        let step = hbox(ITEM_GAP);
        step.add_css_class("rule-soft-bottom");
        step.set_margin_top(10);
        step.set_margin_bottom(10);
        let n = line_numeric(ordinal, "helm-itemn");
        n.set_size_request(ORDINAL_COL, -1);
        step.append(&n);
        step.append(&wrapped(text, "helm-note-sm", BODY_MEASURE));
        steps_body.append(&step);
    }
    let steps = labelled_row("When the deadline expires", &steps_body);
    steps.add_css_class("helm-sec-top");
    steps.set_margin_top(24);
    page.append(&steps);

    let no_cancel = labelled_row(
        "No cancel",
        &wrapped(
            "HELM cannot stop an attempt it started. The launch call is synchronous and returns \
             no handle, so there is nothing to cancel and nothing to observe live. Making that \
             possible needs orchestration HELM does not have.",
            "helm-note-sm",
            BODY_MEASURE,
        ),
    );
    no_cancel.add_css_class("helm-sec-top");
    no_cancel.set_margin_top(24);
    page.append(&no_cancel);

    AttemptUi {
        root: page.upcast(),
        elapsed,
        meter,
    }
}

// ----------------------------------------------------------------- result

pub struct ResultUi {
    pub root: gtk::Widget,
    pub output_toggle: gtk::Button,
    pub output_panel: gtk::Widget,
}

pub fn result(dispatch: &Dispatch) -> ResultUi {
    let page = screen_page();
    page.append(&line(
        &format!("{} · Result", state::PROGRAM_NAME),
        "helm-crumb",
    ));

    let head = hbox(32);
    head.add_css_class("helm-pagehead");
    head.set_margin_top(12);
    head.set_valign(gtk::Align::End);

    let head_left = vbox(0);
    head_left.set_hexpand(true);
    let state_line = hbox(10);
    state_line.append(&mark("mark-ended", false));
    state_line.append(&line("Ended", "helm-stateword-lg"));
    head_left.append(&state_line);
    let title = wrapped("The program ended and reported 0.", "helm-h1", 660);
    title.set_margin_top(12);
    head_left.append(&title);
    head_left.append(&spaced(
        wrapped(
            "HELM does not interpret what that number means. The program is the authority on its \
             own exit codes.",
            "helm-lede",
            600,
        ),
        10,
    ));

    let head_actions = hbox(20);
    head_actions.set_valign(gtk::Align::End);
    let details = link_button("Launch details", false);
    on(&details, dispatch, Action::Go(Screen::Evidence));
    let again = primary_button("Attempt launch again");
    on(&again, dispatch, Action::Go(Screen::Authority));
    head_actions.append(&details);
    head_actions.append(&again);

    head.append(&head_left);
    head.append(&head_actions);
    page.append(&head);

    let facts_head = grouphead("The facts that qualify it");
    facts_head.set_margin_top(20);
    facts_head.set_margin_bottom(8);
    page.append(&facts_head);

    let facts = vbox(0);
    facts.append(&numbered(
        "01",
        "Start-up report",
        "HELM received no start-up failure report. It did not establish that the program began \
         running.",
        170,
        false,
        false,
    ));
    facts.append(&numbered(
        "02",
        "How it ended",
        "The direct child was observed to end by exit, reporting 0.",
        170,
        false,
        false,
    ));
    facts.append(&numbered(
        "03",
        "Run deadline",
        "The deadline did not expire. HELM issued no stop and no force.",
        170,
        false,
        false,
    ));
    facts.append(&numbered(
        "04",
        "Group cleanup",
        "Group cleanup was issued. That is not containment: a descendant that left the group \
         survives it.",
        170,
        false,
        false,
    ));
    let last = numbered(
        "05",
        "What was read",
        "Output and errors were drained to the end of file. Nothing was cut short.",
        170,
        false,
        false,
    );
    last.add_css_class("rule-bottom");
    facts.append(&last);
    page.append(&facts);

    // Program output.
    let output_body = field_body();
    let counts = vbox(0);
    counts.append(&fact("Output", state::STDOUT_BYTES));
    counts.append(&fact("Errors and messages", state::STDERR_BYTES));
    output_body.append(&counts);

    let output_actions = hbox(20);
    output_actions.set_margin_top(12);
    let output_toggle = quiet_button("Show what was printed", false, true);
    on(&output_toggle, dispatch, Action::ToggleOutput);
    output_actions.append(&output_toggle);
    output_actions.append(&line("Kept in memory for this session only", "helm-micro"));
    output_body.append(&output_actions);

    let output_panel = vbox(0);
    output_panel.add_css_class("rule-soft-top");
    output_panel.set_margin_top(12);
    let printed = selectable_mono(state::PROGRAM_OUTPUT, "helm-mono");
    printed.set_margin_top(12);
    output_panel.append(&printed);
    output_panel.append(&spaced(
        wrapped(
            "Output can contain anything the program printed, including paths. It is never \
             included in an export without a separate, explicit opt-in.",
            "helm-micro",
            BODY_MEASURE,
        ),
        10,
    ));
    output_body.append(&output_panel);

    let output_row = labelled_row("Program output", &output_body);
    output_row.add_css_class("helm-sec-top");
    output_row.set_margin_top(28);
    page.append(&output_row);

    // What is next.
    let next_body = field_body();
    let chips = hbox(10);
    let back = quiet_button("Back to the program", false, true);
    on(&back, dispatch, Action::GoProgram);
    let evidence = quiet_button("View evidence", false, true);
    on(&evidence, dispatch, Action::Go(Screen::Evidence));
    let different = quiet_button("Choose a different program", false, true);
    on(&different, dispatch, Action::GoChoose);
    chips.append(&back);
    chips.append(&evidence);
    chips.append(&different);
    next_body.append(&chips);
    let next = labelled_row("What is next", &next_body);
    next.add_css_class("helm-sec-top");
    next.set_margin_top(24);
    page.append(&next);

    ResultUi {
        root: page.upcast(),
        output_toggle,
        output_panel: output_panel.upcast(),
    }
}

// --------------------------------------------------------------- evidence

pub struct EvidenceUi {
    pub root: gtk::Widget,
    pub copy: gtk::Button,
}

pub fn evidence(dispatch: &Dispatch) -> EvidenceUi {
    let page = screen_page();
    page.append(&line(
        &format!("{} · Result · Launch details", state::PROGRAM_NAME),
        "helm-crumb",
    ));

    let head = hbox(32);
    head.add_css_class("helm-pagehead");
    head.set_margin_top(12);
    head.set_valign(gtk::Align::End);
    let title = heading("What exactly was recorded", false);
    title.set_hexpand(true);
    let copy = primary_button("Copy the exact receipt bytes");
    on(&copy, dispatch, Action::CopyBytes);
    head.append(&title);
    head.append(&copy);
    page.append(&head);

    // The receipt non-claim. No shield, no lock, no seal, no badge.
    let lede = wrapped(
        "This receipt is data. It records what HELM observed. It is not signed, carries no proof \
         of origin, and grants no permission to run anything. Anyone can write bytes that look \
         like it. Its digest identifies exactly these bytes and nothing else.",
        "helm-pull-plain",
        700,
    );
    lede.set_margin_start(ORDINAL_COL + ITEM_GAP);
    lede.set_margin_top(22);
    lede.set_margin_bottom(24);
    page.append(&lede);

    let facts_head = grouphead("Receipt facts");
    facts_head.set_margin_bottom(8);
    page.append(&facts_head);

    let facts = vbox(0);
    let rows = [
        (
            "01",
            "Exec status",
            "exec_status",
            "indeterminate · status_eof_without_record",
        ),
        ("02", "Child end", "child_end", "exited · code 0"),
        (
            "03",
            "Termination",
            "sigterm_sent · sigkill_sent · group_sweep",
            "false · false · issued",
        ),
        (
            "04",
            "Output stream",
            "stdout · drained · completeness",
            "1 284 bytes · complete_at_eof",
        ),
        (
            "05",
            "Error stream",
            "stderr · drained · completeness",
            "0 bytes · complete_at_eof",
        ),
        (
            "06",
            "Pre-execution measurement",
            "size · sha256 · mode_bits · elf_type",
            "41 984 · sha256:9d41…c7e0 · 0o755 · EXEC",
        ),
        ("07", "Plan identity", "plan_sha256", state::PLAN_DIGEST),
        ("08", "Backend identity", "backend", state::BACKEND_IDENTITY),
    ];
    let last_ordinal = "08";
    for (ordinal, key, field, value) in rows {
        let row = numbered_field(ordinal, key, field, value);
        if ordinal == last_ordinal {
            row.add_css_class("rule-bottom");
        }
        facts.append(&row);
    }
    page.append(&facts);

    // The digest of the exact bytes. Selectable, and carrying no badge.
    let digest_body = field_body();
    digest_body.append(&selectable_mono(state::RECEIPT_DIGEST, "helm-digest"));
    digest_body.append(&spaced(
        wrapped(
            "Reformatting a receipt changes its bytes and therefore its digest. Copying copies the \
             exact bytes, unreformatted.",
            "helm-micro",
            BODY_MEASURE,
        ),
        10,
    ));
    let digest = labelled_row("SHA-256 of the exact receipt bytes", &digest_body);
    digest.add_css_class("helm-sec-top");
    digest.set_margin_top(28);
    page.append(&digest);

    let bytes_body = field_body();
    bytes_body.append(&selectable_mono(state::RECEIPT_BYTES, "helm-bytes"));
    let bytes = labelled_row("Exact bytes", &bytes_body);
    bytes.add_css_class("helm-sec-top");
    bytes.set_margin_top(24);
    page.append(&bytes);

    let footer = hbox(22);
    footer.add_css_class("rule-strong-top");
    footer.set_margin_top(26);
    let back = quiet_button("Back to result", false, false);
    back.set_margin_top(20);
    on(&back, dispatch, Action::Go(Screen::Result));
    let note = line(
        "This screen is the whole disclosure; there is no further level.",
        "helm-micro",
    );
    note.set_margin_top(20);
    note.set_valign(gtk::Align::Center);
    footer.append(&back);
    footer.append(&note);
    page.append(&footer);

    EvidenceUi {
        root: page.upcast(),
        copy,
    }
}

/// Wraps a screen in the measured content column and a scroller.
pub fn clamped(child: &impl IsA<gtk::Widget>) -> gtk::Widget {
    let clamp = adw::Clamp::builder()
        .maximum_size(1096)
        .tightening_threshold(1096)
        .build();
    clamp.set_child(Some(child));
    clamp.upcast()
}

/// Hides `advanced` blocks in Normal disclosure. Kept next to the screens so
/// that adding a block and forgetting to gate it is visible in one file.
pub fn apply_disclosure(state: &State, blocks: &[&gtk::Widget]) {
    for block in blocks {
        block.set_visible(state.advanced());
    }
}

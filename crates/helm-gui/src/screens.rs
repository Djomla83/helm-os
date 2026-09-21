//! The seven accepted D8 surfaces, populated from **real** `helm-launch` facts.
//!
//! The visual structure is the accepted one and is not rebuilt around backend
//! internals: the frame, the rail, the ledger rows and the disclosure behaviour
//! are exactly as the accepted fidelity spike established them. What changed is
//! where the values come from.
//!
//! Screens whose content varies with real facts expose a container that
//! `main.rs` clears and refills. Screens that are fixed copy are built once.

use std::rc::Rc;

use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use crate::widgets::{
    BODY_MEASURE, ITEM_GAP, LABEL_COL, ORDINAL_COL, ROW_GAP, column, field_body, grouphead, hbox,
    labelled_row, line, line_numeric, link_button, mark, primary_button, quiet_button, section_row,
    vbox, wrapped,
};
use helm_gui::state::Screen;

/// Everything the interface lets a person do. Each one is a real operation.
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Go(Screen),
    GoChoose,
    GoProgram,
    /// Open the real GTK file chooser for the program.
    ChooseProgramFile,
    /// Open the real GTK folder chooser for the working directory.
    ChooseWorkingFolder,
    /// Compose the one-shot authority, at the person's explicit instruction.
    Authorise,
    RefuseAuthority,
    /// Consume that authority once, on a worker thread.
    StartAttempt,
    /// Return through fresh authority preparation.
    AttemptAgain,
    CloseEntry,
    ToggleOutput,
    ToggleDisclosure,
    CopyReceiptBytes,
}

pub type Dispatch = Rc<dyn Fn(Action)>;

pub fn on(button: &gtk::Button, dispatch: &Dispatch, action: Action) {
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

pub fn spaced<W: IsA<gtk::Widget>>(widget: W, top: i32) -> W {
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

/// Empties a container that is refilled from real facts on every render.
pub fn clear(container: &impl IsA<gtk::Widget>) {
    let container = container.as_ref();
    while let Some(child) = container.first_child() {
        child.unparent();
    }
}

// ---------------------------------------------------------------- library

pub struct LibraryUi {
    pub root: gtk::Widget,
    pub empty: gtk::Widget,
    pub entry: gtk::Widget,
    pub mark: gtk::Box,
    pub name: gtk::Label,
    pub path: gtk::Label,
    pub word: gtk::Label,
    pub note: gtk::Label,
    pub opened: gtk::Label,
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
    let name = line("", "helm-entryname");
    let path = line_numeric("", "helm-entrypath");
    path.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    path.set_max_width_chars(44);
    name_col.append(&name);
    name_col.append(&path);

    let state_col = vbox(6);
    state_col.set_size_request(520, -1);
    let state_line = hbox(9);
    let entry_mark = mark("mark-known", false);
    let word = line("", "helm-stateword");
    state_line.append(&entry_mark);
    state_line.append(&word);
    let note = wrapped("", "helm-statenote", 520);
    state_col.append(&state_line);
    state_col.append(&note);

    let opened = line_numeric("", "helm-entrypath");
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
        name,
        path,
        word,
        note,
        opened,
    }
}

// ----------------------------------------------------------------- choose

pub struct ChooseUi {
    pub root: gtk::Widget,
    pub program_phase: gtk::Label,
    pub program_body: gtk::Box,
    pub folder_phase: gtk::Label,
    pub folder_body: gtk::Box,
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

    let program_section = vbox(0);
    program_section.add_css_class("helm-sec-top-strong");
    program_section.set_margin_top(30);
    let program_label = vbox(4);
    program_label.set_size_request(LABEL_COL, -1);
    program_label.append(&column("The program file", "helm-rowlabel", LABEL_COL));
    let program_phase = line("Not checked yet", "helm-rowphase");
    program_label.append(&program_phase);
    let program_body = field_body();
    let program_row = section_row(program_label.upcast_ref::<gtk::Widget>(), &program_body);
    program_section.append(&program_row);
    page.append(&program_section);

    let folder_section = vbox(0);
    folder_section.add_css_class("helm-sec-top");
    folder_section.set_margin_top(26);
    let folder_label = vbox(4);
    folder_label.set_size_request(LABEL_COL, -1);
    folder_label.append(&column("The working folder", "helm-rowlabel", LABEL_COL));
    let folder_phase = line("Not checked yet", "helm-rowphase");
    folder_label.append(&folder_phase);
    let folder_body = field_body();
    let folder_row = section_row(folder_label.upcast_ref::<gtk::Widget>(), &folder_body);
    folder_section.append(&folder_row);
    page.append(&folder_section);

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
        program_body,
        folder_phase,
        folder_body,
        continue_button,
    }
}

/// The "nothing chosen yet" body, carrying the real chooser button.
pub fn choose_prompt(dispatch: &Dispatch, note: &str, label: &str, action: Action) -> gtk::Box {
    let body = vbox(0);
    body.append(&wrapped(note, "helm-note", BODY_MEASURE));
    let button = quiet_button(label, true, false);
    button.set_margin_top(14);
    button.set_halign(gtk::Align::Start);
    on(&button, dispatch, action);
    body.append(&button);
    body
}

/// A real refusal, explained without judging the object.
pub fn refusal_body(
    dispatch: &Dispatch,
    title: &str,
    text: &str,
    detail: Option<&str>,
    retry_label: &str,
    action: Action,
) -> gtk::Box {
    let body = vbox(0);
    let head = hbox(9);
    let refusal_mark = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    refusal_mark.add_css_class("helm-mark-refused");
    refusal_mark.set_valign(gtk::Align::Center);
    refusal_mark.update_state(&[gtk::accessible::State::Hidden(true)]);
    head.append(&refusal_mark);
    head.append(&line(title, "helm-refusal-title"));
    body.append(&head);
    body.append(&spaced(wrapped(text, "helm-refusal-text", 560), 10));
    if let Some(detail) = detail {
        body.append(&spaced(line_numeric(detail, "helm-refusal-code"), 8));
    }
    body.append(&spaced(
        wrapped(
            "Nothing was admitted. HELM closed the descriptor it opened and kept nothing.",
            "helm-micro",
            560,
        ),
        10,
    ));
    let retry = quiet_button(retry_label, true, false);
    retry.set_margin_top(14);
    retry.set_halign(gtk::Align::Start);
    on(&retry, dispatch, action);
    body.append(&retry);
    body
}

// ---------------------------------------------------------------- program

pub struct ProgramUi {
    pub root: gtk::Widget,
    pub title: gtk::Label,
    pub mark: gtk::Box,
    pub word: gtk::Label,
    pub note: gtk::Label,
    pub authorise: gtk::Button,
    pub attempt: gtk::Button,
    pub available: gtk::Widget,
    pub overview: gtk::Box,
    pub runtime_advanced: gtk::Box,
    pub result_body: gtk::Box,
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
    let title = heading("", false);
    head_left.append(&title);
    let state_line = hbox(9);
    let program_mark = mark("mark-known", false);
    let word = line("", "helm-stateword-lg");
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

    let overview_body = field_body();
    let overview = vbox(0);
    overview_body.append(&overview);
    overview_body.append(&spaced(
        wrapped(
            "HELM has no application identity, version or icon for this program. Those would come \
             from a specification HELM cannot read yet.",
            "helm-micro",
            BODY_MEASURE,
        ),
        10,
    ));
    let overview_row = labelled_row("Overview", &overview_body);
    overview_row.add_css_class("helm-sec-bottom");
    page.append(&overview_row);

    let runtime_body = field_body();
    runtime_body.append(&wrapped(
        "HELM runs this program directly on this computer.",
        "helm-note-ink",
        BODY_MEASURE,
    ));
    let runtime_advanced = vbox(0);
    runtime_advanced.set_margin_top(10);
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

    let result_body = field_body();
    let result_row = labelled_row("Result", &result_body);
    result_row.add_css_class("helm-sec-bottom");
    page.append(&result_row);

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
        title,
        mark: program_mark,
        word,
        note,
        authorise,
        attempt,
        available: available.upcast(),
        overview,
        runtime_advanced,
        result_body,
    }
}

// -------------------------------------------------------------- authority

pub struct AuthorityUi {
    pub root: gtk::Widget,
    pub title: gtk::Label,
    pub ledger: gtk::Box,
    pub advanced: gtk::Box,
    pub detail_grid: gtk::Box,
    pub authorise: gtk::Button,
}

pub fn authority(dispatch: &Dispatch) -> AuthorityUi {
    let page = screen_page();
    page.set_margin_bottom(40);

    let head = hbox(32);
    head.add_css_class("helm-pagehead");
    head.set_valign(gtk::Align::End);
    let title = wrapped("", "helm-h1", 620);
    title.set_hexpand(true);
    let meta = line("Authority review\nSingle use", "helm-pagemeta");
    meta.set_xalign(1.0);
    meta.set_halign(gtk::Align::End);
    meta.set_justify(gtk::Justification::Right);
    head.append(&title);
    head.append(&meta);
    page.append(&head);

    // Chosen, checked, permitted and not-done are all real, so the whole
    // ledger is rebuilt from facts on every render.
    let ledger = vbox(0);
    page.append(&ledger);

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

    let advanced = vbox(0);
    advanced.set_margin_start(ORDINAL_COL + ITEM_GAP);
    advanced.set_margin_bottom(24);
    let detail_head = line("Technical detail", "helm-grouphead");
    detail_head.set_margin_bottom(8);
    advanced.append(&detail_head);
    let detail_grid = vbox(0);
    advanced.append(&detail_grid);
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
        title,
        ledger,
        advanced,
        detail_grid,
        authorise,
    }
}

// ---------------------------------------------------------------- attempt

pub struct AttemptUi {
    pub root: gtk::Widget,
    pub crumb: gtk::Label,
    pub elapsed: gtk::Label,
    pub meter: gtk::ProgressBar,
    pub deadline_note: gtk::Label,
}

/// The attempt screen carries **no controls at all**. `launch` is synchronous
/// and returns no handle, so there is nothing to cancel and nothing to observe
/// live. Running it on a worker thread does not change that, and the screen
/// says so rather than offering a button that would lie.
pub fn attempt() -> AttemptUi {
    let page = screen_page();
    let crumb = line("", "helm-crumb");
    page.append(&crumb);

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
    let elapsed = line_numeric("", "helm-elapsed");
    elapsed_body.append(&elapsed);
    let meter = gtk::ProgressBar::new();
    meter.add_css_class("helm-meter");
    meter.set_margin_top(14);
    meter.set_fraction(0.0);
    meter.update_state(&[gtk::accessible::State::Hidden(true)]);
    elapsed_body.append(&meter);
    elapsed_body.append(&spaced(
        wrapped(
            "This is a bound, not progress towards success. Reaching the deadline is a described \
             outcome, not a failure to finish in time. The count is the interface's own clock and \
             is part of no receipt.",
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
    let deadline_note = wrapped("", "helm-note-sm", BODY_MEASURE);
    steps_body.append(&deadline_note);
    let steps = labelled_row("When the deadline expires", &steps_body);
    steps.add_css_class("helm-sec-top");
    steps.set_margin_top(24);
    page.append(&steps);

    let no_cancel = labelled_row(
        "No cancel",
        &wrapped(
            "HELM cannot stop an attempt it started. The launch call is synchronous and returns \
             no handle, so there is nothing to cancel and nothing to observe live. Running it off \
             the interface's thread keeps this window responsive; it creates no session, no stop \
             and no cancel.",
            "helm-note-sm",
            BODY_MEASURE,
        ),
    );
    no_cancel.add_css_class("helm-sec-top");
    no_cancel.set_margin_top(24);
    page.append(&no_cancel);

    AttemptUi {
        root: page.upcast(),
        crumb,
        elapsed,
        meter,
        deadline_note,
    }
}

// ----------------------------------------------------------------- result

pub struct ResultUi {
    pub root: gtk::Widget,
    pub crumb: gtk::Label,
    pub mark: gtk::Box,
    pub state_word: gtk::Label,
    pub headline: gtk::Label,
    pub subtitle: gtk::Label,
    pub details_link: gtk::Button,
    pub facts_head: gtk::Label,
    pub facts: gtk::Box,
    pub output_row: gtk::Widget,
    pub output_body: gtk::Box,
}

pub fn result(dispatch: &Dispatch) -> ResultUi {
    let page = screen_page();
    let crumb = line("", "helm-crumb");
    page.append(&crumb);

    let head = hbox(32);
    head.add_css_class("helm-pagehead");
    head.set_margin_top(12);
    head.set_valign(gtk::Align::End);

    let head_left = vbox(0);
    head_left.set_hexpand(true);
    let state_line = hbox(10);
    let result_mark = mark("mark-ended", false);
    let state_word = line("Ended", "helm-stateword-lg");
    state_line.append(&result_mark);
    state_line.append(&state_word);
    head_left.append(&state_line);
    let headline = wrapped("", "helm-h1", 660);
    headline.set_margin_top(12);
    head_left.append(&headline);
    let subtitle = spaced(wrapped("", "helm-lede", 600), 10);
    head_left.append(&subtitle);

    let head_actions = hbox(20);
    head_actions.set_valign(gtk::Align::End);
    let details_link = link_button("Launch details", false);
    on(&details_link, dispatch, Action::Go(Screen::Evidence));
    let again = primary_button("Attempt launch again");
    on(&again, dispatch, Action::AttemptAgain);
    head_actions.append(&details_link);
    head_actions.append(&again);
    head.append(&head_left);
    head.append(&head_actions);
    page.append(&head);

    let facts_head = grouphead("The facts that qualify it");
    facts_head.set_margin_top(20);
    facts_head.set_margin_bottom(8);
    page.append(&facts_head);
    let facts = vbox(0);
    page.append(&facts);

    let output_body = field_body();
    let output_row = labelled_row("Program output", &output_body);
    output_row.add_css_class("helm-sec-top");
    output_row.set_margin_top(28);
    page.append(&output_row);

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
        crumb,
        mark: result_mark,
        state_word,
        headline,
        subtitle,
        details_link,
        facts_head,
        facts,
        output_row: output_row.upcast(),
        output_body,
    }
}

// --------------------------------------------------------------- evidence

pub struct EvidenceUi {
    pub root: gtk::Widget,
    pub crumb: gtk::Label,
    pub copy: gtk::Button,
    pub lede: gtk::Label,
    pub body: gtk::Box,
}

pub fn evidence(dispatch: &Dispatch) -> EvidenceUi {
    let page = screen_page();
    let crumb = line("", "helm-crumb");
    page.append(&crumb);

    let head = hbox(32);
    head.add_css_class("helm-pagehead");
    head.set_margin_top(12);
    head.set_valign(gtk::Align::End);
    let title = heading("What exactly was recorded", false);
    title.set_hexpand(true);
    let copy = primary_button("Copy the exact receipt bytes");
    on(&copy, dispatch, Action::CopyReceiptBytes);
    head.append(&title);
    head.append(&copy);
    page.append(&head);

    let lede = wrapped("", "helm-pull-plain", 700);
    lede.set_margin_start(ORDINAL_COL + ITEM_GAP);
    lede.set_margin_top(22);
    lede.set_margin_bottom(24);
    page.append(&lede);

    let body = vbox(0);
    page.append(&body);

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
        crumb,
        copy,
        lede,
        body,
    }
}

/// Wraps a screen in the measured content column.
pub fn clamped(child: &impl IsA<gtk::Widget>) -> gtk::Widget {
    let clamp = adw::Clamp::builder()
        .maximum_size(1096)
        .tightening_threshold(1096)
        .build();
    clamp.set_child(Some(child));
    clamp.upcast()
}

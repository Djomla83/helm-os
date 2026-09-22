//! **EXPERIMENTAL — HELM G2, first real backend-connected vertical.**
//!
//! Not an accepted HELM product surface.
//!
//! This is the accepted G2 interface driving the accepted `helm-launch` 0.1
//! end to end: a real local file chosen through the desktop file dialog, real
//! executable and working-directory admission, a real parsed plan, real
//! authorisation, a real launch, and a real receipt.
//!
//! **There is no mock launch result anywhere in this binary.** Everything the
//! Result and Evidence surfaces show comes from a real `LaunchOutcome` or a
//! real `LaunchError`.
//!
//! What it still does not do: persist anything, install anything, manage a
//! session, cancel anything, contain anything, or support a graphical subject.
//! G-1, G-2 and G-3 are unauthorised and untouched.

mod screens;
mod theme;
mod widgets;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use helm_gui::session::Session;
use helm_gui::state::{self, Disclosure, Phase, Screen, StreamView, bytes_label};
use helm_gui::vertical::{self, Message, Refusal};
use helm_launch::{Backend, Stream};
use screens::{Action, Dispatch, clear, spaced};
use widgets::{
    BODY_MEASURE, fact, fact_advanced, hbox, line, line_numeric, numbered, numbered_field,
    quiet_button, selectable_mono, vbox, wrapped,
};

const APP_ID: &str = "dev.helm.G2Vertical";

/// How often the main loop drains the worker channel and advances the ephemeral
/// elapsed readout.
const TICK: Duration = Duration::from_millis(100);

// ---------------------------------------------------------------------------
// Ui
// ---------------------------------------------------------------------------

struct Ui {
    session: RefCell<Session>,
    updating: Cell<bool>,
    window: gtk::ApplicationWindow,
    tx: Sender<Message>,
    rx: RefCell<Receiver<Message>>,

    stack: gtk::Stack,
    nav: Vec<(Screen, gtk::ToggleButton)>,
    rail_label: gtk::Label,
    rail_note: gtk::Label,
    status_mark: gtk::Box,
    status_subject: gtk::Label,
    status_bound: gtk::Label,
    status_mode: gtk::Button,

    dispatch: RefCell<Option<Dispatch>>,
    library: screens::LibraryUi,
    choose: screens::ChooseUi,
    program: screens::ProgramUi,
    authority: screens::AuthorityUi,
    attempt: screens::AttemptUi,
    result: screens::ResultUi,
    evidence: screens::EvidenceUi,
}

impl Ui {
    fn dispatch(&self) -> Dispatch {
        self.dispatch
            .borrow()
            .clone()
            .unwrap_or_else(|| Rc::new(|_| {}))
    }
}

fn set_mark(shape: &gtk::Box, class: &str) {
    for existing in ["mark-known", "mark-available", "mark-attempt", "mark-ended"] {
        shape.remove_css_class(existing);
    }
    shape.add_css_class(class);
}

/// The subject the Attempt, Result and Evidence surfaces describe.
///
/// It is the attempt's **own** subject — the program that was measured and
/// authorised for that attempt — and never whatever happens to be open when
/// the result arrives. That is the whole of `PGR-01` at the drawing end.
fn attempt_subject(session: &Session) -> String {
    session.attempt().map_or_else(
        || session.program_name(),
        |attempt| attempt.subject().name.clone(),
    )
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

fn refresh(ui: &Rc<Ui>) {
    ui.updating.set(true);
    {
        let session = ui.session.borrow();
        let phase = session.phase();
        let dispatch = ui.dispatch();

        ui.stack.set_visible_child_name(session.screen.id());
        for (screen, button) in &ui.nav {
            button.set_active(*screen == session.screen);
        }

        ui.rail_label.set_text(if phase.is_open() {
            "Held in memory"
        } else {
            "Nothing open"
        });
        ui.rail_note.set_text(if phase.is_open() {
            "Entries live until HELM closes. Nothing was written to disk."
        } else {
            "HELM knows nothing about any program yet."
        });

        set_mark(&ui.status_mark, phase.mark().css_class());
        ui.status_subject.set_text(&if phase.is_open() {
            format!("{} — {}", session.program_name(), phase.state_word())
        } else {
            "No program open".to_owned()
        });
        ui.status_bound
            .set_text(&format!("Run deadline {} s", vertical::TIMEOUT_MS / 1000));
        ui.status_mode.set_label(if session.advanced() {
            "Advanced disclosure"
        } else {
            "Normal disclosure"
        });

        render_library(ui, &session, phase);
        render_choose(ui, &session, &dispatch);
        render_program(ui, &session, phase, &dispatch);
        render_authority(ui, &session);
        render_attempt(ui, &session);
        render_result(ui, &session, &dispatch);
        render_evidence(ui, &session);

        ui.program.runtime_advanced.set_visible(session.advanced());
        ui.authority.advanced.set_visible(session.advanced());
    }
    ui.updating.set(false);
}

fn render_library(ui: &Rc<Ui>, session: &Session, phase: Phase) {
    let open = phase.is_open() || session.exec_path.is_some();
    ui.library.empty.set_visible(!open);
    ui.library.entry.set_visible(open);
    if !open {
        return;
    }
    set_mark(&ui.library.mark, phase.mark().css_class());
    ui.library.name.set_text(&session.program_name());
    ui.library.path.set_text(
        &session
            .exec_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    );
    ui.library.word.set_text(phase.state_word());
    ui.library.note.set_text(phase.state_note());
    ui.library
        .opened
        .set_text(session.opened_at.as_deref().unwrap_or("—"));
}

fn render_choose(ui: &Rc<Ui>, session: &Session, dispatch: &Dispatch) {
    // --- the program file -------------------------------------------------
    ui.choose.program_phase.set_text(if session.exec_pending() {
        "Inspecting…"
    } else if session.has_executable() {
        "Admitted"
    } else if session.exec_refusal.is_some() {
        "Refused"
    } else {
        "Not checked yet"
    });

    clear(&ui.choose.program_body);
    if session.exec_pending() {
        ui.choose.program_body.append(&wrapped(
            "HELM is opening the file and measuring it. It is not running it.",
            "helm-note",
            BODY_MEASURE,
        ));
    } else if let Some(facts) = session.program.as_ref() {
        ui.choose
            .program_body
            .append(&line(&facts.name, "helm-fieldtitle"));
        let path = line_numeric(&facts.path, "helm-entrypath");
        path.set_margin_top(5);
        path.set_wrap(true);
        path.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        path.set_max_width_chars(72);
        ui.choose.program_body.append(&path);
        let facts_box = vbox(0);
        facts_box.set_margin_top(16);
        facts_box.append(&fact("Kind", "Regular file"));
        facts_box.append(&fact("Size", &bytes_label(facts.size)));
        facts_box.append(&fact(
            "File mode bits",
            &state::mode_bits_label(facts.mode_bits),
        ));
        facts_box.append(&fact("ELF type", &facts.elf_type_display));
        ui.choose.program_body.append(&facts_box);
        ui.choose.program_body.append(&spaced(
            wrapped(
                "HELM opened and measured this file. It did not run, copy, modify or install \
                 anything. This is a pre-execution measurement of the pinned object, never the \
                 identity of bytes that executed.",
                "helm-micro",
                560,
            ),
            12,
        ));
        let advanced = vbox(0);
        advanced.set_margin_top(12);
        advanced.append(&fact_advanced(
            "Measurement digest",
            "pre_exec_sha256",
            &state::short_digest(&facts.sha256_hex),
        ));
        if session.advanced() {
            ui.choose.program_body.append(&advanced);
        }
        let again = quiet_button("Choose a different file…", true, false);
        again.set_margin_top(14);
        again.set_halign(gtk::Align::Start);
        screens::on(&again, dispatch, Action::ChooseProgramFile);
        ui.choose.program_body.append(&again);
    } else if let Some(refusal) = session.exec_refusal.as_ref() {
        let body = refusal_widget(
            dispatch,
            refusal,
            "Choose a different file…",
            Action::ChooseProgramFile,
            session.advanced(),
        );
        ui.choose.program_body.append(&body);
    } else {
        ui.choose.program_body.append(&screens::choose_prompt(
            dispatch,
            "Nothing is chosen. HELM knows nothing about this until a file is opened.",
            "Choose a program file…",
            Action::ChooseProgramFile,
        ));
    }

    // --- the working folder ----------------------------------------------
    ui.choose
        .folder_phase
        .set_text(if session.workdir_pending() {
            "Checking…"
        } else if session.has_working_directory() {
            "Admitted"
        } else if session.workdir_refusal.is_some() {
            "Refused"
        } else {
            "Not checked yet"
        });

    clear(&ui.choose.folder_body);
    if session.workdir_pending() {
        ui.choose.folder_body.append(&wrapped(
            "HELM is opening the folder you chose.",
            "helm-note",
            BODY_MEASURE,
        ));
    } else if session.has_working_directory() {
        let path = line_numeric(
            &session
                .workdir_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            "helm-fieldpath",
        );
        path.set_wrap(true);
        path.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        path.set_max_width_chars(72);
        ui.choose.folder_body.append(&path);
        ui.choose.folder_body.append(&spaced(
            wrapped(
                &format!(
                    "Directory, identifier {}. The program will start here.",
                    vertical::WORKING_DIRECTORY_ID
                ),
                "helm-note",
                BODY_MEASURE,
            ),
            8,
        ));
        let again = quiet_button("Choose a different folder…", true, false);
        again.set_margin_top(14);
        again.set_halign(gtk::Align::Start);
        screens::on(&again, dispatch, Action::ChooseWorkingFolder);
        ui.choose.folder_body.append(&again);
    } else if let Some(refusal) = session.workdir_refusal.as_ref() {
        let body = refusal_widget(
            dispatch,
            refusal,
            "Choose a different folder…",
            Action::ChooseWorkingFolder,
            session.advanced(),
        );
        ui.choose.folder_body.append(&body);
    } else {
        ui.choose.folder_body.append(&screens::choose_prompt(
            dispatch,
            "Not chosen yet.",
            "Choose a working folder…",
            Action::ChooseWorkingFolder,
        ));
    }

    if let Some(detail) = session.plan_refusal.as_ref() {
        ui.choose.folder_body.append(&spaced(
            wrapped(
                &format!("HELM could not build a valid launch plan: {detail}"),
                "helm-refusal-text",
                560,
            ),
            12,
        ));
    }

    ui.choose.continue_button.set_visible(session.admitted());
}

/// A real refusal, whatever produced it, explained in the same shape.
fn refusal_widget(
    dispatch: &Dispatch,
    refusal: &Refusal,
    retry_label: &str,
    action: Action,
    advanced: bool,
) -> gtk::Box {
    match refusal {
        Refusal::NotLocal => screens::refusal_body(
            dispatch,
            "Refused — that is not a local file",
            "This G2 slice currently supports local files only. HELM did not open it, and it \
             does not fetch anything from elsewhere.",
            None,
            retry_label,
            action,
        ),
        Refusal::Open(error) => screens::refusal_body(
            dispatch,
            "Refused — HELM could not open it",
            "HELM asked the operating system to open the object you chose, for reading only, and \
             was not able to. Nothing was admitted.",
            advanced.then(|| error.to_string()).as_deref(),
            retry_label,
            action,
        ),
        Refusal::Admission(error) => screens::refusal_body(
            dispatch,
            state::admission_title(error.code()),
            state::admission_text(error.code()),
            advanced.then(|| state::admission_detail(error)).as_deref(),
            retry_label,
            action,
        ),
    }
}

fn render_program(ui: &Rc<Ui>, session: &Session, phase: Phase, dispatch: &Dispatch) {
    ui.program.title.set_text(&session.program_name());
    set_mark(&ui.program.mark, phase.mark().css_class());
    ui.program.word.set_text(phase.state_word());
    ui.program
        .note
        .set_text(&format!("— {}", phase.state_note()));
    ui.program.authorise.set_visible(phase.can_authorise());
    ui.program.attempt.set_visible(phase.can_attempt());
    ui.program.available.set_visible(phase.can_attempt());

    clear(&ui.program.overview);
    if let Some(facts) = session.program.as_ref() {
        ui.program
            .overview
            .append(&fact("Program file", &facts.path));
        ui.program.overview.append(&fact(
            "Working folder",
            &session
                .workdir_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
        ));
        ui.program
            .overview
            .append(&fact("Measured", &facts.summary()));
    } else {
        ui.program.overview.append(&wrapped(
            "Nothing is open. Choose a local program to see what HELM measured.",
            "helm-note",
            BODY_MEASURE,
        ));
    }

    clear(&ui.program.runtime_advanced);
    ui.program.runtime_advanced.append(&fact_advanced(
        "Backend identity",
        "backend",
        Backend::LinuxX8664Clone3PidfdExecveat.as_str(),
    ));

    clear(&ui.program.result_body);
    if let Some(outcome) = session.outcome() {
        let end = outcome.receipt().record().child_end();
        ui.program.result_body.append(&wrapped(
            &format!(
                "{} {}",
                state::child_end_headline(end),
                state::child_end_subtitle(end)
            ),
            "helm-note-ink",
            BODY_MEASURE,
        ));
        let actions = hbox(20);
        actions.set_margin_top(12);
        let open = quiet_button("Open last result", true, false);
        screens::on(&open, dispatch, Action::Go(Screen::Result));
        let details = widgets::link_button("Launch details", false);
        screens::on(&details, dispatch, Action::Go(Screen::Evidence));
        actions.append(&open);
        actions.append(&details);
        ui.program.result_body.append(&actions);
    } else if let Some(error) = session.launch_error() {
        ui.program.result_body.append(&wrapped(
            state::launch_error_normal(error.code()),
            "helm-note-ink",
            BODY_MEASURE,
        ));
        let actions = hbox(20);
        actions.set_margin_top(12);
        let open = quiet_button("Open last result", true, false);
        screens::on(&open, dispatch, Action::Go(Screen::Result));
        actions.append(&open);
        ui.program.result_body.append(&actions);
    } else {
        ui.program.result_body.append(&wrapped(
            "No launch has been attempted.",
            "helm-note",
            BODY_MEASURE,
        ));
    }
}

fn render_authority(ui: &Rc<Ui>, session: &Session) {
    ui.authority.title.set_text(&format!(
        "Authorise one launch of {}?",
        session.program_name()
    ));
    ui.authority
        .authorise
        .set_sensitive(session.admitted() && !session.has_authority());

    clear(&ui.authority.ledger);
    let Some(plan) = session.plan_facts.as_ref() else {
        ui.authority.ledger.append(&spaced(
            wrapped(
                "Nothing is admitted yet, so there is nothing to authorise. Choose a local \
                 program and a working folder first.",
                "helm-note",
                600,
            ),
            20,
        ));
        clear(&ui.authority.detail_grid);
        return;
    };
    let program = session.program.as_ref();

    // WHAT YOU CHOSE
    let chosen = widgets::grouphead("What you chose");
    chosen.set_margin_top(20);
    chosen.set_margin_bottom(8);
    ui.authority.ledger.append(&chosen);
    let group = vbox(0);
    group.append(&numbered(
        "01",
        "The program file",
        &program.map(|f| f.path.clone()).unwrap_or_default(),
        170,
        false,
        true,
    ));
    let last = numbered(
        "02",
        "The working folder",
        &session
            .workdir_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        170,
        false,
        true,
    );
    last.add_css_class("rule-bottom");
    group.append(&last);
    ui.authority.ledger.append(&group);

    // WHAT HELM CHECKED
    let checked = widgets::grouphead("What HELM checked");
    checked.set_margin_top(24);
    checked.set_margin_bottom(8);
    ui.authority.ledger.append(&checked);
    let group = vbox(0);
    group.append(&numbered(
        "03",
        "The program",
        &program.map_or_else(
            || "Nothing was measured.".to_owned(),
            |f| {
                format!(
                    "HELM opened this exact file and measured it: {}. The measurement is of the \
                     pinned object, never of bytes that executed.",
                    f.summary()
                )
            },
        ),
        170,
        false,
        false,
    ));
    let last = numbered(
        "04",
        "The folder",
        "HELM opened the folder you chose and admitted it as a directory. It resolves no name \
         later.",
        170,
        false,
        false,
    );
    last.add_css_class("rule-bottom");
    group.append(&last);
    ui.authority.ledger.append(&group);

    // WHAT THIS ATTEMPT WILL BE AUTHORISED TO USE
    let permits = widgets::grouphead("What this attempt will be authorised to use");
    permits.set_margin_top(24);
    permits.set_margin_bottom(8);
    ui.authority.ledger.append(&permits);
    let group = vbox(0);
    group.append(&numbered(
        "05",
        "The program",
        "HELM will run exactly this file — the one it inspected, not a file found by name later.",
        150,
        false,
        false,
    ));
    group.append(&numbered(
        "06",
        "The folder",
        &format!(
            "It will start in the folder admitted as {}.",
            plan.working_directory_id
        ),
        150,
        false,
        false,
    ));
    group.append(&numbered(
        "07",
        "The input",
        "It will receive no input. Anything it reads from input ends immediately.",
        150,
        false,
        false,
    ));
    let last = numbered("08", "The time", &plan.time_sentence(), 150, false, false);
    last.add_css_class("rule-bottom");
    group.append(&last);
    ui.authority.ledger.append(&group);

    // WHAT HELM DOES NOT DO
    let refuses = widgets::grouphead("What HELM does not do");
    refuses.set_margin_top(24);
    refuses.set_margin_bottom(8);
    ui.authority.ledger.append(&refuses);
    let group = vbox(0);
    group.append(&numbered(
        "09",
        "No settings",
        "HELM passes no environment settings at all — not your home folder, not your display, not \
         your language.",
        150,
        false,
        false,
    ));
    group.append(&numbered(
        "10",
        "No inheritance",
        "It inherits no open files or connections from HELM beyond its own input and output.",
        150,
        false,
        false,
    ));
    group.append(&numbered(
        "11",
        "No new privileges",
        "It cannot gain additional privileges while starting.",
        150,
        false,
        false,
    ));
    let last = numbered(
        "12",
        "Everything else",
        "Beyond those, this program runs with the same access you have. HELM does not restrict \
         what it can read or change on this computer.",
        150,
        false,
        false,
    );
    last.add_css_class("rule-bottom");
    group.append(&last);
    ui.authority.ledger.append(&group);

    // Technical detail, straight off the validated plan.
    clear(&ui.authority.detail_grid);
    let grid = gtk::Grid::new();
    grid.set_column_spacing(44);
    grid.set_column_homogeneous(true);
    let details = [
        (
            "Environment settings",
            "environment_mode",
            plan.environment_mode.to_owned(),
        ),
        ("Input", "stdin_mode", plan.stdin_mode.to_owned()),
        (
            "Run deadline",
            "timeout_ms",
            format!("{} ms", plan.timeout_ms),
        ),
        ("Grace period", "grace_ms", format!("{} ms", plan.grace_ms)),
        (
            "Termination signal",
            "termination.signal",
            plan.termination_signal.to_owned(),
        ),
        ("Arguments", "argv_len", plan.argument_count.to_string()),
        (
            "Capture per stream",
            "capture_prefix_bytes",
            bytes_label(u64::from(plan.capture_bytes)),
        ),
        (
            "Plan identity",
            "plan_sha256",
            state::short_digest(&plan.plan_sha256_hex),
        ),
    ];
    let mut row = 0_i32;
    let mut col = 0_i32;
    for (label, field, value) in details {
        let cell = fact_advanced(label, field, &value);
        cell.remove_css_class("rule-soft-top");
        cell.add_css_class("rule-soft-bottom");
        grid.attach(&cell, col, row, 1, 1);
        col += 1;
        if col == 2 {
            col = 0;
            row += 1;
        }
    }
    ui.authority.detail_grid.append(&grid);
}

fn render_attempt(ui: &Rc<Ui>, session: &Session) {
    ui.attempt
        .crumb
        .set_text(&format!("{} · Attempt", attempt_subject(session)));
    let bound = f64::from(vertical::TIMEOUT_MS) / 1000.0;
    let elapsed = session
        .attempt()
        .map_or(0.0, |attempt| attempt.started().elapsed().as_secs_f64())
        .min(bound);
    ui.attempt
        .elapsed
        .set_text(&format!("{elapsed:.1} s of at most {bound:.0} s"));
    ui.attempt.meter.set_fraction(elapsed / bound);
    // The attempt's own validated plan, not a plan parsed for something the
    // person selected afterwards.
    let plan = session
        .attempt()
        .map(helm_gui::session::Attempt::plan)
        .or(session.plan_facts.as_ref());
    if let Some(plan) = plan {
        ui.attempt.deadline_note.set_text(&format!(
            "HELM asks the program to stop, waits {} seconds, then forces it. Those are the \
             plan's own values: {} ms and {} ms.",
            plan.grace_ms / 1000,
            plan.timeout_ms,
            plan.grace_ms
        ));
    }
}

fn render_result(ui: &Rc<Ui>, session: &Session, dispatch: &Dispatch) {
    ui.result
        .crumb
        .set_text(&format!("{} · Result", attempt_subject(session)));
    clear(&ui.result.facts);
    clear(&ui.result.output_body);

    let Some(outcome) = session.outcome() else {
        // A pre-child refusal. No child was created and no receipt exists, so
        // this is shaped as a result but never dressed as one.
        set_mark(&ui.result.mark, state::Mark::FilledSquare.css_class());
        ui.result.state_word.set_text("Could not begin");
        ui.result.details_link.set_visible(false);
        ui.result.output_row.set_visible(false);
        ui.result.facts_head.set_text("What HELM can say");
        if let Some(error) = session.launch_error() {
            ui.result
                .headline
                .set_text(state::launch_error_normal(error.code()));
            ui.result.subtitle.set_text(
                "No direct child was created, so there is nothing to report about a program. \
                 This is a fact about HELM.",
            );
            let group = vbox(0);
            group.append(&numbered(
                "01",
                "What happened",
                "HELM refused before creating anything. No process was started and no receipt \
                 exists.",
                170,
                false,
                false,
            ));
            let last = numbered(
                "02",
                "Recorded detail",
                &state::launch_error_detail(error),
                170,
                false,
                true,
            );
            last.add_css_class("rule-bottom");
            group.append(&last);
            ui.result.facts.append(&group);
        } else {
            ui.result.headline.set_text("No launch has been attempted.");
            ui.result.subtitle.set_text("");
        }
        return;
    };

    let receipt = outcome.receipt();
    let record = receipt.record();
    let end = record.child_end();

    set_mark(&ui.result.mark, state::Mark::FilledSquare.css_class());
    ui.result.state_word.set_text("Ended");
    ui.result.details_link.set_visible(true);
    ui.result.output_row.set_visible(true);
    ui.result.facts_head.set_text("The facts that qualify it");
    ui.result.headline.set_text(&state::child_end_headline(end));
    ui.result.subtitle.set_text(state::child_end_subtitle(end));

    let group = vbox(0);
    group.append(&numbered(
        "01",
        "Start-up report",
        state::exec_status_normal(record.exec_status()),
        170,
        false,
        false,
    ));
    group.append(&numbered(
        "02",
        "How it ended",
        &state::child_end_fact(end),
        170,
        false,
        false,
    ));
    group.append(&numbered(
        "03",
        "Run deadline",
        &state::deadline_fact_of(record),
        170,
        false,
        false,
    ));
    group.append(&numbered(
        "04",
        "Group cleanup",
        state::group_sweep_fact(record.termination().group_sweep()),
        170,
        false,
        false,
    ));
    let stdout_view = StreamView::build(
        record,
        Stream::Stdout,
        outcome.stdout_prefix(),
        outcome.prefix_truncated(Stream::Stdout),
    );
    let stderr_view = StreamView::build(
        record,
        Stream::Stderr,
        outcome.stderr_prefix(),
        outcome.prefix_truncated(Stream::Stderr),
    );
    let last = numbered(
        "05",
        "What was read",
        stdout_view.completeness_fact,
        170,
        false,
        false,
    );
    last.add_css_class("rule-bottom");
    group.append(&last);
    ui.result.facts.append(&group);

    // --- captured output ---------------------------------------------------
    let counts = vbox(0);
    counts.append(&fact("Output", &stdout_view.label()));
    counts.append(&fact("Errors and messages", &stderr_view.label()));
    ui.result.output_body.append(&counts);

    let actions = hbox(20);
    actions.set_margin_top(12);
    let toggle = quiet_button(
        if session.output_open {
            "Hide what was printed"
        } else {
            "Show what was printed"
        },
        false,
        true,
    );
    screens::on(&toggle, dispatch, Action::ToggleOutput);
    actions.append(&toggle);
    actions.append(&line("Kept in memory for this session only", "helm-micro"));
    ui.result.output_body.append(&actions);

    if session.output_open {
        let panel = vbox(0);
        panel.add_css_class("rule-soft-top");
        panel.set_margin_top(12);
        for (name, view) in [
            ("Output", &stdout_view),
            ("Errors and messages", &stderr_view),
        ] {
            let heading = line(name, "helm-factlabel");
            heading.set_margin_top(12);
            panel.append(&heading);
            if let Some(notice) = view.text.notice() {
                panel.append(&spaced(
                    wrapped(notice, "helm-refusal-text", BODY_MEASURE),
                    6,
                ));
            }
            if let Some(notice) = view.truncation_notice() {
                panel.append(&spaced(wrapped(notice, "helm-micro", BODY_MEASURE), 6));
            }
            let text = match &view.text {
                state::OutputText::Empty => "(nothing was captured)".to_owned(),
                state::OutputText::Text(text) => text.clone(),
                state::OutputText::NotUtf8 { lossy } => lossy.clone(),
            };
            panel.append(&spaced(selectable_mono(&text, "helm-mono"), 6));
        }
        panel.append(&spaced(
            wrapped(
                "Output can contain anything the program printed, including paths. It is held in \
                 memory for this session only, it is written nowhere, and it is part of no \
                 receipt.",
                "helm-micro",
                BODY_MEASURE,
            ),
            10,
        ));
        ui.result.output_body.append(&panel);
    }
}

fn render_evidence(ui: &Rc<Ui>, session: &Session) {
    ui.evidence.crumb.set_text(&format!(
        "{} · Result · Launch details",
        attempt_subject(session)
    ));
    ui.evidence.copy.set_label(&session.copy_label);
    clear(&ui.evidence.body);

    let Some(outcome) = session.outcome() else {
        ui.evidence.copy.set_visible(false);
        ui.evidence.lede.set_text(
            "No receipt was produced, because no direct child attempt was created. A receipt \
             records what HELM observed of a child; there was none to observe.",
        );
        if session.launch_error().is_some() {
            ui.evidence.body.append(&wrapped(
                "HELM refused before creating anything. There is nothing further to disclose, \
                 and HELM does not invent a receipt to fill the space.",
                "helm-note",
                BODY_MEASURE,
            ));
        } else {
            ui.evidence.body.append(&wrapped(
                "No launch has been attempted in this session.",
                "helm-note",
                BODY_MEASURE,
            ));
        }
        return;
    };

    ui.evidence.copy.set_visible(true);
    ui.evidence.lede.set_text(
        "This receipt is data. It records what HELM observed. It is not signed, carries no proof \
         of origin, and grants no permission to run anything. Anyone can write bytes that look \
         like it. Its digest identifies exactly these bytes and nothing else.",
    );

    let receipt = outcome.receipt();
    let record = receipt.record();

    let head = widgets::grouphead("Receipt facts");
    head.set_margin_bottom(8);
    ui.evidence.body.append(&head);
    let rows = state::receipt_rows(record);
    let group = vbox(0);
    let count = rows.len();
    for (index, (key, field, value)) in rows.into_iter().enumerate() {
        let row = numbered_field(&format!("{:02}", index + 1), key, &field, &value);
        if index + 1 == count {
            row.add_css_class("rule-bottom");
        }
        group.append(&row);
    }
    ui.evidence.body.append(&group);

    let digest_body = widgets::field_body();
    digest_body.append(&selectable_mono(&receipt.sha256().to_hex(), "helm-digest"));
    digest_body.append(&spaced(
        wrapped(
            "Reformatting a receipt changes its bytes and therefore its digest. Copying copies \
             the exact bytes, unreformatted.",
            "helm-micro",
            BODY_MEASURE,
        ),
        10,
    ));
    let digest = widgets::labelled_row("SHA-256 of the exact receipt bytes", &digest_body);
    digest.add_css_class("helm-sec-top");
    digest.set_margin_top(28);
    ui.evidence.body.append(&digest);

    let bytes_body = widgets::field_body();
    let text = String::from_utf8_lossy(receipt.exact_bytes()).into_owned();
    bytes_body.append(&selectable_mono(&text, "helm-bytes"));
    let bytes = widgets::labelled_row("Exact bytes", &bytes_body);
    bytes.add_css_class("helm-sec-top");
    bytes.set_margin_top(24);
    ui.evidence.body.append(&bytes);
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

fn handle(ui: &Rc<Ui>, action: Action) {
    if ui.updating.get() {
        return;
    }
    // While an attempt is outstanding the open program cannot be replaced.
    // The session refuses it in any case; this only avoids opening a chooser
    // whose answer would have to be discarded. Correctness is the session's,
    // not this check's.
    if ui.session.borrow().attempt_running()
        && matches!(
            action,
            Action::ChooseProgramFile | Action::ChooseWorkingFolder | Action::GoChoose
        )
    {
        return;
    }
    match action {
        Action::ChooseProgramFile => {
            open_file_dialog(ui, false);
            return;
        }
        Action::ChooseWorkingFolder => {
            open_file_dialog(ui, true);
            return;
        }
        Action::CopyReceiptBytes => {
            copy_receipt_bytes(ui);
            return;
        }
        _ => {}
    }

    {
        let mut session = ui.session.borrow_mut();
        match action {
            Action::Go(screen) => session.screen = screen,
            Action::GoChoose => session.go_choose(),
            Action::GoProgram => session.screen = Screen::Program,
            Action::RefuseAuthority => session.screen = Screen::Program,
            Action::CloseEntry => session.close(),
            Action::Authorise => session.authorise(),
            Action::ToggleOutput => session.output_open = !session.output_open,
            Action::ToggleDisclosure => {
                session.disclosure = match session.disclosure {
                    Disclosure::Advanced => Disclosure::Normal,
                    Disclosure::Normal => Disclosure::Advanced,
                };
            }
            _ => {}
        }
    }

    match action {
        Action::StartAttempt => start_attempt(ui),
        Action::AttemptAgain => attempt_again(ui),
        _ => {}
    }

    refresh(ui);
}

/// The real desktop file dialog. GTK routes it through the portal where that
/// applies; HELM builds no file manager of its own.
fn open_file_dialog(ui: &Rc<Ui>, folder: bool) {
    let dialog = gtk::FileDialog::new();
    dialog.set_title(if folder {
        "Choose a working folder"
    } else {
        "Choose a local program"
    });
    dialog.set_modal(true);
    let weak = Rc::downgrade(ui);
    let window = ui.window.clone();

    let handle_result = move |file: Result<gio::File, glib::Error>| {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        let Ok(file) = file else {
            // Dismissing the dialog is not a refusal; nothing changes.
            return;
        };
        // A `GFile` with no local path is a remote or virtual location. This
        // slice opens local files only, and says so rather than pretending.
        let Some(path) = file.path() else {
            ui.session.borrow_mut().refuse_non_local(folder);
            refresh(&ui);
            return;
        };
        begin_admission(&ui, path, folder);
    };

    if folder {
        dialog.select_folder(Some(&window), gio::Cancellable::NONE, handle_result);
    } else {
        dialog.open(Some(&window), gio::Cancellable::NONE, handle_result);
    }
}

/// The moment a selection is recorded and its worker starts. The session
/// decides what the selection invalidates; this function only supplies the
/// wall-clock text the session cannot produce and starts the worker.
fn begin_admission(ui: &Rc<Ui>, path: PathBuf, folder: bool) {
    let started = {
        let mut session = ui.session.borrow_mut();
        if folder {
            session.begin_working_directory(path)
        } else {
            session.begin_executable(path, local_time_of_day())
        }
    };
    // `None` means the session refused the selection, which it does while an
    // attempt is outstanding. Nothing was recorded, so nothing is started.
    let Some((op, path)) = started else {
        return;
    };
    if folder {
        vertical::spawn_admit_working_directory(op, path, ui.tx.clone());
    } else {
        vertical::spawn_admit_executable(op, path, ui.tx.clone());
    }
    refresh(ui);
}

/// The "opened at" text in the Library row. Ephemeral, local, and written
/// nowhere: it exists so a person can tell two selections apart in one sitting.
fn local_time_of_day() -> Option<String> {
    glib::DateTime::now_local()
        .ok()
        .and_then(|now| now.format("%H:%M:%S").ok())
        .map(|text| text.to_string())
}

/// Consumes the authority exactly once, on a worker thread.
fn start_attempt(ui: &Rc<Ui>) {
    let Some((op, authorized)) = ui.session.borrow_mut().start_attempt() else {
        return;
    };
    vertical::spawn_launch(op, authorized, ui.tx.clone());
}

/// "Attempt launch again" returns through fresh authority preparation: the
/// previous authority was consumed and nothing about it is replayed. The paths
/// are re-opened and re-admitted from scratch.
fn attempt_again(ui: &Rc<Ui>) {
    let (executable, working_directory) = ui.session.borrow_mut().attempt_again();
    if let Some((op, path)) = executable {
        vertical::spawn_admit_executable(op, path, ui.tx.clone());
    }
    if let Some((op, path)) = working_directory {
        vertical::spawn_admit_working_directory(op, path, ui.tx.clone());
    }
}

/// Copies exactly the receipt's own bytes, unreformatted.
fn copy_receipt_bytes(ui: &Rc<Ui>) {
    let label = {
        let session = ui.session.borrow();
        let Some(outcome) = session.outcome() else {
            return;
        };
        let bytes = outcome.receipt().exact_bytes();
        match (gdk::Display::default(), core::str::from_utf8(bytes)) {
            (Some(display), Ok(text)) => {
                display.clipboard().set_text(text);
                format!("Copied {} exact bytes", bytes.len())
            }
            (None, _) => "Clipboard unavailable — select the bytes below".to_owned(),
            (_, Err(_)) => "These bytes are not text — select them below".to_owned(),
        }
    };
    ui.session.borrow_mut().copy_label = label;
    refresh(ui);

    let weak = Rc::downgrade(ui);
    glib::timeout_add_local_once(Duration::from_millis(2600), move || {
        if let Some(ui) = weak.upgrade() {
            ui.session.borrow_mut().copy_label = "Copy the exact receipt bytes".to_owned();
            refresh(&ui);
        }
    });
}

// ---------------------------------------------------------------------------
// The main-loop tick: drains worker results and advances the elapsed readout.
// ---------------------------------------------------------------------------

fn install_tick(ui: &Rc<Ui>) {
    let weak = Rc::downgrade(ui);
    glib::timeout_add_local(TICK, move || {
        let Some(ui) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        let mut changed = false;
        loop {
            let message = ui.rx.borrow().try_recv();
            let Ok(message) = message else {
                break;
            };
            ui.session.borrow_mut().apply(message);
            changed = true;
        }
        let attempting = ui.session.borrow().attempt_running();
        if changed || attempting {
            refresh(&ui);
        }
        glib::ControlFlow::Continue
    });
}

// ---------------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------------

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(theme::CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn titlebar() -> gtk::CenterBox {
    let bar = gtk::CenterBox::new();
    bar.add_css_class("helm-titlebar");
    let controls = gtk::WindowControls::new(gtk::PackType::Start);
    controls.set_decoration_layout(Some("close,minimize,maximize:"));
    bar.set_start_widget(Some(&controls));
    let title = line("Helm", "helm-titlename");
    title.set_halign(gtk::Align::Center);
    bar.set_center_widget(Some(&title));
    let filler = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    filler.set_size_request(66, -1);
    bar.set_end_widget(Some(&filler));
    bar
}

fn rail(
    dispatch: &Dispatch,
) -> (
    gtk::Box,
    Vec<(Screen, gtk::ToggleButton)>,
    gtk::Label,
    gtk::Label,
) {
    let rail = vbox(0);
    rail.add_css_class("helm-rail");
    rail.set_size_request(250, -1);
    rail.set_margin_top(6);
    rail.set_margin_bottom(22);

    let brand = hbox(8);
    brand.set_margin_start(26);
    brand.set_margin_end(26);
    brand.set_margin_bottom(30);
    brand.set_valign(gtk::Align::Baseline);
    brand.append(&line("Helm", "helm-wordmark"));
    brand.append(&line("g2", "helm-g2"));
    rail.append(&brand);

    let nav_box = vbox(2);
    let mut nav: Vec<(Screen, gtk::ToggleButton)> = Vec::new();
    let mut group: Option<gtk::ToggleButton> = None;
    for (screen, label) in [
        (Screen::Library, "Library"),
        (Screen::Program, "Chosen program"),
        (Screen::Authority, "Authority"),
        (Screen::Evidence, "Evidence"),
    ] {
        let button = gtk::ToggleButton::new();
        button.add_css_class("helm-navitem");
        let inner = hbox(11);
        let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        dot.add_css_class("helm-navdot");
        dot.set_valign(gtk::Align::Center);
        dot.update_state(&[gtk::accessible::State::Hidden(true)]);
        inner.append(&dot);
        inner.append(&line(label, "helm-navitem-label"));
        button.set_child(Some(&inner));
        if let Some(first) = &group {
            button.set_group(Some(first));
        } else {
            group = Some(button.clone());
        }
        let dispatch = Rc::clone(dispatch);
        button.connect_toggled(move |b| {
            if b.is_active() {
                dispatch(Action::Go(screen));
            }
        });
        nav.push((screen, button.clone()));
        nav_box.append(&button);
    }
    rail.append(&nav_box);

    let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    rail.append(&spacer);

    let foot = vbox(6);
    foot.set_margin_start(26);
    foot.set_margin_end(26);
    let rail_label = line("", "helm-raillabel");
    let rail_note = wrapped("", "helm-railnote", 198);
    foot.append(&rail_label);
    foot.append(&rail_note);
    rail.append(&foot);

    (rail, nav, rail_label, rail_note)
}

fn build(app: &adw::Application) -> Rc<Ui> {
    load_css();
    // Theme locking: HELM's palette is an accepted product decision, so a host
    // theme cannot repaint it. `G2-UI-A11Y-01` carries the open question of
    // system high-contrast preference; it is not solved here.
    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceLight);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .default_width(1440)
        .default_height(900)
        .title("Helm")
        .build();
    window.set_titlebar(Some(&titlebar()));

    let root = hbox(0);
    root.add_css_class("helm-root");
    let (tx, rx) = channel::<Message>();

    let ui: Rc<Ui> = Rc::new_cyclic(|weak: &Weak<Ui>| {
        let weak = weak.clone();
        let dispatch: Dispatch = Rc::new(move |action| {
            if let Some(ui) = weak.upgrade() {
                handle(&ui, action);
            }
        });

        let (rail_box, nav, rail_label, rail_note) = rail(&dispatch);
        root.append(&rail_box);

        let library = screens::library(&dispatch);
        let choose = screens::choose(&dispatch);
        let program = screens::program(&dispatch);
        let authority = screens::authority(&dispatch);
        let attempt = screens::attempt();
        let result = screens::result(&dispatch);
        let evidence = screens::evidence(&dispatch);

        let stack = gtk::Stack::new();
        for (screen, child) in [
            (Screen::Library, &library.root),
            (Screen::Choose, &choose.root),
            (Screen::Program, &program.root),
            (Screen::Authority, &authority.root),
            (Screen::Attempt, &attempt.root),
            (Screen::Result, &result.root),
            (Screen::Evidence, &evidence.root),
        ] {
            stack.add_named(&screens::clamped(child), Some(screen.id()));
        }

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();
        scroller.add_css_class("helm-scroll");
        scroller.set_child(Some(&stack));

        let statusbar = hbox(28);
        statusbar.add_css_class("helm-statusbar");
        let subject_box = hbox(9);
        let status_mark = widgets::mark("mark-known", true);
        // The subject line is the interface's status region, so a phase change
        // — an attempt beginning, a result arriving — is announced and not only
        // drawn. GTK has no `live` accessible property; `Status` is the role it
        // does have, and the role is construct-only.
        let status_subject = gtk::Label::builder()
            .accessible_role(gtk::AccessibleRole::Status)
            .label("")
            .xalign(0.0)
            .build();
        status_subject.add_css_class("helm-statustext");
        status_subject.set_halign(gtk::Align::Start);
        subject_box.append(&status_mark);
        subject_box.append(&status_subject);
        subject_box.set_hexpand(true);

        let facts = hbox(24);
        facts.set_halign(gtk::Align::End);
        let status_bound = line_numeric("", "helm-statusfact");
        let status_mode = gtk::Button::with_label("");
        status_mode.add_css_class("helm-modebtn");
        status_mode.set_has_frame(false);
        {
            let dispatch = Rc::clone(&dispatch);
            status_mode.connect_clicked(move |_| dispatch(Action::ToggleDisclosure));
        }
        facts.append(&status_bound);
        facts.append(&status_mode);
        facts.append(&line("Session memory", "helm-statusfact"));
        statusbar.append(&subject_box);
        statusbar.append(&facts);

        let sheet = vbox(0);
        sheet.add_css_class("helm-sheet");
        sheet.set_hexpand(true);
        sheet.set_margin_end(16);
        sheet.set_margin_bottom(16);
        sheet.set_overflow(gtk::Overflow::Hidden);
        sheet.append(&scroller);
        sheet.append(&statusbar);
        root.append(&sheet);

        Ui {
            session: RefCell::new(Session::new()),
            updating: Cell::new(false),
            window: window.clone(),
            tx,
            rx: RefCell::new(rx),
            stack,
            nav,
            rail_label,
            rail_note,
            status_mark,
            status_subject,
            status_bound,
            status_mode,
            dispatch: RefCell::new(Some(dispatch)),
            library,
            choose,
            program,
            authority,
            attempt,
            result,
            evidence,
        }
    });

    window.set_child(Some(&root));
    install_tick(&ui);
    refresh(&ui);
    window.present();
    ui
}

/// A developer and owner smoke-test affordance, **not** a demo mode.
///
/// `--g2-preselect <program> <folder>` supplies exactly the two values the
/// desktop file dialog would have returned, and nothing else. Everything after
/// it is the ordinary real path: the same caller-side open, the same accepted
/// admission, the same parsed plan, the same authorisation and the same launch.
/// **No mock value enters anywhere**, and there is no result to fake because
/// results only ever come from a real `LaunchOutcome`.
///
/// It exists because a file dialog cannot be driven from a script, and the
/// owner should be able to reach the Result and Evidence surfaces repeatably.
/// Without the flag the interface behaves exactly as it does for any person.
struct Preselect {
    program: PathBuf,
    folder: PathBuf,
}

fn preselect_from_args() -> Option<Preselect> {
    let args: Vec<String> = std::env::args().collect();
    let index = args.iter().position(|arg| arg == "--g2-preselect")?;
    let program = args.get(index + 1)?;
    let folder = args.get(index + 2)?;
    Some(Preselect {
        program: PathBuf::from(program),
        folder: PathBuf::from(folder),
    })
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    // The application holds the one strong reference to the interface model, so
    // it lives as long as the window; every button holds only a weak reference
    // back, so nothing forms a cycle.
    let model: Rc<RefCell<Option<Rc<Ui>>>> = Rc::new(RefCell::new(None));
    app.connect_command_line(move |app, _| {
        let ui = build(app);
        if let Some(preselect) = preselect_from_args() {
            // Exactly what the dialog would have handed back, and nothing more.
            begin_admission(&ui, preselect.program, false);
            begin_admission(&ui, preselect.folder, true);
            ui.session.borrow_mut().screen = Screen::Choose;
            refresh(&ui);
        }
        *model.borrow_mut() = Some(ui);
        glib::ExitCode::SUCCESS
    });
    app.run()
}

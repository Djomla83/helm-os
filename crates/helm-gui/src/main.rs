//! **EXPERIMENTAL — G2-D9 UI FIDELITY SPIKE. NOT A HELM PRODUCT SURFACE.**
//!
//! This binary exists to answer exactly one question:
//!
//! > Can GTK 4 + gtk-rs with selective libadwaita faithfully reproduce the
//! > accepted HELM G2-D8 application?
//!
//! It is presentation only. It connects no backend, links no HELM crate, reads
//! no file, creates no process, writes nothing to disk and remembers nothing
//! between runs. Every value on screen is a constant in `state.rs`.
//!
//! The visual source of truth is `docs/prototypes/g2-html/index.html`. The
//! product-semantic authority is
//! `docs/implementation/HELM-G2-VISUAL-KICKOFF.md`. Where this spike and
//! either of those disagree, the spike is wrong.

mod screens;
mod state;
mod theme;
mod widgets;

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use screens::{Action, Dispatch};
use state::{Screen, State};
use widgets::{hbox, line, line_numeric, vbox};

const APP_ID: &str = "dev.helm.G2UiSpike";

/// Every widget `refresh` writes to. A flat struct on purpose: the spike has
/// one state and one render pass, exactly like the canonical prototype.
struct Ui {
    state: RefCell<State>,
    updating: Cell<bool>,

    stack: gtk::Stack,
    nav: Vec<(Screen, gtk::ToggleButton)>,

    rail_label: gtk::Label,
    rail_note: gtk::Label,

    status_mark: gtk::Box,
    status_subject: gtk::Label,
    status_bound: gtk::Label,
    status_mode: gtk::Button,

    library: screens::LibraryUi,
    choose: screens::ChooseUi,
    program: screens::ProgramUi,
    authority: screens::AuthorityUi,
    attempt: screens::AttemptUi,
    result: screens::ResultUi,
    evidence: screens::EvidenceUi,
}

impl Ui {
    /// The single render pass. Mirrors `render()` in the canonical prototype.
    fn refresh(&self) {
        let state = *self.state.borrow();
        self.updating.set(true);

        self.stack.set_visible_child_name(state.screen.id());
        for (screen, button) in &self.nav {
            button.set_active(*screen == state.screen);
        }

        self.rail_label.set_text(state.rail_label());
        self.rail_note.set_text(state.rail_note());

        set_mark(&self.status_mark, state.mark().css_class());
        self.status_subject.set_text(&state.footer_subject());
        self.status_bound.set_text(&state.footer_bound());
        self.status_mode.set_label(state.footer_mode());

        // Library
        let open = !matches!(state.entry, state::Entry::None);
        self.library.empty.set_visible(!open);
        self.library.entry.set_visible(open);
        set_mark(&self.library.mark, state.mark().css_class());
        self.library.word.set_text(state.state_word());
        self.library.note.set_text(state.state_note());

        // Choose
        self.choose.program_phase.set_text(state.program_phase());
        self.choose.folder_phase.set_text(state.folder_phase());
        self.choose
            .none
            .set_visible(matches!(state.program, state::Program::None));
        self.choose
            .admitted
            .set_visible(matches!(state.program, state::Program::Admitted));
        if let state::Program::Refused(refusal) = state.program {
            self.choose.refused.set_visible(true);
            self.choose.refusal_title.set_text(refusal.title());
            self.choose.refusal_text.set_text(refusal.text());
            self.choose.refusal_code.set_text(refusal.code());
        } else {
            self.choose.refused.set_visible(false);
        }
        self.choose.folder_admitted.set_visible(state.chosen());
        self.choose.folder_pending.set_visible(!state.chosen());
        self.choose.continue_button.set_visible(state.chosen());

        // Chosen program
        set_mark(&self.program.mark, state.mark().css_class());
        self.program.word.set_text(state.state_word());
        self.program
            .note
            .set_text(&format!("— {}", state.state_note()));
        self.program.authorise.set_visible(state.can_authorise());
        self.program.attempt.set_visible(state.can_attempt());
        self.program.available.set_visible(state.can_attempt());
        self.program.has_result.set_visible(state.has_result());
        self.program.no_result.set_visible(!state.has_result());

        // Attempt
        self.attempt.elapsed.set_text(&state.elapsed_label());
        self.attempt.meter.set_fraction(state.elapsed_fraction());

        // Result
        self.result
            .output_toggle
            .set_label(state.output_toggle_label());
        self.result.output_panel.set_visible(state.output_open);

        // Progressive technical disclosure, one layer deep per section.
        screens::apply_disclosure(
            &state,
            &[
                &self.choose.advanced,
                &self.program.advanced,
                &self.authority.advanced,
            ],
        );

        self.updating.set(false);
    }
}

/// Swaps the state-mark class, keeping the base classes.
fn set_mark(shape: &gtk::Box, class: &str) {
    for existing in ["mark-known", "mark-available", "mark-attempt", "mark-ended"] {
        shape.remove_css_class(existing);
    }
    shape.add_css_class(class);
}

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

/// The 36 px graphite bar. Real window controls rather than the prototype's
/// drawn dots: the dots were the designer's stand-in for them.
fn titlebar() -> gtk::CenterBox {
    let bar = gtk::CenterBox::new();
    bar.add_css_class("helm-titlebar");
    let controls = gtk::WindowControls::new(gtk::PackType::Start);
    // Without this the default layout keeps every control on the end side and
    // a Start-packed WindowControls draws nothing at all.
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
    let rail_note = widgets::wrapped("", "helm-railnote", 198);
    foot.append(&rail_label);
    foot.append(&rail_note);
    rail.append(&foot);

    (rail, nav, rail_label, rail_note)
}

fn build(app: &adw::Application) -> Rc<Ui> {
    load_css();

    // Theme locking. HELM's palette is an accepted product decision, so the
    // spike pins the colour scheme rather than letting a host theme repaint
    // it. This is the selective-libadwaita benefit in one line.
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

    // `Rc::new_cyclic` lets the buttons hold a dispatcher that holds only a
    // weak reference back to the UI, so the tree is dropped when the window is.
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

        // Status bar.
        let statusbar = hbox(28);
        statusbar.add_css_class("helm-statusbar");
        let subject_box = hbox(9);
        let status_mark = widgets::mark("mark-known", true);
        let status_subject = line("", "helm-statustext");
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
            state: RefCell::new(State::default()),
            updating: Cell::new(false),
            stack,
            nav,
            rail_label,
            rail_note,
            status_mark,
            status_subject,
            status_bound,
            status_mode,
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
    ui.refresh();
    window.present();
    ui
}

/// Applies one action and re-renders. All of it is presentation state.
fn handle(ui: &Rc<Ui>, action: Action) {
    if ui.updating.get() {
        return;
    }
    {
        let mut state = ui.state.borrow_mut();
        match action {
            Action::Go(screen) => state.go(screen),
            Action::GoChoose => state.go_choose(),
            Action::GoProgram => state.go_program(),
            Action::Pick(candidate) => state.pick(candidate),
            Action::ChooseFolder => state.choose_folder(),
            Action::Authorise => state.authorise(),
            Action::RefuseAuthority => state.refuse_authority(),
            Action::CloseEntry => state.close_entry(),
            Action::ToggleOutput => state.toggle_output(),
            Action::ToggleDisclosure => state.toggle_disclosure(),
            Action::StartAttempt => state.start_attempt(),
            Action::CopyBytes => {}
        }
    }

    match action {
        // A mock attempt on a timer. There is deliberately no counterpart that
        // stops it: HELM has no cancel, so the spike offers none.
        Action::StartAttempt => start_mock_attempt(ui),
        Action::CopyBytes => copy_receipt_bytes(ui),
        _ => {}
    }

    ui.refresh();
}

fn start_mock_attempt(ui: &Rc<Ui>) {
    let weak = Rc::downgrade(ui);
    let started = Instant::now();
    glib::timeout_add_local(Duration::from_millis(100), move || {
        let Some(ui) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        let elapsed = started.elapsed();
        {
            let mut state = ui.state.borrow_mut();
            if !matches!(state.entry, state::Entry::Attempt) {
                return glib::ControlFlow::Break;
            }
            #[allow(clippy::cast_precision_loss)]
            let seconds = elapsed.as_millis() as f64 / 1000.0;
            state.elapsed_seconds = seconds;
            if elapsed >= Duration::from_millis(state::MOCK_ATTEMPT_MS) {
                state.finish_attempt();
                drop(state);
                ui.refresh();
                return glib::ControlFlow::Break;
            }
        }
        ui.refresh();
        glib::ControlFlow::Continue
    });
}

/// Copies the exact mock receipt bytes. The button says what happened; where
/// the clipboard is unavailable it says that instead of reporting a copy that
/// did not occur.
fn copy_receipt_bytes(ui: &Rc<Ui>) {
    let button = ui.evidence.copy.clone();
    let label = match gdk::Display::default() {
        Some(display) => {
            display.clipboard().set_text(state::RECEIPT_BYTES);
            let bytes = state::RECEIPT_BYTES.len();
            format!("Copied {bytes} exact bytes")
        }
        None => "Clipboard unavailable — select the bytes below".to_owned(),
    };
    button.set_label(&label);
    glib::timeout_add_local_once(Duration::from_millis(2200), move || {
        button.set_label("Copy the exact receipt bytes");
    });
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    // The application holds the one strong reference to the UI model, so it
    // lives as long as the window. Every button holds only a weak reference
    // back to it, so the widget tree and the model never form a cycle.
    let model: Rc<RefCell<Option<Rc<Ui>>>> = Rc::new(RefCell::new(None));
    app.connect_activate(move |app| {
        *model.borrow_mut() = Some(build(app));
    });
    app.run()
}

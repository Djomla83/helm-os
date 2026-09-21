//! Shared builders for the recurring shapes of the accepted D8 design.
//!
//! GTK CSS paints but does not arrange, so every measurement from the
//! prototype — the 196 px label column, the 28 px ordinal column, the 620 px
//! body measure, each gap — lives here as a size request, a spacing or a
//! margin. Keeping them in one place is what stops the seven screens from
//! drifting apart.
//!
//! Nothing here is a HELM abstraction. These are local helpers for one spike.

use gtk::prelude::*;
use gtk4 as gtk;

/// Prototype measurements, named so the screens read like the design.
pub const LABEL_COL: i32 = 196;
pub const ORDINAL_COL: i32 = 28;
pub const BODY_MEASURE: i32 = 620;
pub const ROW_GAP: i32 = 40;
pub const ITEM_GAP: i32 = 14;

/// A label that wraps at a measure, left aligned, top aligned.
///
/// **A real friction point, recorded rather than smoothed over.** CSS says
/// `max-width: 620px` and the browser wraps there. GTK has no equivalent: a
/// wrapping `GtkLabel` reports its whole text as its natural width, so the
/// measure has to be expressed as `max-width-chars` and therefore approximated
/// from the font size. `CHARS_PER_PX` is that approximation. It is the one
/// place where the prototype's pixel measures do not survive translation
/// exactly.
pub fn wrapped(text: &str, class: &str, measure: i32) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class(class);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_xalign(0.0);
    label.set_yalign(0.0);
    label.set_halign(gtk::Align::Start);
    label.set_valign(gtk::Align::Start);
    label.set_hexpand(false);
    if measure > 0 {
        label.set_max_width_chars(chars_for(measure));
    }
    label
}

/// Roughly one character per this many pixels at the body sizes this design
/// uses. Calibrated by measuring rendered screens against the prototype, not
/// derived: at 7 the 620 px measure overshot by about 13 per cent.
const CHARS_PER_PX: i32 = 8;

pub const fn chars_for(measure: i32) -> i32 {
    let chars = measure / CHARS_PER_PX;
    if chars < 16 { 16 } else { chars }
}

/// A fixed-width column, where the prototype gives an exact pixel width and
/// GTK can honour it directly.
pub fn column(text: &str, class: &str, width: i32) -> gtk::Label {
    let label = wrapped(text, class, width);
    label.set_size_request(width, -1);
    label.set_max_width_chars(chars_for(width));
    label
}

/// A single-line label.
pub fn line(text: &str, class: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_halign(gtk::Align::Start);
    label
}

/// Tabular figures.
///
/// `.numeric` is libadwaita's style class for this; it is documented as
/// equivalent to a Pango `font-features` attribute of `tnum=1`. It is applied
/// to every figure the design aligns in a column — byte counts, digests,
/// timestamps, paths, the elapsed readout — because proportional figures make
/// a ledger unreadable.
pub fn numeric(label: &gtk::Label) {
    label.add_css_class("numeric");
}

pub fn line_numeric(text: &str, class: &str) -> gtk::Label {
    let label = line(text, class);
    numeric(&label);
    label
}

pub fn hbox(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Horizontal, spacing)
}

pub fn vbox(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Vertical, spacing)
}

/// One of the four state marks. A shape, never a colour on its own.
pub fn mark(class: &str, small: bool) -> gtk::Box {
    let shape = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    shape.add_css_class("helm-mark");
    shape.add_css_class(class);
    if small {
        shape.add_css_class("mark-sm");
    }
    shape.set_valign(gtk::Align::Center);
    shape.set_halign(gtk::Align::Center);
    // A state mark is decoration beside a word that already says the state, so
    // it is hidden from assistive technology rather than announced twice.
    shape.set_can_focus(false);
    shape.update_state(&[gtk::accessible::State::Hidden(true)]);
    shape
}

/// `label column | body`, the row that carries most of the product.
pub fn section_row(label: &gtk::Widget, body: &impl IsA<gtk::Widget>) -> gtk::Box {
    let row = hbox(ROW_GAP);
    row.set_valign(gtk::Align::Start);
    label.set_size_request(LABEL_COL, -1);
    label.set_valign(gtk::Align::Start);
    row.append(label);
    // Deliberately no `hexpand`: the body sizes itself, so a `field_body`
    // holds the prototype's 620 px measure instead of stretching to the row.
    let body = body.as_ref();
    body.set_halign(gtk::Align::Start);
    row.append(body);
    row
}

/// The common case: a bold label string against a body widget.
pub fn labelled_row(label_text: &str, body: &impl IsA<gtk::Widget>) -> gtk::Box {
    let label = column(label_text, "helm-rowlabel", LABEL_COL);
    section_row(label.upcast_ref::<gtk::Widget>(), body)
}

/// `key ........ value`, hairline underneath. The fact line of the design.
pub fn fact(key: &str, value: &str) -> gtk::Box {
    let row = hbox(20);
    row.add_css_class("rule-soft-bottom");
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    let k = line(key, "helm-factk");
    let v = line_numeric(value, "helm-factv");
    v.set_hexpand(true);
    v.set_halign(gtk::Align::End);
    v.set_xalign(1.0);
    v.set_wrap(true);
    v.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    v.set_justify(gtk::Justification::Right);
    row.append(&k);
    row.append(&v);
    row
}

/// The advanced-disclosure fact: a plain-language label, the field name
/// underneath it, and the value. Technical detail must look like data.
pub fn fact_advanced(label: &str, field: &str, value: &str) -> gtk::Box {
    let row = hbox(20);
    row.add_css_class("rule-soft-top");
    row.set_margin_top(8);
    row.set_margin_bottom(8);

    let left = vbox(2);
    left.append(&line(label, "helm-factlabel"));
    left.append(&line_numeric(field, "helm-factcode"));

    let v = line_numeric(value, "helm-factv-sm");
    v.set_hexpand(true);
    v.set_halign(gtk::Align::End);
    v.set_xalign(1.0);
    v.set_valign(gtk::Align::Start);

    row.append(&left);
    row.append(&v);
    row
}

/// `01 | key | value`, the numbered ledger line used by Authority, Result and
/// Evidence.
pub fn numbered(
    ordinal: &str,
    key: &str,
    value: &str,
    key_width: i32,
    key_plain: bool,
    value_numeric: bool,
) -> gtk::Box {
    let row = hbox(ITEM_GAP);
    row.add_css_class("rule-top");
    row.set_margin_top(12);
    row.set_margin_bottom(12);
    row.set_valign(gtk::Align::Start);

    let n = line_numeric(ordinal, "helm-itemn");
    n.set_size_request(ORDINAL_COL, -1);
    n.set_valign(gtk::Align::Start);

    let k = column(
        key,
        if key_plain {
            "helm-itemk-plain"
        } else {
            "helm-itemk"
        },
        key_width,
    );

    let v = wrapped(value, "helm-itemv", BODY_MEASURE);
    v.set_hexpand(true);
    if value_numeric {
        numeric(&v);
    }

    row.append(&n);
    row.append(&k);
    row.append(&v);
    row
}

/// A numbered line whose key carries its field name underneath, for Evidence.
pub fn numbered_field(ordinal: &str, key: &str, field: &str, value: &str) -> gtk::Box {
    let row = hbox(ITEM_GAP);
    row.add_css_class("rule-top");
    row.set_margin_top(11);
    row.set_margin_bottom(11);
    row.set_valign(gtk::Align::Start);

    let n = line_numeric(ordinal, "helm-itemn");
    n.set_size_request(ORDINAL_COL, -1);
    n.set_valign(gtk::Align::Start);

    let k = vbox(2);
    k.set_size_request(210, -1);
    k.set_hexpand(false);
    k.set_halign(gtk::Align::Start);
    k.set_valign(gtk::Align::Start);
    let key_label = column(key, "helm-itemk-plain", 210);
    key_label.set_max_width_chars(26);
    let field_label = column(field, "helm-factcode", 210);
    field_label.set_max_width_chars(26);
    k.append(&key_label);
    k.append(&field_label);

    let v = wrapped(value, "helm-itemv", BODY_MEASURE);
    numeric(&v);
    v.set_hexpand(true);

    row.append(&n);
    row.append(&k);
    row.append(&v);
    row
}

/// A section heading, indented to the ordinal column so it aligns with the
/// numbered lines beneath it.
pub fn grouphead(text: &str) -> gtk::Label {
    let label = line(text, "helm-grouphead");
    label.set_margin_start(ORDINAL_COL + ITEM_GAP);
    label
}

pub fn primary_button(text: &str) -> gtk::Button {
    let button = gtk::Button::with_label(text);
    button.add_css_class("helm-primary");
    button.set_valign(gtk::Align::Center);
    button
}

pub fn quiet_button(text: &str, strong: bool, chip: bool) -> gtk::Button {
    let button = gtk::Button::with_label(text);
    button.add_css_class("helm-quiet");
    if strong {
        button.add_css_class("btn-strong");
    }
    if chip {
        button.add_css_class("btn-chip");
    }
    button.set_valign(gtk::Align::Center);
    button
}

/// A link-styled action. It is a real `GtkButton`, not a label with a gesture,
/// so it keeps its role, its keyboard behaviour and its focus ring.
pub fn link_button(text: &str, muted: bool) -> gtk::Button {
    let button = gtk::Button::with_label(text);
    button.add_css_class("helm-link");
    if muted {
        button.add_css_class("link-muted");
    }
    button.set_valign(gtk::Align::Center);
    button.set_has_frame(false);
    button
}

/// A body column clamped to the prototype's measure.
pub fn field_body() -> gtk::Box {
    let body = vbox(0);
    body.set_size_request(BODY_MEASURE, -1);
    body.set_halign(gtk::Align::Start);
    body.set_valign(gtk::Align::Start);
    body
}

/// Selectable monospace text, for the digest and the exact receipt bytes.
pub fn selectable_mono(text: &str, class: &str) -> gtk::Label {
    let label = wrapped(text, class, BODY_MEASURE);
    label.set_hexpand(true);
    label.set_selectable(true);
    label
}

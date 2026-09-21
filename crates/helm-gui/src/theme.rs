//! The HELM stylesheet for the G2-D9 fidelity spike.
//!
//! Translated from `docs/prototypes/g2-html/styles.css`, which is the visual
//! source of truth. Where the two differ the prototype is right and this file
//! is wrong.
//!
//! Two GTK realities shape how this is written.
//!
//! **Colours are `@define-color`, not CSS custom properties.** GTK only gained
//! CSS custom properties in 4.16. This spike targets what a current LTS
//! distribution ships, so the HELM tokens use GTK's own colour-variable
//! mechanism, which has been available far longer and is the idiomatic GTK
//! spelling anyway.
//!
//! **GTK CSS does not do layout.** There is no `display`, `flex`, `grid`,
//! `position`, `width` or `height`, and percentages are not accepted for
//! padding or margin. Everything structural — the 250 px rail, the 196 px
//! label column, the measured content column, every gap — is expressed in
//! `widgets.rs` and `screens.rs` as size requests, spacing and margins. This
//! file paints; it does not arrange.

/// Applied at `GTK_STYLE_PROVIDER_PRIORITY_APPLICATION`, above the theme.
pub const CSS: &str = r#"
/* ---------------------------------------------------------------- tokens */
/* The G2-D8 working token set. Working tokens, not final branding. */
@define-color helm_burgundy      #7A1F3D;
@define-color helm_burgundy_deep #5C1730;
@define-color helm_graphite      #1F2937;
@define-color helm_graphite_deep #1A222D;
@define-color helm_sheet         #F8F6F4;
@define-color helm_rose          #D7A8B4;
@define-color helm_cool_gray     #CBD0D6;

@define-color helm_ink   #1F2937;
@define-color helm_ink_2 #3F4854;
@define-color helm_ink_3 #5B6673;
@define-color helm_ink_4 #6E6862;
@define-color helm_ink_5 #6F6A61;

@define-color helm_rule_strong #1F2937;
@define-color helm_rule        #DED9D3;
@define-color helm_rule_soft   #E6E1DB;
@define-color helm_rule_sheet  #E2DDD7;
@define-color helm_btn_border  #C9C2BA;

@define-color helm_nav_idle   #AEB8C4;
@define-color helm_rail_label #96A2B0;
@define-color helm_title_text #8E9AA8;
@define-color helm_statusbar  #F3EFEB;

/* ------------------------------------------------------------- app shell */

.helm-root {
  background-color: @helm_graphite;
  font-family: "Inter", "Cantarell", "DejaVu Sans", sans-serif;
  font-size: 14px;
  letter-spacing: -0.006em;
  color: @helm_ink;
}

.helm-titlebar {
  background-color: @helm_graphite_deep;
  background-image: none;
  border: none;
  box-shadow: none;
  min-height: 36px;
  padding: 0 9px;
}
.helm-titlename { font-size: 12.5px; color: @helm_title_text; }

/* --------------------------------------------------------- graphite rail */

.helm-rail { background-color: @helm_graphite; }
.helm-wordmark {
  font-family: "DejaVu Serif", "Liberation Serif", serif;
  font-size: 23px;
  color: @helm_sheet;
}
.helm-g2 { font-size: 12px; color: @helm_rose; }

.helm-navitem {
  background-image: none;
  background-color: transparent;
  border: none;
  box-shadow: none;
  text-shadow: none;
  outline: none;
  border-radius: 0;
  min-height: 0;
  min-width: 0;
  padding: 8px 26px;
  font-size: 14px;
  font-weight: 400;
  color: @helm_nav_idle;
}
.helm-navitem:hover { background-color: transparent; color: #FFFFFF; }
.helm-navitem:checked,
.helm-navitem:active { background-color: transparent; color: #FFFFFF; }
.helm-navitem:focus-visible { outline: 2px solid @helm_rose; outline-offset: -2px; }

/* The active indicator is a shape, not only a colour: idle items carry no
   dot at all, so the state survives a monochrome rendering. */
.helm-navdot { min-width: 5px; min-height: 5px; border-radius: 50%; background-color: transparent; }
.helm-navitem:checked .helm-navdot { background-color: @helm_rose; }

.helm-raillabel { font-size: 12.5px; color: @helm_rail_label; }
.helm-railnote  { font-size: 12px; color: @helm_nav_idle; }

/* -------------------------------------------------------- working sheet */

.helm-sheet { background-color: @helm_sheet; border-radius: 6px; }
.helm-sheet scrolledwindow,
.helm-sheet viewport,
.helm-scroll { background-color: @helm_sheet; background-image: none; }

.helm-statusbar {
  background-color: @helm_statusbar;
  border-top: 1px solid @helm_rule_sheet;
  padding: 11px 48px;
}
.helm-statustext { font-size: 12.5px; color: @helm_ink_2; }
.helm-statusfact { font-size: 12.5px; color: @helm_ink_4; }

.helm-modebtn {
  background-image: none;
  background-color: transparent;
  border: none;
  box-shadow: none;
  text-shadow: none;
  border-radius: 0;
  min-height: 0;
  min-width: 0;
  padding: 0;
  font-size: 12.5px;
  font-weight: 400;
  color: @helm_ink_4;
}
.helm-modebtn:hover { color: @helm_ink; text-decoration: underline; }

/* -------------------------------------------------- state marks (shapes) */

.helm-mark {
  min-width: 9px;
  min-height: 9px;
  border: 1.5px solid @helm_graphite;
  background-color: transparent;
}
.helm-mark.mark-known     { border-color: @helm_graphite; background-color: transparent; }
.helm-mark.mark-available { border-color: @helm_burgundy; background-color: transparent; }
.helm-mark.mark-attempt   { border-color: @helm_burgundy; background-color: transparent; border-radius: 50%; }
.helm-mark.mark-ended     { border-color: @helm_graphite; background-color: @helm_graphite; }
.helm-mark.mark-sm        { min-width: 8px; min-height: 8px; }

/* The diamond is the outlined square turned 45 degrees. */
.helm-mark.mark-available { transform: rotate(45deg); }

.helm-mark-refused {
  min-width: 9px;
  min-height: 9px;
  border: 1.5px solid @helm_burgundy;
  transform: rotate(45deg);
}
.helm-mark-attempt-lg {
  min-width: 11px;
  min-height: 11px;
  border: 1.5px solid @helm_burgundy;
  border-radius: 50%;
}

/* --------------------------------------------------------- typography */

.helm-h1 {
  font-family: "DejaVu Serif", "Liberation Serif", serif;
  font-size: 34px;
  font-weight: 400;
  letter-spacing: -0.015em;
  color: @helm_ink;
}
.helm-h1-lg { font-size: 38px; }

.helm-lede    { font-size: 14px; color: @helm_ink_3; }
.helm-lede-ink{ font-size: 14px; color: @helm_ink_2; }
.helm-crumb   { font-size: 13px; color: @helm_ink_3; }
.helm-micro   { font-size: 12.5px; color: @helm_ink_3; }
.helm-note    { font-size: 13.5px; color: @helm_ink_3; }
.helm-note-ink{ font-size: 13.5px; color: @helm_ink_2; }
.helm-note-sm { font-size: 13px; color: @helm_ink_2; }

.helm-pull {
  font-family: "DejaVu Serif", "Liberation Serif", serif;
  font-size: 19px;
  color: @helm_ink;
}
.helm-pull-plain { font-size: 14px; color: @helm_ink_2; }

.helm-mono { font-family: monospace; font-size: 12px; color: @helm_ink_2; }
.helm-digest { font-family: monospace; font-size: 12.5px; color: @helm_ink; }
.helm-bytes  { font-family: monospace; font-size: 11.5px; color: @helm_ink_2; }

/* ------------------------------------------------------ ledger and facts */

.helm-grouphead { font-size: 15px; font-weight: 600; color: @helm_ink; }
.helm-rowlabel  { font-size: 15px; font-weight: 600; color: @helm_ink; }
.helm-rowlabel-accent { font-size: 15px; font-weight: 600; color: @helm_burgundy; }
.helm-rowphase  { font-size: 12.5px; color: @helm_ink_3; }
.helm-rowbody   { font-size: 13.5px; color: @helm_ink_3; }
.helm-rowbody-sm{ font-size: 13px; color: @helm_ink_3; }
.helm-rowbody-ink { font-size: 13.5px; color: @helm_ink_2; }

.helm-factk { font-size: 13px; color: @helm_ink_3; }
.helm-factv { font-size: 13px; color: @helm_ink; }
.helm-factv-sm { font-size: 12.5px; color: @helm_ink_2; }
.helm-factlabel { font-size: 12.5px; color: @helm_ink; }
.helm-factcode  { font-size: 12px; color: @helm_ink_4; }

.helm-itemn { font-size: 12.5px; color: @helm_ink_5; }
.helm-itemk { font-size: 13px; font-weight: 500; color: @helm_ink; }
.helm-itemk-plain { font-size: 13px; font-weight: 400; color: @helm_ink; }
.helm-itemv { font-size: 13px; color: @helm_ink_2; }

.helm-entryname { font-size: 16.5px; font-weight: 500; color: @helm_ink; }
.helm-entrypath { font-size: 12.5px; color: @helm_ink_3; }
.helm-fieldtitle { font-size: 16px; font-weight: 500; color: @helm_ink; }
.helm-fieldpath  { font-size: 13.5px; color: @helm_ink; }
.helm-stateword  { font-size: 13px; font-weight: 500; color: @helm_ink; }
.helm-stateword-lg { font-size: 13.5px; font-weight: 500; color: @helm_ink; }
.helm-statenote  { font-size: 12.5px; color: @helm_ink_3; }
.helm-statedash  { font-size: 13px; color: @helm_ink_3; }
.helm-pagemeta   { font-size: 13px; color: @helm_ink_3; }

.helm-refusal-title { font-size: 14px; font-weight: 600; color: @helm_ink; }
.helm-refusal-text  { font-size: 13.5px; color: @helm_ink_2; }
.helm-refusal-code  { font-size: 12px; color: @helm_ink_4; }

.helm-elapsed { font-size: 28px; color: @helm_ink; }

/* --------------------------------------------------------------- rules */
/* Structure is hairlines and type, never a card. Three weights, all 1px. */

.rule-strong-top    { border-top: 1px solid @helm_rule_strong; }
.rule-strong-bottom { border-bottom: 1px solid @helm_rule_strong; }
.rule-top           { border-top: 1px solid @helm_rule; }
.rule-bottom        { border-bottom: 1px solid @helm_rule; }
.rule-soft-top      { border-top: 1px solid @helm_rule_soft; }
.rule-soft-bottom   { border-bottom: 1px solid @helm_rule_soft; }

/* Section rows put their 22 px inside the rule. A GTK margin sits outside the
   border, so the prototype's `padding: 22px 0` has to be real CSS padding
   here; the margins on the rows remain the gap *above* the rule. */
.helm-sec-top        { border-top: 1px solid @helm_rule; padding-top: 22px; }
.helm-sec-top-strong { border-top: 1px solid @helm_rule_strong; padding-top: 22px; }
.helm-sec-bottom     { border-bottom: 1px solid @helm_rule; padding-top: 22px; padding-bottom: 22px; }
.helm-pagehead       { border-bottom: 1px solid @helm_rule_strong; padding-bottom: 20px; }

/* ------------------------------------------------------------- buttons */

.helm-primary {
  background-image: none;
  background-color: @helm_burgundy;
  color: @helm_sheet;
  border: none;
  border-radius: 2px;
  box-shadow: none;
  text-shadow: none;
  min-height: 0;
  min-width: 0;
  padding: 11px 20px;
  font-size: 13px;
  font-weight: 500;
}
.helm-primary:hover { background-color: @helm_burgundy_deep; }
.helm-primary:disabled { background-color: @helm_btn_border; color: @helm_sheet; }
.helm-primary:focus-visible { outline: 2px solid @helm_graphite; outline-offset: 2px; }
.helm-primary.btn-sm { padding: 10px 18px; }

.helm-quiet {
  background-image: none;
  background-color: transparent;
  color: @helm_ink;
  border: 1px solid @helm_btn_border;
  border-radius: 2px;
  box-shadow: none;
  text-shadow: none;
  min-height: 0;
  min-width: 0;
  padding: 9px 16px;
  font-size: 13px;
  font-weight: 400;
}
.helm-quiet:hover { border-color: @helm_graphite; background-color: transparent; }
.helm-quiet:focus-visible { outline: 2px solid @helm_burgundy; outline-offset: 2px; }
.helm-quiet.btn-strong { font-weight: 500; }
.helm-quiet.btn-chip { padding: 9px 14px; }

.helm-link {
  background-image: none;
  background-color: transparent;
  color: @helm_burgundy;
  border: none;
  border-radius: 0;
  box-shadow: none;
  text-shadow: none;
  min-height: 0;
  min-width: 0;
  padding: 0;
  font-size: 13px;
  font-weight: 400;
}
.helm-link:hover { color: @helm_burgundy_deep; background-color: transparent; }
.helm-link:focus-visible { outline: 2px solid @helm_burgundy; outline-offset: 2px; }
.helm-link.link-muted { color: @helm_ink_3; }

/* ---------------------------------------------- the bound meter (2 px) */

.helm-meter { min-height: 2px; }
.helm-meter trough {
  min-height: 2px;
  border: none;
  border-radius: 0;
  background-color: @helm_rule;
  background-image: none;
}
.helm-meter progress {
  min-height: 2px;
  border: none;
  border-radius: 0;
  background-color: @helm_burgundy;
  background-image: none;
}

/* ------------------------------------------------- chooser stand-in */

.helm-chooser-title { font-size: 12.5px; color: @helm_ink_3; }
.helm-chooser-path  { font-size: 12.5px; color: @helm_ink_3; }
.helm-chooser-item {
  background-image: none;
  background-color: transparent;
  border: none;
  border-radius: 0;
  box-shadow: none;
  min-height: 0;
  padding: 11px 0;
}
.helm-chooser-item:hover { background-color: #EFEAE4; }
.helm-chooser-name { font-size: 13.5px; color: @helm_ink; }
.helm-chooser-size { font-size: 12.5px; color: @helm_ink_3; }
"#;

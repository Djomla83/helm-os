//! **EXPERIMENTAL — HELM G2 first real backend-connected vertical.**
//!
//! Not an accepted HELM product surface. This library holds the two parts of
//! the G2 interface that are worth testing without opening a window:
//!
//! * [`state`] — the mapping from real `helm-launch` facts to the words HELM is
//!   allowed to say about them;
//! * [`vertical`] — the narrow orchestration adapter that opens what a person
//!   selected, hands it to the accepted public API, and carries the facts back.
//!
//! The widgets live in the binary. Nothing here draws anything, and nothing
//! here is a general HELM abstraction.
//!
//! **Linux x86_64 only.** `helm-launch`'s admission, authorisation and launch
//! exist on that cohort alone, and this crate drives them directly.

pub mod state;
pub mod vertical;

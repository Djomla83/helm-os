//! Capability admission and the authorisation boundary.
//!
//! Three boundaries, per accepted ADR-0022:
//!   1. untrusted plan bytes  -> `ValidatedPlan`   (no authority at all)
//!   2. trusted caller        -> `AuthorizedScope` (deliberate, exact grant)
//!   3. authorised scope      -> observation       (only those targets)
//!
//! Obligation C is satisfied *by construction*: [`authorize`] consumes the
//! `ValidatedPlan` and moves it into the scope, and `observe` takes only the
//! scope. There is no public way to express "authorise plan A, observe plan B".

use std::os::fd::OwnedFd;

use crate::error::{AdmissionError, AdmissionErrorCode as A, AdmissionErrors};
use crate::plan::{Digest, ValidatedPlan};

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use crate::linux;

/// An already-open directory descriptor the trusted caller chose to grant,
/// bound to one validated logical root ID.
///
/// There is deliberately no path-based constructor: the observer never resolves
/// a name to obtain authority, and never consults HOME, cwd, `/`, PATH, a
/// prefix locator or any installation-discovery rule.
#[derive(Debug)]
pub struct RootCapability {
    id: String,
    fd: OwnedFd,
    tuple: Option<crate::model::ObjectTuple>,
}

impl RootCapability {
    /// Validated logical root ID. Never a host path.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn fd(&self) -> &OwnedFd {
        &self.fd
    }

    pub(crate) const fn tuple(&self) -> Option<crate::model::ObjectTuple> {
        self.tuple
    }
}

/// The trusted current-process procfs descriptor-directory capability.
///
/// Its only permitted use is reopening an internally generated descriptor number
/// that this call already pinned. It is not an observation root, cannot be named
/// by a target, is never enumerated, and cannot reach another process.
#[derive(Debug)]
pub struct ProcFdCapability {
    fd: OwnedFd,
}

impl ProcFdCapability {
    pub(crate) fn fd(&self) -> &OwnedFd {
        &self.fd
    }
}

/// Admit a caller-supplied directory descriptor as a named root capability.
///
/// # Errors
/// Fails when the descriptor is not a directory, its metadata is unavailable, or
/// its filesystem is outside the supported local-ext4 0.1 cohort.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn root_from_fd(id: &str, fd: OwnedFd) -> Result<RootCapability, AdmissionError> {
    let tuple = linux::admit_root(&fd)?;
    Ok(RootCapability {
        id: id.to_owned(),
        fd,
        tuple: Some(tuple),
    })
}

/// Admit the caller-supplied current-process procfs descriptor directory.
///
/// Runs both reviewed checks: the filesystem really is procfs, and the directory
/// really maps *this* process's descriptor numbers.
///
/// # Errors
/// Fails for a non-procfs decoy, a foreign process's descriptor directory, or an
/// otherwise unusable capability.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn proc_fd_from_trusted_current_process(
    fd: OwnedFd,
) -> Result<ProcFdCapability, AdmissionError> {
    linux::admit_procfs(&fd)?;
    Ok(ProcFdCapability { fd })
}

/// A deliberate grant of exactly one plan against exactly one root set.
#[derive(Debug)]
pub struct AuthorizedScope {
    plan: ValidatedPlan,
    authorized_plan_sha256: Digest,
    roots: Vec<RootCapability>,
    reopen: ProcFdCapability,
}

impl AuthorizedScope {
    /// The exact plan identity this scope authorises, over all 32 digest bytes.
    #[must_use]
    pub const fn authorized_plan_sha256(&self) -> Digest {
        self.authorized_plan_sha256
    }

    /// Borrowed view of the authorised plan. It cannot be replaced or mutated:
    /// `ValidatedPlan` has no public constructor other than parsing, and this
    /// borrow hands out no ownership.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedPlan {
        &self.plan
    }

    pub(crate) fn root(&self, id: &str) -> Option<&RootCapability> {
        self.roots.iter().find(|r| r.id == id)
    }

    pub(crate) fn roots(&self) -> &[RootCapability] {
        &self.roots
    }

    pub(crate) const fn reopen(&self) -> &ProcFdCapability {
        &self.reopen
    }
}

/// The trusted caller's deliberate grant for this exact plan and this exact root set.
///
/// Consumes the plan, so the scope owns the only authorised plan and `observe`
/// cannot be handed a different one (obligation C). Binds the complete plan
/// SHA-256 (obligation A) and matches roots by logical ID, never by position
/// (obligation B).
///
/// # Errors
/// Fails when a declared root has no capability, a capability is not declared by
/// the plan, or the same logical root ID is supplied more than once. Passing
/// validation alone never grants anything.
pub fn authorize(
    plan: ValidatedPlan,
    roots: Vec<RootCapability>,
    reopen: ProcFdCapability,
) -> Result<AuthorizedScope, AdmissionErrors> {
    let mut errors = Vec::new();

    // Obligation B: bind by ID, never by vector position.
    let mut seen: Vec<&str> = Vec::new();
    for cap in &roots {
        if seen.contains(&cap.id.as_str()) {
            errors.push(AdmissionError::for_root(A::DuplicateRoot, &cap.id));
        } else {
            seen.push(&cap.id);
        }
        if !plan.root_ids().iter().any(|d| d == &cap.id) {
            errors.push(AdmissionError::for_root(A::ExtraRoot, &cap.id));
        }
    }
    for declared in plan.root_ids() {
        if !roots.iter().any(|c| &c.id == declared) {
            errors.push(AdmissionError::for_root(A::MissingRoot, declared));
        }
    }
    if !errors.is_empty() {
        return Err(AdmissionErrors::new(errors));
    }

    // Obligation A: bind the complete exact plan identity.
    let authorized_plan_sha256 = plan.sha256();
    Ok(AuthorizedScope {
        plan,
        authorized_plan_sha256,
        roots,
        reopen,
    })
}

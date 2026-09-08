//! Immutable desired-state views. No public constructor, Deserialize or mutation API.

/// A declared SHA-256 identity, or the computed exact document identity.
///
/// ```compile_fail
/// let invalid = helm_app_spec::Digest("not-a-digest".to_owned());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest(pub(crate) String);
impl Digest {
    /// Exactly 64 lowercase hexadecimal characters.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A bounded local identifier; no global catalogue or publisher identity is implied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub(crate) String);
impl Identifier {
    /// Validated identifier spelling, without normalization.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Inert prefix-relative Windows C-drive entry-point spelling; never a filesystem capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeEntryPoint(pub(crate) String);
impl RelativeEntryPoint {
    /// Exact decoded spelling, beginning `drive_c/`, using only forward slashes.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The only declared application-source architecture in schema 0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceArchitecture {
    /// Windows x86-64 source; this is not a PE inspection result.
    X86_64,
}
/// The only desired runtime family in schema 0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFamily {
    /// Wine requirements; no runtime discovery or selection.
    Wine,
}
/// The only desired Windows environment architecture in schema 0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsArchitecture {
    /// Dedicated win64 intent, not a host-architecture observation.
    Win64,
}
/// Desired prefix association, with no location, instance identity or ownership proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixRole {
    /// Dedicated to this application by intent; grants no creation/deletion authority.
    Dedicated,
}

/// Desired source byte identity; application labels do not substitute for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRequirement {
    pub(crate) size: u64,
    pub(crate) sha256: Digest,
    pub(crate) architecture: SourceArchitecture,
}
impl SourceRequirement {
    /// Required source byte length, not an observed file length.
    pub fn size(&self) -> u64 {
        self.size
    }
    /// Required source digest; the source bytes were not supplied or hashed here.
    pub fn sha256(&self) -> &Digest {
        &self.sha256
    }
    /// Declared architecture, never discovered from the source artifact.
    pub fn architecture(&self) -> SourceArchitecture {
        self.architecture
    }
}

/// Desired application subject and bounded human version metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationRequirement {
    pub(crate) id: Identifier,
    pub(crate) version: String,
    pub(crate) source: SourceRequirement,
}
impl ApplicationRequirement {
    /// Local application ID; no application-specific branches depend on its value.
    pub fn id(&self) -> &Identifier {
        &self.id
    }
    /// Human metadata, never used as immutable identity or parsed as a version range.
    pub fn version(&self) -> &str {
        &self.version
    }
    /// Immutable desired source requirement.
    pub fn source(&self) -> &SourceRequirement {
        &self.source
    }
}

/// One immutable runtime artifact requirement. It says nothing about installed/loaded bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRequirement {
    pub(crate) role: Identifier,
    pub(crate) size: u64,
    pub(crate) sha256: Digest,
    pub(crate) label: Option<String>,
}
impl ArtifactRequirement {
    /// Unique local role within the runtime requirement.
    pub fn role(&self) -> &Identifier {
        &self.role
    }
    /// Required artifact byte length.
    pub fn size(&self) -> u64 {
        self.size
    }
    /// Required immutable artifact digest, not installed-loader evidence.
    pub fn sha256(&self) -> &Digest {
        &self.sha256
    }
    /// Optional human metadata with no selection or identity semantics.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

/// A bounded desired artifact set, not a hermetic runtime closure or package resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequirement {
    pub(crate) family: RuntimeFamily,
    pub(crate) artifacts: Vec<ArtifactRequirement>,
}
impl RuntimeRequirement {
    /// Desired runtime family.
    pub fn family(&self) -> RuntimeFamily {
        self.family
    }
    /// 1-16 requirements in declaration order; order is not execution order.
    pub fn artifacts(&self) -> &[ArtifactRequirement] {
        &self.artifacts
    }
}

/// Minimal desired configuration. No registry, environment-variable or override language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentRequirement {
    pub(crate) windows_architecture: WindowsArchitecture,
    pub(crate) prefix_role: PrefixRole,
    pub(crate) disabled_dlls: Vec<String>,
}
impl EnvironmentRequirement {
    /// Required Windows architecture.
    pub fn windows_architecture(&self) -> WindowsArchitecture {
        self.windows_architecture
    }
    /// Application association intent only; ownership remains unclassified.
    pub fn prefix_role(&self) -> PrefixRole {
        self.prefix_role
    }
    /// Unique lowercase DLL basenames requested disabled (neither native nor builtin).
    /// An empty list requests no disables; effective configuration is not observed.
    pub fn disabled_dlls(&self) -> &[String] {
        &self.disabled_dlls
    }
}

/// One desired entry point; no existence, installation or execution assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryPointRequirement {
    pub(crate) path: RelativeEntryPoint,
    pub(crate) sha256: Option<Digest>,
}
impl EntryPointRequirement {
    /// Inert prefix-relative path, never opened, expanded or normalized.
    pub fn path(&self) -> &RelativeEntryPoint {
        &self.path
    }
    /// Optional desired bytes for a future check; None leaves identity unspecified.
    pub fn sha256(&self) -> Option<&Digest> {
        self.sha256.as_ref()
    }
}

/// Reference to frozen pre-execution definition bytes, never to resulting evidence.
/// Content, chronology and provenance cannot be established from the digest alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationDefinitionRef {
    pub(crate) role: Identifier,
    pub(crate) size: u64,
    pub(crate) sha256: Digest,
}
impl VerificationDefinitionRef {
    /// Unique local definition role; no workflow language or provider mechanism.
    pub fn role(&self) -> &Identifier {
        &self.role
    }
    /// Required definition byte length.
    pub fn size(&self) -> u64 {
        self.size
    }
    /// Frozen definition's exact-byte digest, never resolved by this library.
    pub fn sha256(&self) -> &Digest {
        &self.sha256
    }
}

/// A syntactically and semantically valid desired declaration under schema 0.1.
/// It establishes no observed state, compatibility, trust, ownership or authority.
///
/// ```compile_fail
/// fn mutate(spec: &mut helm_app_spec::ValidatedAppSpec) {
///     spec.runtime().artifacts().clear();
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAppSpec {
    pub(crate) spec_sha256: Digest,
    pub(crate) application: ApplicationRequirement,
    pub(crate) runtime: RuntimeRequirement,
    pub(crate) environment: EnvironmentRequirement,
    pub(crate) entry_point: EntryPointRequirement,
    pub(crate) verification: Vec<VerificationDefinitionRef>,
}
impl ValidatedAppSpec {
    /// SHA-256 of EXACT INPUT BYTES, including whitespace and field order.
    pub fn spec_sha256(&self) -> &Digest {
        &self.spec_sha256
    }
    /// Desired application/source subject.
    pub fn application(&self) -> &ApplicationRequirement {
        &self.application
    }
    /// Desired runtime artifacts; never evidence of the loader actually executed.
    pub fn runtime(&self) -> &RuntimeRequirement {
        &self.runtime
    }
    /// Desired configuration, without a bound prefix instance or observed environment.
    pub fn environment(&self) -> &EnvironmentRequirement {
        &self.environment
    }
    /// Desired inert entry point.
    pub fn entry_point(&self) -> &EntryPointRequirement {
        &self.entry_point
    }
    /// 1-8 frozen definition references in declaration order, independent of future results.
    pub fn verification(&self) -> &[VerificationDefinitionRef] {
        &self.verification
    }
}

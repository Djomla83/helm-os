use crate::parse::Value;
use crate::{ErrorCode as Code, SCHEMA, SpecErrors, VERSION, model::*};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeMap;

type Object = BTreeMap<String, Value>;

fn object<'a>(
    value: Option<&'a Value>,
    location: &str,
    fields: &[&str],
    optional: &[&str],
    errors: &mut SpecErrors,
) -> Option<&'a Object> {
    let value = value?;
    let Some(map) = value.as_object() else {
        errors.push(Code::FieldType, location);
        return None;
    };
    if map.keys().any(|key| !fields.contains(&key.as_str())) {
        errors.push(Code::FieldUnknown, location);
    }
    for name in fields {
        if !optional.contains(name) && !map.contains_key(*name) {
            errors.push(Code::FieldMissing, &format!("{location}.{name}"));
        }
    }
    Some(map)
}

fn string<'a>(
    value: Option<&'a Value>,
    location: &str,
    errors: &mut SpecErrors,
) -> Option<&'a str> {
    let value = value?;
    match value.as_str() {
        Some(text) => Some(text),
        None => {
            errors.push(Code::FieldType, location);
            None
        }
    }
}

fn text<'a>(
    value: Option<&'a Value>,
    location: &str,
    code: Code,
    valid: impl FnOnce(&str) -> bool,
    errors: &mut SpecErrors,
) -> Option<&'a str> {
    let text = string(value, location, errors)?;
    if valid(text) {
        Some(text)
    } else {
        errors.push(code, location);
        None
    }
}

fn literal(
    value: Option<&Value>,
    location: &str,
    expected: &str,
    code: Code,
    errors: &mut SpecErrors,
) -> Option<()> {
    text(value, location, code, |s| s == expected, errors).map(|_| ())
}

fn identifier(
    value: Option<&Value>,
    location: &str,
    errors: &mut SpecErrors,
) -> Option<Identifier> {
    text(
        value,
        location,
        Code::IdentifierInvalid,
        |s| {
            (1..=80).contains(&s.len())
                && s.bytes()
                    .next()
                    .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        },
        errors,
    )
    .map(|s| Identifier(s.to_owned()))
}

fn digest(value: Option<&Value>, location: &str, errors: &mut SpecErrors) -> Option<Digest> {
    text(
        value,
        location,
        Code::DigestInvalid,
        |s| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        },
        errors,
    )
    .map(|s| Digest(s.to_owned()))
}

fn metadata(value: Option<&Value>, location: &str, errors: &mut SpecErrors) -> Option<String> {
    text(
        value,
        location,
        Code::MetadataInvalid,
        |s| (1..=256).contains(&s.len()) && s.bytes().all(|b| (32..=126).contains(&b)),
        errors,
    )
    .map(str::to_owned)
}

fn size(value: Option<&Value>, location: &str, errors: &mut SpecErrors) -> Option<u64> {
    let value = value?;
    match value.as_u64() {
        Some(size) if (1..=1_099_511_627_776).contains(&size) => Some(size),
        _ => {
            errors.push(Code::SizeInvalid, location);
            None
        }
    }
}

fn array<'a>(
    value: Option<&'a Value>,
    location: &str,
    min: usize,
    max: usize,
    errors: &mut SpecErrors,
) -> Option<&'a [Value]> {
    let value = value?;
    let Some(array) = value.as_array() else {
        errors.push(Code::FieldType, location);
        return None;
    };
    if !(min..=max).contains(&array.len()) {
        errors.push(Code::CollectionLimit, location);
        return None;
    }
    Some(array)
}

pub(crate) fn spec(value: &Value, bytes: &[u8]) -> Result<ValidatedAppSpec, SpecErrors> {
    let mut errors = SpecErrors(Vec::new());
    let result = spec_model(value, bytes, &mut errors);
    match result {
        Some(model) if errors.0.is_empty() => Ok(model),
        _ => Err(errors),
    }
}

fn spec_model(value: &Value, bytes: &[u8], errors: &mut SpecErrors) -> Option<ValidatedAppSpec> {
    let map = object(
        Some(value),
        "$",
        &[
            "schema",
            "version",
            "application",
            "runtime",
            "environment",
            "entry_point",
            "verification",
        ],
        &[],
        errors,
    )?;
    let schema = literal(
        map.get("schema"),
        "$.schema",
        SCHEMA,
        Code::SchemaUnknown,
        errors,
    );
    let version = literal(
        map.get("version"),
        "$.version",
        VERSION,
        Code::SchemaVersion,
        errors,
    );
    let application = application(map.get("application"), errors);
    let runtime = runtime(map.get("runtime"), errors);
    let environment = environment(map.get("environment"), errors);
    let entry_point = entry_point(map.get("entry_point"), errors);
    let verification = verification(map.get("verification"), errors);
    schema?;
    version?;
    Some(ValidatedAppSpec {
        spec_sha256: Digest(format!("{:x}", Sha256::digest(bytes))),
        application: application?,
        runtime: runtime?,
        environment: environment?,
        entry_point: entry_point?,
        verification: verification?,
    })
}

fn application(value: Option<&Value>, errors: &mut SpecErrors) -> Option<ApplicationRequirement> {
    let map = object(
        value,
        "$.application",
        &["id", "version", "source"],
        &[],
        errors,
    )?;
    let id = identifier(map.get("id"), "$.application.id", errors);
    let version = metadata(map.get("version"), "$.application.version", errors);
    let source = source(map.get("source"), errors);
    Some(ApplicationRequirement {
        id: id?,
        version: version?,
        source: source?,
    })
}

fn source(value: Option<&Value>, errors: &mut SpecErrors) -> Option<SourceRequirement> {
    let map = object(
        value,
        "$.application.source",
        &["size", "sha256", "architecture"],
        &[],
        errors,
    )?;
    let size = size(map.get("size"), "$.application.source.size", errors);
    let sha256 = digest(map.get("sha256"), "$.application.source.sha256", errors);
    let architecture = literal(
        map.get("architecture"),
        "$.application.source.architecture",
        "x86_64",
        Code::ValueUnsupported,
        errors,
    );
    architecture?;
    Some(SourceRequirement {
        size: size?,
        sha256: sha256?,
        architecture: SourceArchitecture::X86_64,
    })
}

fn runtime(value: Option<&Value>, errors: &mut SpecErrors) -> Option<RuntimeRequirement> {
    let map = object(value, "$.runtime", &["family", "artifacts"], &[], errors)?;
    let family = literal(
        map.get("family"),
        "$.runtime.family",
        "wine",
        Code::ValueUnsupported,
        errors,
    );
    let list = array(map.get("artifacts"), "$.runtime.artifacts", 1, 16, errors);
    let mut artifacts = Vec::new();
    let mut roles = Vec::new();
    for (index, value) in list?.iter().enumerate() {
        let location = format!("$.runtime.artifacts[{index}]");
        if let Some(map) = object(
            Some(value),
            &location,
            &["role", "size", "sha256", "label"],
            &["label"],
            errors,
        ) {
            let role = unique_role(
                map,
                &location,
                &mut roles,
                Code::DuplicateRuntimeArtifact,
                errors,
            );
            let size = size(map.get("size"), &format!("{location}.size"), errors);
            let sha256 = digest(map.get("sha256"), &format!("{location}.sha256"), errors);
            let label = metadata(map.get("label"), &format!("{location}.label"), errors);
            if let (Some(role), Some(size), Some(sha256)) = (role, size, sha256) {
                artifacts.push(ArtifactRequirement {
                    role,
                    size,
                    sha256,
                    label,
                });
            }
        }
    }
    family?;
    Some(RuntimeRequirement {
        family: RuntimeFamily::Wine,
        artifacts,
    })
}

fn unique_role(
    map: &Object,
    location: &str,
    roles: &mut Vec<Identifier>,
    code: Code,
    errors: &mut SpecErrors,
) -> Option<Identifier> {
    let location = format!("{location}.role");
    let role = identifier(map.get("role"), &location, errors)?;
    if roles.contains(&role) {
        errors.push(code, &location);
    } else {
        roles.push(role.clone());
    }
    Some(role)
}

fn environment(value: Option<&Value>, errors: &mut SpecErrors) -> Option<EnvironmentRequirement> {
    let map = object(
        value,
        "$.environment",
        &["windows_architecture", "prefix", "disabled_dlls"],
        &[],
        errors,
    )?;
    let architecture = literal(
        map.get("windows_architecture"),
        "$.environment.windows_architecture",
        "win64",
        Code::ValueUnsupported,
        errors,
    );
    let prefix = object(
        map.get("prefix"),
        "$.environment.prefix",
        &["role"],
        &[],
        errors,
    )
    .and_then(|map| {
        literal(
            map.get("role"),
            "$.environment.prefix.role",
            "dedicated",
            Code::ValueUnsupported,
            errors,
        )
    });
    let list = array(
        map.get("disabled_dlls"),
        "$.environment.disabled_dlls",
        0,
        8,
        errors,
    );
    let mut disabled_dlls = Vec::new();
    for (index, value) in list?.iter().enumerate() {
        let location = format!("$.environment.disabled_dlls[{index}]");
        if let Some(name) = text(
            Some(value),
            &location,
            Code::DllInvalid,
            |s| {
                (1..=64).contains(&s.len())
                    && s.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
                    && s.bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            },
            errors,
        ) {
            if disabled_dlls.iter().any(|s| s == name) {
                errors.push(Code::DuplicateDll, &location);
            } else {
                disabled_dlls.push(name.to_owned());
            }
        }
    }
    architecture?;
    prefix?;
    Some(EnvironmentRequirement {
        windows_architecture: WindowsArchitecture::Win64,
        prefix_role: PrefixRole::Dedicated,
        disabled_dlls,
    })
}

fn entry_point(value: Option<&Value>, errors: &mut SpecErrors) -> Option<EntryPointRequirement> {
    let map = object(
        value,
        "$.entry_point",
        &["path", "sha256"],
        &["sha256"],
        errors,
    )?;
    let path = text(
        map.get("path"),
        "$.entry_point.path",
        Code::PathUnsafe,
        safe_path,
        errors,
    );
    let sha256 = digest(map.get("sha256"), "$.entry_point.sha256", errors);
    Some(EntryPointRequirement {
        path: RelativeEntryPoint(path?.to_owned()),
        sha256,
    })
}

fn safe_path(path: &str) -> bool {
    if path.len() > 1024 || !path.starts_with("drive_c/") {
        return false;
    }
    let parts = path.split('/');
    if parts.clone().count() > 32 {
        return false;
    }
    parts.into_iter().all(|part| {
        let stem = part
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end()
            .to_ascii_lowercase();
        let device = matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
            || ((stem.starts_with("com") || stem.starts_with("lpt"))
                && stem.len() == 4
                && stem
                    .as_bytes()
                    .last()
                    .is_some_and(|b| (b'1'..=b'9').contains(b)));
        (1..=255).contains(&part.len())
            && !matches!(part, "." | "..")
            && !part.starts_with(' ')
            && !part.ends_with([' ', '.'])
            && !device
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._- ".contains(&b))
    })
}

fn verification(
    value: Option<&Value>,
    errors: &mut SpecErrors,
) -> Option<Vec<VerificationDefinitionRef>> {
    let map = object(value, "$.verification", &["definitions"], &[], errors)?;
    let list = array(
        map.get("definitions"),
        "$.verification.definitions",
        1,
        8,
        errors,
    )?;
    let mut definitions = Vec::new();
    let mut roles = Vec::new();
    for (index, value) in list.iter().enumerate() {
        let location = format!("$.verification.definitions[{index}]");
        if let Some(map) = object(
            Some(value),
            &location,
            &["role", "size", "sha256"],
            &[],
            errors,
        ) {
            let role = unique_role(
                map,
                &location,
                &mut roles,
                Code::DuplicateVerificationDefinition,
                errors,
            );
            let size = size(map.get("size"), &format!("{location}.size"), errors);
            let sha256 = digest(map.get("sha256"), &format!("{location}.sha256"), errors);
            if let (Some(role), Some(size), Some(sha256)) = (role, size, sha256) {
                definitions.push(VerificationDefinitionRef { role, size, sha256 });
            }
        }
    }
    Some(definitions)
}

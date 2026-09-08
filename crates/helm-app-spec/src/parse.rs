//! Bounded JSON tree, before schema validation. Never deserialize straight into
//! Value: its ordinary object visitor overwrites duplicate decoded map keys.
use crate::{ErrorCode, SpecErrors};
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use std::{collections::BTreeMap, fmt};

// A private tree also avoids serde_json::Map's optional preserve_order feature:
// downstream feature unification must not introduce randomized map hashing here.
pub(crate) enum Value {
    Other,
    String(String),
    Size(u64),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Value {
    pub(crate) fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
        if let Self::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub(crate) fn as_array(&self) -> Option<&[Value]> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub(crate) fn as_str(&self) -> Option<&str> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub(crate) fn as_u64(&self) -> Option<u64> {
        if let Self::Size(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

struct Budget {
    nodes: usize,
    failure: Option<ErrorCode>,
}
impl Budget {
    fn fail<E: Error>(&mut self, code: ErrorCode) -> E {
        self.failure = Some(code);
        E::custom("bounded JSON rejected")
    }
}

struct Seed<'a> {
    budget: &'a mut Budget,
    depth: usize,
}

pub(crate) fn json(bytes: &[u8]) -> Result<Value, SpecErrors> {
    let mut budget = Budget {
        nodes: 0,
        failure: None,
    };
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let result = Seed {
        budget: &mut budget,
        depth: 0,
    }
    .deserialize(&mut parser);
    let value = result
        .map_err(|_| SpecErrors::one(budget.failure.unwrap_or(ErrorCode::JsonInvalid), "$"))?;
    parser
        .end()
        .map_err(|_| SpecErrors::one(ErrorCode::JsonInvalid, "$"))?;
    Ok(value)
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, parser: D) -> Result<Value, D::Error> {
        if self.depth > 8 || self.budget.nodes == 256 {
            return Err(self.budget.fail(ErrorCode::JsonLimit));
        }
        self.budget.nodes += 1;
        parser.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON")
    }
    fn visit_unit<E: Error>(self) -> Result<Value, E> {
        Ok(Value::Other)
    }
    fn visit_bool<E: Error>(self, _value: bool) -> Result<Value, E> {
        Ok(Value::Other)
    }
    fn visit_u64<E: Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Size(value))
    }
    fn visit_i64<E: Error>(self, _value: i64) -> Result<Value, E> {
        Ok(Value::Other)
    }
    fn visit_f64<E: Error>(self, _value: f64) -> Result<Value, E> {
        Ok(Value::Other)
    }
    fn visit_str<E: Error>(self, value: &str) -> Result<Value, E> {
        if value.len() > 2048 {
            return Err(self.budget.fail(ErrorCode::JsonLimit));
        }
        Ok(Value::String(value.to_owned()))
    }
    fn visit_map<M: MapAccess<'de>>(self, mut input: M) -> Result<Value, M::Error> {
        let mut object = BTreeMap::new();
        // Do not reserve from an untrusted size_hint or overwrite duplicate keys.
        while let Some(key) = input.next_key::<String>()? {
            if key.len() > 80 || object.len() == 16 {
                return Err(self.budget.fail(ErrorCode::JsonLimit));
            }
            if object.contains_key(&key) {
                return Err(self.budget.fail(ErrorCode::FieldDuplicate));
            }
            let value = input.next_value_seed(Seed {
                budget: self.budget,
                depth: self.depth + 1,
            })?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }
    fn visit_seq<S: SeqAccess<'de>>(self, mut input: S) -> Result<Value, S::Error> {
        let mut array = Vec::new();
        loop {
            // Detect an extra element without allocating/deserializing its subtree.
            let depth = if array.len() == 16 { 9 } else { self.depth + 1 };
            match input.next_element_seed(Seed {
                budget: self.budget,
                depth,
            })? {
                Some(value) => array.push(value),
                None => return Ok(Value::Array(array)),
            }
        }
    }
}

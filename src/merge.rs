use crate::{Path, PathSegment, PathTarget, Value};
use anyhow::{bail, ensure, Context};
use std::collections::{btree_map::Entry as MapEntry, BTreeMap};
use tracing::*;

pub fn merge(xs: Vec<(Path, PathTarget)>, strict: bool) -> anyhow::Result<Value> {
    let mut ret = Value::Null;
    for (path, target) in xs {
        let target = Value::try_from(target)?;
        ret.set(&path, target, strict).context(path)?;
    }
    Ok(ret)
}

enum JsonEntry<'a> {
    Object(MapEntry<'a, String, Value>),
    Array(&'a mut Vec<Value>, usize),
}
impl<'a> JsonEntry<'a> {
    fn or_insert_with(self, f: impl FnOnce() -> Value) -> &'a mut Value {
        match self {
            JsonEntry::Object(entry) => entry.or_insert_with(f),
            JsonEntry::Array(arr, idx) => {
                if idx == arr.len() {
                    arr.push(f());
                }
                &mut arr[idx]
            }
        }
    }

    fn insert(self, value: Value) -> (&'a mut Value, Option<Value>) {
        match self {
            JsonEntry::Object(MapEntry::Vacant(entry)) => (entry.insert(value), None),
            JsonEntry::Object(MapEntry::Occupied(mut entry)) => {
                let old = entry.insert(value);
                (entry.into_mut(), Some(old))
            }
            JsonEntry::Array(arr, idx) => {
                if idx == arr.len() {
                    arr.push(value);
                    (&mut arr[idx], None)
                } else {
                    let old = std::mem::replace(&mut arr[idx], value);
                    (&mut arr[idx], Some(old))
                }
            }
        }
    }
}

impl Value {
    fn index(&mut self, segment: &PathSegment, strict: bool) -> anyhow::Result<JsonEntry> {
        match segment {
            PathSegment::Object(key) => {
                if !matches!(self, Value::Object(_)) {
                    if strict {
                        bail!(
                            "Tried to index into an object with key \
                            {key}, but the value there was {self}"
                        );
                    } else {
                        warn!("Discarding old value");
                        *self = Value::Object(BTreeMap::new());
                    }
                }
                let Value::Object(map) = self else { unreachable!() };
                Ok(JsonEntry::Object(map.entry(key.clone())))
            }
            PathSegment::Array(idx) => {
                if !matches!(self, Value::Array(_)) {
                    if strict {
                        bail!(
                            "Tried to index into an array with idx \
                            {idx}, but found {self}"
                        );
                    } else {
                        warn!("Discarding old value");
                        *self = Value::Array(Vec::new());
                    }
                }
                let Value::Array(arr) = self else { unreachable!() };
                Ok(JsonEntry::Array(arr, *idx))
            }
        }
    }
}

impl Value {
    pub fn set(&mut self, path: &Path, value: Value, strict: bool) -> anyhow::Result<()> {
        if path.0.is_empty() {
            if strict {
                ensure!(self == &Value::Null);
            }
            *self = value;
            return Ok(());
        }
        let mut segments = path.0.iter();
        let mut ptr = if self == &Value::Null {
            match segments.next().unwrap() {
                PathSegment::Object(first) => {
                    *self = Value::Object(BTreeMap::new());
                    let Value::Object(map) = self else { unreachable!() };
                    JsonEntry::Object(map.entry(first.clone()))
                }
                PathSegment::Array(first) => {
                    *self = Value::Array(Vec::new());
                    let Value::Array(arr) = self else { unreachable!() };
                    JsonEntry::Array(arr, *first)
                }
            }
        } else {
            self.index(segments.next().unwrap(), strict)?
        };
        for segment in segments {
            let val = ptr.or_insert_with(|| match segment {
                PathSegment::Object(_) => Value::Object(BTreeMap::new()),
                PathSegment::Array(_) => Value::Array(Vec::new()),
            });
            ptr = val.index(segment, strict)?;
        }
        if let (new, Some(old)) = ptr.insert(value) {
            if &old != new {
                if strict {
                    bail!(
                        "{path}: Saw multiple contradictory assignments: \
                        {old} and {new}"
                    );
                } else {
                    warn!("Discarding old value");
                }
            }
        }
        Ok(())
    }
}

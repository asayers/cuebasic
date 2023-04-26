use crate::{Path, PathSegment, PathTarget};
use anyhow::{bail, ensure};
use serde_json::Value;
use tracing::*;

enum JsonEntry<'a> {
    Object(serde_json::map::Entry<'a>),
    Array(&'a mut Vec<Value>, usize),
}
impl<'a> JsonEntry<'a> {
    fn or_insert(self, x: Value) -> &'a mut Value {
        match self {
            JsonEntry::Object(entry) => entry.or_insert(x),
            JsonEntry::Array(arr, idx) => {
                if idx == arr.len() {
                    arr.push(x);
                }
                &mut arr[idx]
            }
        }
    }
}

pub fn merge(xs: Vec<(Path, PathTarget)>) -> anyhow::Result<Value> {
    use serde_json::{map::Entry, Map};
    let mut ret = Map::new();
    for (path, target) in xs {
        let mut segments = path.0.iter();
        let PathSegment::Object(first) = segments.next().unwrap() else {
            bail!("top-level must be an object");
        };
        let mut ptr = JsonEntry::Object(ret.entry(first));
        for segment in segments {
            match segment {
                PathSegment::Object(key) => {
                    let x = ptr.or_insert(Value::Object(Map::new()));
                    let Value::Object(x) = x else {
                        bail!(
                            "{path}: Tried to index into a value with key \
                            {key}, but the value there was {x}"
                        );
                    };
                    ptr = JsonEntry::Object(x.entry(key));
                }
                PathSegment::Array(idx) => {
                    let x = ptr.or_insert(Value::Array(Vec::new()));
                    let arr = x.as_array_mut().unwrap();
                    ptr = JsonEntry::Array(arr, *idx);
                }
            }
        }
        match target {
            PathTarget::Value(target) => match ptr {
                JsonEntry::Object(Entry::Vacant(ptr)) => {
                    ptr.insert(target);
                }
                JsonEntry::Object(Entry::Occupied(ptr)) => {
                    let existing = ptr.get();
                    ensure!(
                        existing == &target,
                        "{path}: Saw multiple contradictory assignments: \
                        {existing} and {target}"
                    );
                }
                JsonEntry::Array(arr, idx) => {
                    if idx == arr.len() {
                        arr.push(target);
                    } else {
                        ensure!(&arr[idx] == &target);
                    }
                }
            },
            PathTarget::Ref(_) => error!("TODO"),
        }
    }
    Ok(Value::Object(ret))
}

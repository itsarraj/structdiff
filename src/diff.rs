use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Diff {
    Added {
        path: String,
        value: Value,
    },
    Removed {
        path: String,
        value: Value,
    },
    Changed {
        path: String,
        old: Value,
        new: Value,
    },
}

fn join_path(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

/// Semantically diffs `old` against `new`, walking objects and arrays
/// by key/index and reporting `path`s in `a.b.c` / `a[2]` notation —
/// the point being a reordered object (same keys, different insertion
/// order) or reformatted whitespace produces **no** diff at all, unlike
/// a text-based `diff` on two pretty-printed JSON files.
///
/// A key or index present on only one side is reported as a single
/// `Added`/`Removed` entry holding its *entire* value, without
/// recursing further into it — adding a whole nested object shows up
/// as one entry, not a wall of individual leaf-level entries for every
/// field inside it.
pub fn diff(old: &Value, new: &Value) -> Vec<Diff> {
    let mut out = Vec::new();
    diff_at("", old, new, &mut out);
    out
}

fn diff_at(path: &str, old: &Value, new: &Value, out: &mut Vec<Diff>) {
    match (old, new) {
        (Value::Object(o), Value::Object(n)) => {
            let mut keys: Vec<&String> = o.keys().chain(n.keys()).collect();
            keys.sort();
            keys.dedup();
            for key in keys {
                let child_path = join_path(path, key);
                match (o.get(key), n.get(key)) {
                    (Some(ov), Some(nv)) => diff_at(&child_path, ov, nv, out),
                    (Some(ov), None) => out.push(Diff::Removed {
                        path: child_path,
                        value: ov.clone(),
                    }),
                    (None, Some(nv)) => out.push(Diff::Added {
                        path: child_path,
                        value: nv.clone(),
                    }),
                    (None, None) => unreachable!(),
                }
            }
        }
        (Value::Array(o), Value::Array(n)) => {
            for i in 0..o.len().max(n.len()) {
                let child_path = format!("{path}[{i}]");
                match (o.get(i), n.get(i)) {
                    (Some(ov), Some(nv)) => diff_at(&child_path, ov, nv, out),
                    (Some(ov), None) => out.push(Diff::Removed {
                        path: child_path,
                        value: ov.clone(),
                    }),
                    (None, Some(nv)) => out.push(Diff::Added {
                        path: child_path,
                        value: nv.clone(),
                    }),
                    (None, None) => unreachable!(),
                }
            }
        }
        _ => {
            if old != new {
                out.push(Diff::Changed {
                    path: path.to_string(),
                    old: old.clone(),
                    new: new.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identical_values_produce_no_diff() {
        let v = json!({"a": 1, "b": [1, 2, {"c": true}]});
        assert!(diff(&v, &v).is_empty());
    }

    #[test]
    fn reordered_object_keys_produce_no_diff() {
        let old = json!({"a": 1, "b": 2});
        let new = json!({"b": 2, "a": 1});
        assert!(diff(&old, &new).is_empty());
    }

    #[test]
    fn added_object_key_is_a_single_entry_not_recursed() {
        let old = json!({"a": 1});
        let new = json!({"a": 1, "b": {"c": 1, "d": 2}});
        let findings = diff(&old, &new);
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], Diff::Added { path, .. } if path == "b"));
    }

    #[test]
    fn removed_object_key_is_reported() {
        let old = json!({"a": 1, "b": 2});
        let new = json!({"a": 1});
        let findings = diff(&old, &new);
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], Diff::Removed { path, .. } if path == "b"));
    }

    #[test]
    fn nested_object_path_uses_dot_notation() {
        let old = json!({"user": {"address": {"city": "NYC"}}});
        let new = json!({"user": {"address": {"city": "LA"}}});
        let findings = diff(&old, &new);
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], Diff::Changed { path, .. } if path == "user.address.city"));
    }

    #[test]
    fn array_element_change_uses_bracket_notation() {
        let old = json!({"items": [1, 2, 3]});
        let new = json!({"items": [1, 99, 3]});
        let findings = diff(&old, &new);
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], Diff::Changed { path, .. } if path == "items[1]"));
    }

    #[test]
    fn array_grown_reports_new_indices_as_added() {
        let old = json!([1, 2]);
        let new = json!([1, 2, 3]);
        let findings = diff(&old, &new);
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], Diff::Added { path, .. } if path == "[2]"));
    }

    #[test]
    fn array_shrunk_reports_missing_indices_as_removed() {
        let old = json!([1, 2, 3]);
        let new = json!([1, 2]);
        let findings = diff(&old, &new);
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], Diff::Removed { path, .. } if path == "[2]"));
    }

    #[test]
    fn a_type_change_at_the_same_key_is_one_changed_entry() {
        let old = json!({"a": {"nested": true}});
        let new = json!({"a": "now a string"});
        let findings = diff(&old, &new);
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], Diff::Changed { path, .. } if path == "a"));
    }
}

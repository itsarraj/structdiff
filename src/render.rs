use serde_json::Value;

use crate::diff::Diff;

fn preview(value: &Value) -> String {
    match value {
        Value::String(s) => format!("{s:?}"),
        other => other.to_string(),
    }
}

pub fn render(findings: &[Diff]) -> String {
    let mut lines: Vec<String> = findings
        .iter()
        .map(|f| match f {
            Diff::Added { path, value } => format!("+ {path} = {}", preview(value)),
            Diff::Removed { path, value } => format!("- {path} = {}", preview(value)),
            Diff::Changed { path, old, new } => {
                format!("~ {path}: {} -> {}", preview(old), preview(new))
            }
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renders_each_diff_kind_with_its_own_marker() {
        let findings = vec![
            Diff::Added {
                path: "b".to_string(),
                value: json!(1),
            },
            Diff::Removed {
                path: "c".to_string(),
                value: json!(2),
            },
            Diff::Changed {
                path: "a".to_string(),
                old: json!(1),
                new: json!(2),
            },
        ];
        let text = render(&findings);
        assert!(text.contains("+ b = 1"));
        assert!(text.contains("- c = 2"));
        assert!(text.contains("~ a: 1 -> 2"));
    }

    #[test]
    fn strings_are_shown_quoted_to_distinguish_from_numbers() {
        let findings = vec![Diff::Changed {
            path: "name".to_string(),
            old: json!("1"),
            new: json!(1),
        }];
        let text = render(&findings);
        assert!(text.contains("~ name: \"1\" -> 1"));
    }
}

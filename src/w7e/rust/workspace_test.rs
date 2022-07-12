#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use log::{debug, error};
    use serde::Serialize;

    use crate::experiments::pretty_ron::ToPrettyRonString;
    use crate::LangId;
    use crate::w7e::project_scope::SerializableProjectScope;
    use crate::w7e::workspace::SerializableWorkspace;

    #[test]
    fn test_write_rust_workspace() {
        let workspace_pill = SerializableWorkspace {
            scopes: vec![
                SerializableProjectScope {
                    lang_id: LangId::RUST,
                    path: PathBuf::from("/home/someuser/rust_repo"),
                    handler_id_op: Some("rust_cargo".to_string()),
                }
            ]
        };

        let item = workspace_pill.to_pretty_ron_string().unwrap();

        assert_eq!(item, r#"(
    scopes: [
        (
            lang_id: RUST,
            path: "/home/someuser/rust_repo",
            handler_id_op: Some("rust_cargo"),
        ),
    ],
)"#);
    }
}
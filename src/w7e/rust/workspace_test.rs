#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use serde::Serialize;

    use crate::experiments::pretty_ron::ToPrettyRonString;
    use crate::LangId;
    use crate::new_fs::mock_fs::MockFS;
    use crate::w7e::project_scope::SerializableProjectScope;
    use crate::w7e::workspace::{SerializableWorkspace, Workspace};

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
            path: "rust_repo",
            handler_id_op: Some("rust_cargo"),
        ),
    ],
)"#);
    }

    #[test]
    fn test_read_workspace() {
        let repo_folder = Path::new("workspace");
        let mock_fs = MockFS::new("/tmp").with_file(
            &repo_folder.join(crate::w7e::workspace::WORKSPACE_FILE_NAME),
            r#"(
    scopes: [
        (
            lang_id: RUST,
            path: "rust_repo",
            handler_id_op: Some("rust_cargo"),
        ),
    ],
)"#).to_fsf();

        let path = mock_fs.descendant_checked(&repo_folder).unwrap();
        let repo = Workspace::try_load(path).unwrap();
    }
}
#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use serde::Serialize;

    use crate::experiments::pretty_ron::ToPrettyRonString;
    use crate::{LangId, FilesystemFront, spath};
    use crate::new_fs::mock_fs::MockFS;
    use crate::w7e::project_scope::SerializableProjectScope;
    use crate::w7e::workspace::{ScopeLoadErrors, SerializableWorkspace, Workspace, WORKSPACE_FILE_NAME};
    use crate::w7e::workspace::LoadError::ScopeLoadError;

    #[test]
    fn test_write_rust_workspace() {
        let workspace_pill = SerializableWorkspace {
            scopes: vec![
                SerializableProjectScope {
                    lang_id: LangId::RUST,
                    path: PathBuf::from("rust_repo"),
                    handler_id_op: Some("rust".to_string()),
                }
            ]
        };

        let item = workspace_pill.to_pretty_ron_string().unwrap();

        assert_eq!(item, r#"(
    scopes: [
        (
            lang_id: RUST,
            path: "rust_repo",
            handler_id_op: Some("rust"),
        ),
    ],
)"#);
    }

    #[test]
    fn test_read_rust_workspace() {
        let workspace = r#"(
    scopes: [
        (
            lang_id: RUST,
            path: "rust_repo",
            handler_id_op: Some("rust"),
        ),
    ],
)
        "#;

        let workspace_pill = ron::from_str::<SerializableWorkspace>(workspace).unwrap();

        assert_eq!(workspace_pill.scopes.len(), 1);
        assert_eq!(workspace_pill.scopes[0].lang_id, LangId::RUST);
        assert_eq!(workspace_pill.scopes[0].path, PathBuf::from("rust_repo"));
        assert_eq!(workspace_pill.scopes[0].handler_id_op, Some("rust".to_string()));
    }

    #[tokio::test]
    async fn test_read_workspace() {
        let repo_folder = Path::new("workspace");
        let mock_fs = MockFS::new("/tmp").with_file(
            "workspace/.gladius_workspace.ron",
            r#"(
    scopes: [
        (
            lang_id: RUST,
            path: "rust_project",
            handler_id_op: Some("rust"),
        ),
    ],
)"#).with_file(
            "workspace/rust_project/Cargo.toml",
            r#"
[package]
name = "hello_world" # the name of the package
version = "0.1.0"    # the current version, obeying semver
authors = ["Alice <a@example.com>", "Bob <b@example.com>"]
            "#).to_fsf();

        assert_eq!(spath!(mock_fs, "workspace").unwrap().exists(), true);
        assert_eq!(spath!(mock_fs, "workspace", ".gladius_workspace.ron").unwrap().exists(), true);
        assert_eq!(spath!(mock_fs, "workspace", "rust_project").unwrap().exists(), true);
        assert_eq!(spath!(mock_fs, "workspace", "rust_project", "Cargo.toml").unwrap().exists(), true);

        let path = mock_fs.descendant_checked(&repo_folder).unwrap();
        let (workspace, errors) = Workspace::try_load(path).unwrap();

        assert_eq!(errors, ScopeLoadErrors::default());
    }
}
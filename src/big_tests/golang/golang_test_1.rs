use std::time::Duration;

use crate::mocks::full_setup::FullSetup;
use crate::mocks::with_wait_for::WithWaitFor;

fn get_full_setup(file: &str) -> FullSetup {
    let full_setup: FullSetup = FullSetup::new("./test_envs/golang_1")
        .with_files([file])
        .with_mock_navcomp(false)
        .with_timeout(Duration::from_secs(3))
        .build();

    full_setup
}

#[test]
fn golang_loads() {
    if std::env::var("GITLAB").is_ok() {
        return;
    }

    let mut full_setup = get_full_setup("main.go");
    assert!(full_setup.wait_for(|f| f.is_editor_opened()));
}

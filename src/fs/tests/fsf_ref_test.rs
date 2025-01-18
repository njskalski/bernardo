use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::spath;

#[test]
fn spath_macro() {
    let mockfs = MockFS::new("/").to_fsf();
    let _sp0 = spath!(mockfs);
    let _sp1 = spath!(mockfs, "a");
    let _sp2 = spath!(mockfs, "a", "b");
}

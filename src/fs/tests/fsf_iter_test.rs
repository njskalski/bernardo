use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_iter::RecursiveFsIter;
use crate::fs::mock_fs::MockFS;
use crate::spath;

#[test]
fn test_all_iter() {
    let m = MockFS::new("/tmp")
        .with_file("folder1/folder2/file1.txt", "some text")
        .with_file("folder1/folder3/moulder.txt", "truth is out there")
        .to_fsf();

    let mut iter = RecursiveFsIter::new(m.root());

    assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2", "file1.txt").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_hidden_dir_is_ignored() {
    let m = MockFS::new("/tmp")
        .with_file("folder1/folder2/file1.txt", "some text")
        .with_file("folder1/.git/moulder.txt", "truth is out there")
        .to_fsf();

    let mut iter = RecursiveFsIter::new(m.root());

    assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2", "file1.txt").unwrap()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_root_gitignore_is_respected() {
    let m = MockFS::new("/tmp")
        .with_file(".gitignore", "file1.txt")
        .with_file("folder1/folder2/file1.txt", "not matched")
        .with_file("folder1/folder3/moulder.txt", "truth is out there")
        .to_fsf();

    let mut iter = RecursiveFsIter::new(m.root());

    assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_nested_gitignore_is_respected() {
    let m = MockFS::new("/tmp")
        .with_file("folder1/folder2/.gitignore", "file1.txt")
        .with_file("folder1/folder2/file1.txt", "not matched")
        .with_file("folder1/folder3/moulder.txt", "truth is out there")
        .to_fsf();

    let mut iter = RecursiveFsIter::new(m.root());

    assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_nested_gitignore_does_not_affect_other_files() {
    let m = MockFS::new("/tmp")
        .with_file("folder1/folder2/.gitignore", "file1.txt")
        .with_file("folder1/folder2/file1.txt", "not matched")
        .with_file("folder1/folder3/file1.txt", "matched")
        .to_fsf();

    let mut iter = RecursiveFsIter::new(m.root());

    assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "file1.txt").unwrap()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_multiple_gitignores_are_respected() {
    let m = MockFS::new("/tmp")
        .with_file(".gitignore", "file1.txt")
        .with_file("folder1/folder2/.gitignore", "file2.txt")
        .with_file("folder1/folder2/file1.txt", "not matched")
        .with_file("folder1/folder2/file2.txt", "not matched")
        .with_file("folder1/folder3/file1.txt", "not matched")
        .with_file("folder1/folder3/file2.txt", "matched")
        .with_file("folder1/folder3/moulder.txt", "truth is out there")
        .to_fsf();

    let mut iter = RecursiveFsIter::new(m.root());

    assert_eq!(iter.next(), Some(spath!(m, "folder1").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder2").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "file2.txt").unwrap()));
    assert_eq!(iter.next(), Some(spath!(m, "folder1", "folder3", "moulder.txt").unwrap()));
    assert_eq!(iter.next(), None);
}

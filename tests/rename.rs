use std::{fs, path::Path};

use nomi::rename::{self, Entry, MatchMode, RenameOp};

fn entry(name: &str) -> Entry {
    Entry {
        name: name.into(),
        selected: true,
        is_dir: false,
    }
}

#[test]
fn regex_preview_expands_captures() {
    let dir = Path::new("/work");
    let preview = rename::build_preview(
        dir,
        &[entry("IMG_001.jpg")],
        r"^IMG_(\d+)\.jpg$",
        "holiday_$1.jpg",
        MatchMode::Regex,
    );
    assert_eq!(preview.names, vec![Some("holiday_001.jpg".into())]);
    assert!(preview.error.is_none());
}

#[test]
fn literal_preview_replaces_all_matches() {
    let preview = rename::build_preview(
        Path::new("/work"),
        &[entry("foo-foo.txt")],
        "foo",
        "bar",
        MatchMode::Literal,
    );
    assert_eq!(preview.names, vec![Some("bar-bar.txt".into())]);
}

#[test]
fn rejects_duplicate_destinations() {
    let dir = Path::new("/work");
    let preview = rename::build_preview(
        dir,
        &[entry("a1.txt"), entry("a2.txt")],
        r"a\d",
        "same",
        MatchMode::Regex,
    );
    assert!(preview.error.unwrap().contains("would become"));
}

#[test]
fn execute_supports_swaps() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("a.txt"), "A").unwrap();
    fs::write(temp.path().join("b.txt"), "B").unwrap();
    let operations = vec![
        RenameOp {
            from: temp.path().join("a.txt"),
            to: temp.path().join("b.txt"),
        },
        RenameOp {
            from: temp.path().join("b.txt"),
            to: temp.path().join("a.txt"),
        },
    ];
    rename::execute(temp.path(), &operations).unwrap();
    assert_eq!(fs::read_to_string(temp.path().join("a.txt")).unwrap(), "B");
    assert_eq!(fs::read_to_string(temp.path().join("b.txt")).unwrap(), "A");
}

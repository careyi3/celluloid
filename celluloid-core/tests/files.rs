//! Saving and loading animation files.

use celluloid_core::{from_json, Recorder, FORMAT_VERSION};

fn small() -> Recorder {
    let mut rec = Recorder::new("small");
    let g = rec.grid("map", 4, 3);
    rec.state(g, "wall", "#555555");
    rec.set(g, (1, 1), "wall");
    rec.marker(g, "me", (0, 0));
    rec.var("steps", 0);
    rec.frame("start");
    rec.marker(g, "me", (1, 0));
    rec.var("steps", 1);
    rec.bookmark("moved");
    rec.frame("step");
    rec
}

#[test]
fn save_then_load() {
    let path = std::env::temp_dir().join(format!("celluloid-{}.json", std::process::id()));
    small().save(&path).unwrap();
    let json = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(&path).unwrap();

    let loaded = from_json(&json).unwrap();
    let mut expected = small().finish();
    expected.created_at.clone_from(&loaded.created_at);
    assert_eq!(loaded, expected);
    assert_eq!(loaded.bookmarks().collect::<Vec<_>>(), [(1, "moved")]);
}

#[test]
fn rejects_other_json() {
    let err = from_json(r#"{"hello": "world"}"#).unwrap_err();
    assert_eq!(err.to_string(), "not a celluloid animation");
    assert!(from_json("not json").is_err());
}

#[test]
fn rejects_newer_formats() {
    let mut animation = small().finish();
    animation.format = FORMAT_VERSION + 1;
    let json = serde_json::to_string(&animation).unwrap();
    assert!(from_json(&json).unwrap_err().to_string().contains("format"));
}

#[test]
fn rejects_ops_that_point_nowhere() {
    let mut animation = small().finish();
    animation.panels.clear();
    let json = serde_json::to_string(&animation).unwrap();
    assert!(from_json(&json).is_err());
}

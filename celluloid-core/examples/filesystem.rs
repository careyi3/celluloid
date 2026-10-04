//! cargo run -p celluloid-core --example filesystem
//! then: celluloid filesystem.json
//!
//! Replays a terminal session (Advent of Code 2022 day 7 style) into a
//! tree of directories and files, then totals directory sizes.

use celluloid_core::Recorder;
use std::collections::HashMap;

const SESSION: &str = "\
$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

pub fn record() -> Recorder {
    let mut rec = Recorder::new("Filesystem");
    rec.frame_delay(300.0);
    let t = rec.tree("disk");
    rec.state(t, "dir", "#4e79a7");
    rec.state(t, "file", "#6b7280");
    rec.state(t, "small", "#59a14f");

    let mut cwd = String::from("/");
    let mut files: Vec<(String, u64)> = Vec::new();
    rec.set(t, "/", "dir");
    rec.label(t, "/", "/");
    for line in SESSION.lines() {
        match line.split_whitespace().collect::<Vec<_>>()[..] {
            ["$", "cd", "/"] => cwd = "/".into(),
            ["$", "cd", ".."] => {
                cwd.pop();
                cwd.truncate(cwd.rfind('/').unwrap() + 1);
            }
            ["$", "cd", dir] => cwd = format!("{cwd}{dir}/"),
            ["$", "ls"] => {}
            ["dir", name] => {
                let dir = format!("{cwd}{name}/");
                rec.add_child(t, &cwd, &dir);
                rec.set(t, &dir, "dir");
                rec.label(t, &dir, name);
            }
            [size, name] => {
                let file = format!("{cwd}{name}");
                let size: u64 = size.parse().unwrap();
                rec.add_child(t, &cwd, &file);
                rec.set(t, &file, "file");
                rec.label(t, &file, format!("{name} {size}"));
                files.push((file, size));
            }
            _ => {}
        }
        rec.marker(t, "cwd", &cwd);
        rec.frame(line);
    }
    rec.hide_marker(t, "cwd");

    let mut sizes: HashMap<String, u64> = HashMap::new();
    for (file, size) in &files {
        for (i, _) in file.match_indices('/') {
            *sizes.entry(file[..=i].to_string()).or_default() += size;
        }
    }
    let mut dirs: Vec<_> = sizes.into_iter().collect();
    dirs.sort();
    let mut total_small = 0;
    for (dir, size) in dirs {
        let name = dir.trim_end_matches('/').rsplit('/').next().filter(|n| !n.is_empty());
        rec.label(t, &dir, format!("{} {size}", name.unwrap_or("/")));
        if size <= 100_000 {
            rec.set(t, &dir, "small");
            total_small += size;
        }
        rec.var("small total", total_small);
        rec.frame(format!("{dir} is {size}"));
    }
    rec.bookmark("sizes");
    rec.frame(format!("dirs of at most 100000 total {total_small}"));
    rec
}

#[allow(dead_code)]
fn main() {
    record().save("filesystem.json").unwrap();
    println!("wrote filesystem.json");
}

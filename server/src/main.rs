#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer, NamedFile};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct AnimationList {
    animations: Vec<AnimationInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
struct AnimationInfo {
    name: String,
    file: String,
    algorithm: String,
    created_at: String,
}

type AnimationCache = Mutex<Option<AnimationList>>;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new(relative!("templates")).join("index.html"))
        .await
        .ok()
}

#[get("/animations")]
fn list_animations(cache: &State<AnimationCache>) -> Json<AnimationList> {
    let mut cache = cache.lock().unwrap();

    if cache.is_none() {
        *cache = Some(scan_animations());
    }

    Json(cache.clone().unwrap())
}

#[get("/animation/<name>")]
async fn get_animation(name: String) -> Option<NamedFile> {
    let path = Path::new(relative!("../animations")).join(format!("{}.json", name));
    NamedFile::open(path).await.ok()
}

#[post("/animations/refresh")]
fn refresh_animations(cache: &State<AnimationCache>) -> Json<AnimationList> {
    let mut cache = cache.lock().unwrap();
    *cache = Some(scan_animations());
    Json(cache.clone().unwrap())
}

fn scan_animations() -> AnimationList {
    let animations_dir = Path::new(relative!("../animations"));
    let mut animations = Vec::new();

    if let Ok(entries) = fs::read_dir(animations_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        animations.push(AnimationInfo {
                            name: data["name"].as_str().unwrap_or("Unknown").to_string(),
                            file: path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            algorithm: data["algorithm"].as_str().unwrap_or("unknown").to_string(),
                            created_at: data["created_at"].as_str().unwrap_or("").to_string(),
                        });
                    }
                }
            }
        }
    }

    animations.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    AnimationList { animations }
}

#[catch(404)]
fn not_found() -> String {
    "404 - Page not found".to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(AnimationCache::new(None))
        .mount("/", routes![index])
        .mount(
            "/api",
            routes![list_animations, get_animation, refresh_animations],
        )
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/pkg", FileServer::from(relative!("../pkg")))
        .register("/", catchers![not_found])
}

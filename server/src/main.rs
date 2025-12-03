#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer, NamedFile};
use std::path::Path;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new(relative!("templates")).join("index.html"))
        .await
        .ok()
}

#[catch(404)]
fn not_found() -> String {
    "404 - Page not found".to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/pkg", FileServer::from(relative!("../pkg")))
        .register("/", catchers![not_found])
}

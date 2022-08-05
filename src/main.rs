#[macro_use] extern crate rocket;

mod tera;
mod blog;

use std::process;

use rocket_dyn_templates::Template;

#[get("/")]
fn world() -> &'static str {
    "Hello world!"
}

// Try visiting:
//   http://127.0.0.1:8000/wave/Rocketeer/100
#[get("/<name>/<age>")]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

#[launch]
fn rocket() -> _ {
    if let Err(e) = blog::render_markdown_files("./blog") {
        eprintln!("Error rendering markdown pages: {}", e);
        process::exit(1)
    }
    rocket::build()
        .mount("/", routes![world])
        .mount("/wave", routes![wave])
        .mount("/tera", routes![tera::index, tera::hello, tera::about])
        .register("/tera", catchers![tera::not_found])
        .attach(Template::custom(|engines|{
            tera::customize(&mut engines.tera)
        }))
}

#[macro_use]
extern crate rocket;
use rocket::fs::{FileServer, relative};

mod blog;
mod tera;

use std::process;
use std::env;

use rocket_dyn_templates::Template;

#[launch]
async fn rocket() -> _ {
    let blog_posts_render = blog::render_blog_posts("./blog", "./static");

    match blog_posts_render {
        Err(e) => {
            eprintln!("Error rendering blog pages: {}", e);
            process::exit(1)
        }
        Ok(blog_posts) => rocket::build()
            .manage(blog_posts)
            .mount("/", routes![tera::index, tera::blog, tera::blog_posts])
            .mount("/", FileServer::from(relative!("static")))
            .attach(Template::fairing())
    }
}

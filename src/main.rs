#[macro_use]
extern crate rocket;
use rocket::fs::{relative, FileServer};
use std::path::PathBuf;
use std::sync::Mutex;

mod blog;
mod config;
mod tera;

use std::env;
use std::process;

use rocket_dyn_templates::Template;

#[launch]
async fn rocket() -> _ {
    let blog_posts_result = blog::render_blog_posts("./blog");
    let index_cfg: config::IndexPage = config::read_config(PathBuf::from("./Config.toml")).unwrap();

    match blog_posts_result {
        Err(e) => {
            eprintln!("Error rendering blog pages: {}", e);
            process::exit(1)
        }
        Ok((blog_posts, tags, render_mem)) => rocket::build()
            .manage(blog_posts)
            .manage(Mutex::new(render_mem))
            .manage(index_cfg)
            .manage(tags)
            .mount(
                "/",
                routes![
                    tera::index,
                    tera::blog,
                    tera::blog_posts,
                    tera::publications
                ],
            )
            .mount("/", FileServer::from(relative!("static")))
            .register("/", catchers![tera::not_found])
            .attach(Template::fairing()),
    }
}

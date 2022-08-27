#[macro_use]
extern crate rocket;
use rocket::fs::{relative, FileServer};
use std::sync::Mutex;
use tera::SharedRedis;

mod blog;
mod tera;

use std::env;
use std::process;

use rocket_dyn_templates::Template;

#[launch]
async fn rocket() -> _ {
    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut redis_conn = redis_client.get_connection().unwrap();

    let blog_posts_result = blog::render_blog_posts("./blog", &mut redis_conn);
    let shared_redis = SharedRedis {
        connection: Mutex::new(redis_conn),
    };

    match blog_posts_result {
        Err(e) => {
            eprintln!("Error rendering blog pages: {}", e);
            process::exit(1)
        }
        Ok((blog_posts, tags)) => rocket::build()
            .manage(blog_posts)
            .manage(shared_redis)
            .manage(tags)
            .mount("/", routes![tera::index, tera::blog, tera::blog_posts])
            .mount("/", FileServer::from(relative!("static")))
            .attach(Template::fairing()),
    }
}

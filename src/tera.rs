use std::fs;
use std::io;
use std::path::PathBuf;

use rocket::State;

use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::blog::{BlogPost, BlogPostMetadata};
#[derive(Debug, Serialize)]
struct BlogPostView {
    id: String,
    content: String,
    metadata: BlogPostMetadata
}

// TODO: cow strings?
// TODO: caching
// TODO: use generics
fn read_post(id: &String) -> io::Result<String> {
    let base_path = PathBuf::from("./static/");
    let html_file_path = base_path.join(id.clone() + ".html");
    fs::read_to_string(html_file_path)
}

#[get("/")]
pub fn index(top_posts: &State<Vec<BlogPost>>) -> Template {

    let mut posts: Vec<BlogPostView> = vec![];

    // Sort by date and render top N posts
    for v in top_posts.iter() {
        if let Ok(raw_html) = read_post(&v.id) {
            let mut post_html = String::new();
            post_html += &raw_html[0..500];
            post_html += "...";
            posts.push(BlogPostView { id: v.id.clone(), content: post_html, metadata: v.metadata.clone() });
        } else {
            panic!("Error reading html file.");
        }

    }
    Template::render(
        "index",
        context! {
            posts: posts,
        },
    )
}

#[get("/blog")]
pub fn blog() -> Template {
    Template::render(
        "blog",
        context! {
        },
    )
}

#[get("/blog/<post_id>")]
// TODO: handle errors
pub fn blog_posts(post_id: &str, top_posts: &State<Vec<BlogPost>>) -> Template {
    let f_post = top_posts.first().unwrap();
    let content = read_post(&post_id.to_owned()).unwrap();
    let post = BlogPostView{
        id: String::from(post_id),
        content,
        metadata: f_post.metadata.clone()
    };
    Template::render(
        "blog_post",
        context! {
           post: post,
        },
    )
}

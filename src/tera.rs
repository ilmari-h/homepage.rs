use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Mutex;

use rocket::http::Status;
use rocket::response::status;
use rocket::State;

use crate::blog::{BlogPost, BlogPostMetadata};
use crate::config;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct BlogPostView {
    id: String,
    content: String,
    metadata: BlogPostMetadata,
}

#[derive(Debug, Serialize)]
struct TagView<'a> {
    tag: &'a String,
    selected: bool,
}

#[derive(Debug, Serialize)]
struct NavConfig<'a> {
    navbar_links: &'a Vec<config::NavBarLink>,
}

#[get("/")]
pub fn index(
    blog_posts: &State<Vec<BlogPost>>,
    mtx_render_mem: &State<Mutex<HashMap<String, String>>>,
    cfg: &State<config::IndexPage>,
) -> Result<Template, status::Custom<String>> {
    let mut posts: Vec<BlogPostView> = vec![];
    let render_mem = mtx_render_mem.lock().expect("lock shared data");

    // Posts are sorted by date by default. Pick 3 newest.
    for v in blog_posts.iter().take(3) {
        if let Some(raw_html) = render_mem.get(&v.id) {
            let mut post_html = String::new();

            // Cut the post short on index page.
            post_html += &raw_html[0..500];

            // Cut to the last complete word.
            while post_html.chars().last().unwrap_or(' ') != ' ' {
                post_html.pop();
            }

            posts.push(BlogPostView {
                id: v.id.clone(),
                content: post_html,
                metadata: v.metadata.clone(),
            });
        } else {
            return Err(status::Custom(
                Status::InternalServerError,
                "Error reading html file.".to_string(),
            ));
        }
    }
    Ok(Template::render(
        "index",
        context! {
            posts: posts,
            cfg: cfg.inner()
        },
    ))
}

#[get("/blog?<tags>")]
pub fn blog(
    tags: Option<String>,
    blog_posts: &State<Vec<BlogPost>>,
    blog_tags: &State<Vec<String>>,
    cfg: &State<config::IndexPage>,
) -> Template {
    let mut s_tags: Vec<String> = vec![];
    if let Some(have_tags) = tags {
        s_tags = have_tags.split(' ').map(String::from).collect()
    }
    let posts: Vec<&BlogPost> = blog_posts.iter().collect();
    let mut tags: Vec<TagView> = blog_tags
        .iter()
        .map(|tag| TagView {
            tag,
            selected: s_tags.iter().any(|st| st == tag),
        })
        .collect();
    tags.sort_by(|ta, tb| {
        if ta.selected {
            Ordering::Less
        } else if tb.selected {
            Ordering::Greater
        } else {
            ta.tag.to_lowercase().cmp(&tb.tag.to_lowercase())
        }
    });

    let s_posts: Vec<&BlogPost> = if !s_tags.is_empty() {
        posts
            .iter()
            .filter(|p| {
                if let Some(tags) = &p.metadata.tags {
                    tags.iter().any(|at| s_tags.contains(at))
                } else {
                    false // No tags, can't be a match.
                }
            })
            .copied()
            .collect()
    } else {
        posts
    };

    Template::render(
        "blog",
        context! {
            posts: s_posts,
            cfg: NavConfig{ navbar_links: &cfg.navbar_links},
            tags
        },
    )
}

#[get("/blog/<post_id>")]
pub fn blog_posts(
    post_id: &str,
    blog_posts: &State<Vec<BlogPost>>,
    mtx_render_mem: &State<Mutex<HashMap<String, String>>>,
    cfg: &State<config::IndexPage>,
) -> Option<Template> {
    let f_post = blog_posts.iter().find(|post| post.id == post_id)?;
    let render_mem = mtx_render_mem
        .lock()
        .expect("Couldn't aquire lock while rendering block post.");

    if let Some(content) = render_mem.get(post_id) {
        let post = BlogPostView {
            id: String::from(post_id),
            content: content.to_string(),
            metadata: f_post.metadata.clone(),
        };
        Some(Template::render(
            "blog_post",
            context! {
               post: post,
                cfg: NavConfig{ navbar_links: &cfg.navbar_links},
            },
        ))
    } else {
        None
    }
}

#[get("/publications")]
pub fn publications(cfg: &State<config::IndexPage>) -> Template {
    Template::render(
        "publications",
        context! {
            cfg: cfg.inner()
        },
    )
}

#[catch(404)]
pub fn not_found() -> Template {
    Template::render("error/404", context! {})
}

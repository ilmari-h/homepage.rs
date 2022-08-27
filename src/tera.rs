use std::io;
use std::sync::Mutex;

use redis::Commands;
use rocket::State;

use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::blog::{BlogPost, BlogPostMetadata};
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

pub struct SharedRedis {
    pub connection: Mutex<redis::Connection>,
}

// TODO: Cow strings?
fn read_post(id: &String, redis: &mut redis::Connection) -> io::Result<String> {
    redis
        .get(id)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

#[get("/")]
pub fn index(blog_posts: &State<Vec<BlogPost>>, redis: &State<SharedRedis>) -> Template {
    let mut posts: Vec<BlogPostView> = vec![];
    let mut redis_l = redis
        .to_owned()
        .connection
        .lock()
        .expect("lock shared data");

    // Posts are sorted by date by default. Pick 3 newest.
    for v in blog_posts.iter().take(3) {
        if let Ok(raw_html) = read_post(&v.id, &mut redis_l) {
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

#[get("/blog?<tags>")]
pub fn blog(
    tags: Option<String>,
    blog_posts: &State<Vec<BlogPost>>,
    blog_tags: &State<Vec<String>>,
) -> Template {
    let mut s_tags: Vec<String> = vec![];
    if let Some(have_tags) = tags {
        s_tags = have_tags.split(' ').map(|s| String::from(s)).collect()
    }
    let posts: Vec<&BlogPost> = blog_posts.iter().collect();
    let tags: Vec<TagView> = blog_tags
        .iter()
        .map(|tag| TagView {
            tag,
            selected: s_tags.iter().find(|st| *st == tag).is_some(),
        })
        .collect();

    let s_posts: Vec<_> = if s_tags.len() > 0 {
        posts
            .iter()
            .filter(|p| {
                if let Some(tags) = &p.metadata.tags {
                    tags.iter().any(|at| s_tags.contains(at))
                } else {
                    false // No tags, can't be a match.
                }
            })
            .collect()
    } else {
        posts.iter().map(|p| p).collect()
    };

    Template::render(
        "blog",
        context! {
            posts: s_posts,
            tags
        },
    )
}

#[get("/blog/<post_id>")]
pub fn blog_posts(
    post_id: &str,
    blog_posts: &State<Vec<BlogPost>>,
    redis: &State<SharedRedis>,
) -> Option<Template> {
    let f_post = blog_posts.iter().find(|post| post.id == post_id)?;
    let mut redis_l = redis
        .to_owned()
        .connection
        .lock()
        .expect("Couldn't aquire lock on redis connection.");

    if let Ok(content) = read_post(&post_id.to_owned(), &mut redis_l) {
        let post = BlogPostView {
            id: String::from(post_id),
            content,
            metadata: f_post.metadata.clone(),
        };
        Some(Template::render(
            "blog_post",
            context! {
               post: post,
            },
        ))
    } else {
        None
    }
}

use std::fs;
use std::io;
use std::io::{stdout, Error, ErrorKind, Write};
use std::path::{Path, PathBuf};

use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use serde::Deserialize;
use serde::Serialize;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub fn render_blog_posts(path_src: &str, path_out: &str) -> io::Result<Vec<BlogPost>> {
    let files = read_markdown_sync(path_src)?; // TODO run concurrently
    let metadata = read_metadata_sync(path_src)?;// run concurrently
    writeln!(stdout(), "Found {} posts\n", files.len())?;
    for (p, f) in files {
        if let Some(new_fname) = p
            .file_name()
            .and_then(|os_fname| os_fname.to_str().and_then(|fname| fname.strip_suffix("md")))
        {
            fs::write(
                Path::new(path_out).join(String::from(new_fname) + "html"),
                f,
            )?;
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Can't render file {}", p.to_string_lossy()),
            ));
        }
    }
    Ok(metadata)
}

#[derive(Debug, Deserialize, Serialize,Clone)]
pub struct BlogPostMetadata {
    pub title: String,
    pub date_created: String,
    pub tags: Option<Vec<String>>,
    pub date_edited: Option<String>,
}

#[derive(Debug,Serialize)]
pub struct BlogPost {
    pub id: String,
    pub metadata: BlogPostMetadata,
}

pub fn read_metadata_sync(path: &str) -> io::Result<Vec<BlogPost>> {
    let paths = fs::read_dir(path)?;
    let res: Result<Vec<BlogPost>, toml::de::Error> = paths
        .filter_map(|de| -> Option<(String, String)> {
            if let Ok(entry) = de {
                let path = entry.path();
                if path.extension()?.eq("toml") {
                    let id = path.file_name().unwrap().to_str().unwrap().strip_suffix(".toml").unwrap();
                    return fs::read_to_string(&path).map_or_else(|_| None, |f| Some((f, String::from(id))));
                }
            }
            None
        })
        .map(|(toml_s, id)| {
            toml::from_str::<BlogPostMetadata>(&toml_s).map(|metadata| BlogPost { id, metadata })
        })
        .collect();
    match res {
        Ok(valid_res) => Ok(valid_res),
        Err(e) => Err(io::Error::new(ErrorKind::Other, e)),
    }
}

pub fn read_markdown_sync(path: &str) -> io::Result<Vec<(PathBuf, String)>> {
    let paths = fs::read_dir(path)?;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let res: Vec<(PathBuf, String)> = paths
        .filter_map(|de| -> Option<(PathBuf, String)> {
            if let Ok(entry) = de {
                let path = entry.path();
                if path.extension()?.eq("md") {
                    return fs::read_to_string(&path).map_or_else(|_| None, |f| Some((path, f)));
                }
            }
            None
        })
        .map(|(path, md)| {
            let mut html_str = String::new();
            let res = Parser::new_ext(&md, options);

            // Setup syntax highlighting
            // See: https://github.com/raphlinus/pulldown-cmark/issues/167
            let ss = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            let mut syntax = ss.find_syntax_by_extension("rs").unwrap();
            let theme = &ts.themes["base16-eighties.dark"];

            // Apply highlighting
            let mut new_res = Vec::new();
            let mut to_highlight = String::new();
            let mut in_code_block = false; // state

            for event in res {
                match event {
                    Event::Start(Tag::CodeBlock(b)) => {
                        if let CodeBlockKind::Fenced(c) = b {
                            if !c.is_empty() {
                                syntax = ss
                                    .find_syntax_by_extension(&c)
                                    .unwrap_or(ss.find_syntax_plain_text());
                            }
                        }
                        in_code_block = true;
                    }
                    Event::End(Tag::CodeBlock(_)) => {
                        if in_code_block {
                            // Format the whole multi-line code block as HTML all at once
                            let html =
                                highlighted_html_for_string(&to_highlight, &ss, &syntax, &theme);
                            if let Ok(str_html) = html {
                                // And put it into the vector
                                new_res.push(Event::Html(CowStr::from(str_html)));
                                to_highlight = String::new();
                                in_code_block = false;
                            }
                        }
                    }
                    Event::Text(t) => {
                        if in_code_block {
                            // If we're in a code block, build up the string of text
                            to_highlight.push_str(&t);
                        } else {
                            new_res.push(Event::Text(t))
                        }
                    }
                    e => {
                        new_res.push(e);
                    }
                }
            }

            html::push_html(&mut html_str, new_res.into_iter());
            (path, html_str)
        })
        .collect();

    Ok(res)
}

use std::borrow::Cow;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::{error::Error, io};
use std::fs;

use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use pulldown_cmark::{Parser, Options, html,Event,Tag, CowStr, CodeBlockKind};

pub fn render_markdown_files(path: &str) -> io::Result<()> {
    let files = build_markdown_sync(path)?;
    writeln!(stdout(),"Found {} posts\n", files.len());
    for (p,f) in files {
        //stdout().write_all(f.as_bytes());
        let fname = String::from( p.file_name().unwrap().to_str().unwrap() );
        let new_fname = fname.strip_suffix(".md").unwrap();
        fs::write( Path::new("./dist").join( String::from(new_fname) + ".html"), f)?;
    }
    Ok(())
}

pub fn build_markdown_sync(path: &str) -> io::Result<Vec<(PathBuf,String)>> {
    let paths = fs::read_dir(path)?;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let res: Vec<(PathBuf,String)> = paths
        .filter_map(|de| -> Option<(PathBuf, String)> {
            if let Ok(entry) = de {
                let path = entry.path();
                if path.extension()?.eq("md")  {
                    return fs::read_to_string(&path).map_or_else(|_| None, |f| Some((path,f)) )
                }
            }
            None
        })
        .map(|(path,md)| {
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
                                syntax = ss.find_syntax_by_extension(&c).unwrap_or(ss.find_syntax_plain_text());
                            }
                        }
                        // In actual use you'd probably want to keep track of what language this code is
                        in_code_block = true;
                    }
                    Event::End(Tag::CodeBlock(_)) => {
                        if in_code_block {
                            // Format the whole multi-line code block as HTML all at once
                            let html = highlighted_html_for_string(&to_highlight, &ss, &syntax, &theme);
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

            html::push_html(&mut html_str, new_res.into_iter() );
            (path,html_str)
        })
        .collect();

    Ok(res)
}

use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    title: String,
    brief: String,
    page_url: Option<String>,
    page_title: Option<String>,
    source_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavBarLink {
    title: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexPage {
    pub navbar_links: Vec<NavBarLink>,
    pub brief_html: String,
    pub projects: Vec<Project>,
}

pub fn read_config(path: PathBuf) -> io::Result<IndexPage> {
    let cfg = fs::read_to_string(&path)?;
    toml::from_str(&cfg).map_err(|e| Error::new(ErrorKind::Other, e))
}

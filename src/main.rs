use std::error::Error;

use http::{Response, StatusCode};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Person {
    first_name: String,
    last_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    #[serde(rename = "type")]
    extensionType: Vec<String>,
    name: String,
    author: String,
    desc: String,
    url: String,

    #[serde(rename = "makefile-entries")]
    makefileEntries: Vec<String>,
    ext: String,
    version: String,

    #[serde(rename = "last-modified")]
    lastModified: String,
}

pub struct Extensions {
    Extensions: Vec<Extension>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let resp = reqwest::blocking::get("https://httpbin.org/ip")?.text()?;
    println!("{:#?}", resp);
    Ok(())
}

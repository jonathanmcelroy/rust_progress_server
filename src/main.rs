#![feature(custom_derive)]
#![feature(plugin)]
#![feature(field_init_shorthand)]
#![plugin(rocket_codegen)]
#![allow(dead_code)]

extern crate rocket_contrib;
#[macro_use] extern crate nom;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate docopt;
extern crate hyper;
extern crate ini;
extern crate regex;
extern crate rocket;
extern crate serde_json;
extern crate url;

use std::io::Read;
use std::path::{Path, PathBuf};

use hyper::Client;
use hyper::status::StatusCode as hStatusCode;
use regex::Regex;
use rocket::Rocket;
use rocket::config::{active, Value};
use rocket::response::NamedFile;
use rocket_contrib::JSON;
use url::Url;

mod error;
mod parser;
mod util;
use error::{Error, ProgressResult};
use parser::{PreprocessorAnalysisSection, preprocessed_progress};
use util::u8_ref_to_string;

#[derive(Serialize, Deserialize)]
struct ProgressRes {
    pub contents: String,
    pub file_references: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct SearchRes {
    pub results: Vec<String>
}
#[derive(Serialize, Deserialize)]
struct AnalysisSectionsRes {
    pub sections: Vec<PreprocessorAnalysisSection>
}

fn get_file_server_address_from_config() -> ProgressResult<Url> {
    let config = active().ok_or(Error::General("No config file"))?;
    for (key, value) in config.extras() {
        if key == "file_server_address" {
            if let &Value::String(ref file_server_address) = value {
                return Ok(Url::parse(file_server_address)?);
            } else {
                return Err(Error::General("stec_root not a string in config file"));
            }
        }
    }
    return Err(Error::General("No stec_root in config file"));

}


// Get the contents of the progress file from the path
fn get_progress_file(conn: &Client, base: &Url, path: &str) -> ProgressResult<Vec<u8>> {
// fn get_progress_file(conn: &Client, path: &str) -> ProgressResult<String> {
    let url = base.clone();
    let url = url.join("file/")?;
    let url = url.join(path)?;

    let mut res = try!(conn.get(url).send());
    if res.status == hStatusCode::Ok {
        let mut ret = Vec::new();
        let _ = res.read_to_end(&mut ret);
        return Ok(ret);
    } else {
        return Err(Error::General("Could not get file"));
    }
}

// Find a progress file based upon the search query. Right now only works on program file names
fn find_progress_file(conn: &Client, base: &Url, path: &str) -> ProgressResult<Vec<String>> {
// fn find_progress_file(conn: &Client, path: &str) -> ProgressResult<Vec<String>> {
    let url = base.clone();
    let url = url.join("find/")?;
    let url = url.join(path)?;

    let mut res = conn.get(url).send()?;
    if res.status == hStatusCode::Ok {
        let mut ret = String::new();
        let _ = res.read_to_string(&mut ret);
        return Ok(serde_json::from_str(&ret)?);
    } else {
        return Err(Error::General("Could not get file"));
    }
}

// Find the files referenced by this file
/*
fn progress_children<'a>(contents: &'a str) -> Vec<&'a str> {
    let re = Regex::new(r"[\w-]*?\.w").unwrap();
    re.find_iter(contents).map(|(start, end)| &contents[start..end]).collect()
}
*/

#[get("/<path..>")]
fn static_handler(path: PathBuf) -> ProgressResult<NamedFile> {
    let full_path = Path::new("public/").join(path);
    NamedFile::open(full_path).map_err(|err| err.into())
}

#[get("/<path..>")]
fn static_html_handler(path: PathBuf) -> ProgressResult<NamedFile> {
    NamedFile::open(Path::new("public/html/").join(path)).map_err(|err| err.into())
}

#[get("/")]
fn static_html_index() -> ProgressResult<NamedFile> {
    NamedFile::open(Path::new("public/html/index.html")).map_err(|err| err.into())
}

// Return the given program's contents
#[get("/program/<program>", format="application/json")]
fn program(program: String) -> ProgressResult<JSON<ProgressRes>> {
    let conn = Client::new();
    let file_server_url = get_file_server_address_from_config()?;
    let file_contents = get_progress_file(&conn, &file_server_url, &program)?;

    //let file_references_regex = Regex::new(r"[-\w/\\]+?\.[pwi]").unwrap();
    //let file_references = file_references_regex.find_iter(&file_contents).map(|each_match| String::from(each_match.as_str()).replace("\\", "/")).collect();
    Ok(JSON(ProgressRes {
        contents: u8_ref_to_string(&file_contents),
        file_references: vec![]
    }))
}

// Find a program based upon the search query
#[get("/search/<program>", format="application/json")]
fn file(program: String) -> ProgressResult<JSON<SearchRes>> {
    let conn = Client::new();
    let file_server_url = get_file_server_address_from_config()?;
    let find_results = find_progress_file(&conn, &file_server_url, &program)?;
    Ok(JSON(SearchRes {
        results: find_results
    }))
}

// Return the given program's analysis sections
#[get("/analysis_sections/<program>", format="application/json")]
fn analysis_sections(program: String) -> ProgressResult<JSON<AnalysisSectionsRes>> {
    let conn = Client::new();
    let file_server_url = get_file_server_address_from_config()?;
    let file_contents = get_progress_file(&conn, &file_server_url, &program)?;

    let parse = preprocessed_progress(&file_contents).to_full_result()?;
    let sections = PreprocessorAnalysisSection::from(parse)?;
    Ok(JSON(AnalysisSectionsRes {
        sections: sections
    }))
}

fn main() {
    Rocket::ignite()
        .mount("/api/", routes![program, file, analysis_sections])
        .mount("/static/", routes![static_handler])
        .mount("/", routes![static_html_handler, static_html_index])
        .launch();
}

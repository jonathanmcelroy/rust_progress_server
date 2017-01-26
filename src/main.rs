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

use std::path::{Path, PathBuf};

use regex::Regex;
use rocket::Rocket;
use rocket::response::NamedFile;
use rocket_contrib::JSON;


mod error;
mod parser;
mod util;
mod file_server_api;

use error::{Error, ProgressResult};
use parser::{PreprocessorAnalysisSection, preprocessed_progress};
use util::u8_ref_to_string;
use file_server_api::{get_procedure_contents, find_procedure};

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


#[get("/<path..>")]
fn static_handler(path: PathBuf) -> ProgressResult<NamedFile> {
    let full_path = Path::new("public/").join(path);
    NamedFile::open(full_path).map_err(|err| err.into())
}

#[get("/<path..>", rank = 3)]
fn static_html_handler(path: PathBuf) -> ProgressResult<NamedFile> {
    NamedFile::open(Path::new("public/html/").join(path)).map_err(|err| err.into())
}

#[get("/", rank = 4)]
fn static_html_index() -> ProgressResult<NamedFile> {
    NamedFile::open(Path::new("public/html/index.html")).map_err(|err| err.into())
}

// Return the given program's contents
#[get("/procedure/<procedure>", format="application/json")]
fn get_procedure_route(procedure: String) -> ProgressResult<JSON<ProgressRes>> {
    let file_contents = get_procedure_contents(&procedure)?;

    //let file_references_regex = Regex::new(r"[-\w/\\]+?\.[pwi]").unwrap();
    //let file_references = file_references_regex.find_iter(&file_contents).map(|each_match| String::from(each_match.as_str()).replace("\\", "/")).collect();
    Ok(JSON(ProgressRes {
        contents: u8_ref_to_string(&file_contents),
        file_references: vec![]
    }))
}

#[get("/search/procedure/<procedure>/<inner_procedure>")]
fn find_inner_procedure_route(procedure: String, inner_procedure: String) -> ProgressResult<String> {
    Ok("Ok".to_string())
}

// Find a procedure based upon the search query
#[get("/search/procedure/<procedure>", rank = 2, format="application/json")]
fn find_procedure_route(procedure: &str) -> ProgressResult<JSON<SearchRes>> {
    let find_results = find_procedure(procedure)?;
    Ok(JSON(SearchRes {
        results: find_results
    }))
}

// Return the given program's analysis sections
#[get("/analysis_sections/<procedure>", format="application/json")]
fn get_analysis_sections_route(procedure: &str) -> ProgressResult<JSON<AnalysisSectionsRes>> {
    let file_contents = get_procedure_contents(procedure)?;

    let parse = preprocessed_progress(&file_contents).to_full_result()?;
    let sections = PreprocessorAnalysisSection::from(parse)?;
    Ok(JSON(AnalysisSectionsRes {
        sections: sections
    }))
}

fn main() {
    Rocket::ignite()
        .mount("/api/", routes![
               get_procedure_route,
               find_procedure_route,
               find_inner_procedure_route,
               get_analysis_sections_route
        ])
        .mount("/static/", routes![static_handler])
        .mount("/", routes![static_html_handler, static_html_index])
        .launch();
}

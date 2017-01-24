#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate nom;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde;
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

use docopt::Docopt;
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
use error::{Error, ProgressResult, unwrap_or_exit};
use parser::{Statement, parse};

#[derive(Serialize, Deserialize)]
struct ProgressRes {
    pub contents: String,
    pub file_references: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct SearchRes {
    pub results: Vec<String>
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
fn get_progress_file(conn: &Client, base: &Url, path: &str) -> ProgressResult<String> {
// fn get_progress_file(conn: &Client, path: &str) -> ProgressResult<String> {
    let url = base.clone();
    let url = url.join("file/")?;
    let url = url.join(path)?;

    let mut res = try!(conn.get(url).send());
    if res.status == hStatusCode::Ok {
        let mut ret = String::new();
        let _ = res.read_to_string(&mut ret);
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
    // server.mount("/static/", StaticFilesHandler::new("public/"));
}

#[get("/<path..>")]
fn static_html_handler(path: PathBuf) -> ProgressResult<NamedFile> {
    NamedFile::open(Path::new("public/html/").join(path)).map_err(|err| err.into())
    // server.get("/", StaticFilesHandler::new("public/html/"));
}

#[get("/")]
fn static_html_index() -> ProgressResult<NamedFile> {
    NamedFile::open(Path::new("public/html/index.html")).map_err(|err| err.into())
}

#[get("/program/<program>", format="application/json")]
fn program(program: String) -> ProgressResult<JSON<ProgressRes>> {
    let conn = Client::new();
    let file_server_url = get_file_server_address_from_config()?;
    let file_contents = get_progress_file(&conn, &file_server_url, &program)?;

    let file_references_regex = Regex::new(r"[-\w/\\]+?\.[pwi]").unwrap();
    let file_references = file_references_regex.find_iter(&file_contents).map(|each_match| String::from(each_match.as_str()).replace("\\", "/")).collect();
    Ok(JSON(ProgressRes {
        contents: file_contents,
        file_references: file_references
    }))
}

#[get("/search/<program>", format="application/json")]
fn file(program: String) -> ProgressResult<JSON<SearchRes>> {
    let conn = Client::new();
    let file_server_url = get_file_server_address_from_config()?;
    let find_results = find_progress_file(&conn, &file_server_url, &program)?;
    Ok(JSON(SearchRes {
        results: find_results
    }))
}

fn main() {
    // "/api/program/<program>" just gives the given program's contents
    /*
    let program_regex = Regex::new(r"/api/program/(?P<program>[-%~\w/\.]+)$").unwrap();
    let get_program_file_server_url = file_server_url.clone();
    server.get(program_regex,  middleware! { |req, res| {
        let conn = Client::new();
        let r_contents = get_progress_file(&conn, &get_program_file_server_url, &req.param("program").unwrap());
        let contents = match r_contents {
            Ok(contents) => contents,
            Err(err) => panic!("{:?}", err)
        };

        {
            //let parsedContents = parse(&contents).unwrap();
            //println!("{:?}", parsedContents);
            
            // let Progress::String(contents) = parsedContents;
        }

        let file_references_regex = Regex::new(r"[-\w/\\]+?\.[pwi]").unwrap();
        let file_references = file_references_regex.find_iter(&contents).map(|(l, r)| String::from(&contents[l..r]).replace("\\", "/")).collect();

        let progress_json = ProgressRes {
            contents: contents,
            file_references: file_references,
        };

        return res.send(json::encode(&progress_json).unwrap());
    }});
    */

    // "/api/search/<contents>" tries to find a program based upon the search query
    /*
    let search_regex = Regex::new(r"/api/search/(?P<contents>.+)$").unwrap();
    server.get(search_regex, middleware! { |req, res| {
        let conn = Client::new();
        let find_results = find_progress_file(&conn, &file_server_url, &req.param("contents").unwrap()).unwrap();
        let search_json = SearchRes {
            results: find_results
        };
        return res.send(json::encode(&search_json).unwrap());
    }});
    */

    // server.mount("/static/", StaticFilesHandler::new("public/"));

    // server.get("/", StaticFilesHandler::new("public/html/"));

    // let ip = args.get_str("<ip>");
    // server.listen(ip);

    Rocket::ignite()
        .mount("/api/", routes![program, file])
        .mount("/static/", routes![static_handler])
        .mount("/", routes![static_html_handler, static_html_index])
        .launch();
}

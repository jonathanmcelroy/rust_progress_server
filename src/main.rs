#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate ini;
extern crate regex;
extern crate url;

use std::io::Read;
use nickel::{Nickel, HttpRouter, Mountable, StaticFilesHandler};
use hyper::Client;
use hyper::status::StatusCode;
use rustc_serialize::json;
use ini::Ini;
use regex::Regex;
use url::Url;

const HOST: &'static str = "http://192.168.221.80:3000/";

#[derive(RustcDecodable, RustcEncodable)]
struct ProgressRes {
    pub contents: String,
    pub file_references: Vec<String>,
}
#[derive(RustcDecodable, RustcEncodable)]
struct SearchRes {
    pub results: Vec<String>
}

#[derive(Debug)]
enum Error {
    Hyper(hyper::Error),
    Ini(&'static str, ini::ini::Error),
    General(&'static str),
}

fn get_progress_file(conn: &Client, path: &str) -> Result<String, Error> {
    let mut url = String::from(HOST);
    url.push_str("file/");
    url.push_str(path);

    conn.get(&url).send()
        .map_err(|err| Error::Hyper(err))
        .and_then(|mut res| {
            if res.status == StatusCode::Ok {
                let mut ret = String::new();
                let _ = res.read_to_string(&mut ret);
                return Ok(ret);
            } else {
                return Err(Error::General("Could not get file"));
            }
        })
}

fn find_progress_file(conn: &Client, path: &str) -> Result<Vec<String>, Error> {
    let mut url = String::from(HOST);
    url.push_str("find/");
    url.push_str(path);
    println!("{}", url);

    conn.get(&url).send()
        .map_err(|err| Error::Hyper(err))
        .and_then(|mut res| {
            if res.status == StatusCode::Ok {
                let mut ret = String::new();
                let _ = res.read_to_string(&mut ret);
                return Ok(json::decode(&ret).unwrap());
            } else {
                return Err(Error::General("Could not get file"));
            }
        })
}

fn progress_children<'a>(contents: &'a str) -> Vec<&'a str> {
    let re = Regex::new(r"[\w-]*?\.w").unwrap();
    re.find_iter(contents).map(|(start, end)| &contents[start..end]).collect()
}

fn main() {
    let mut server = Nickel::new();

    server.utilize(middleware! {
        |req, res| {
            println!("Resolving {:?}", req.path_without_query().unwrap());
            return res.next_middleware();
        }
    });

    let program_regex = Regex::new(r"/program/(?P<program>[-%~\w/\.]+)$").unwrap();
    let search_regex = Regex::new(r"/search/(?P<contents>.+)$").unwrap();
    server.mount("/api/", router! {
        get program_regex => |req, res| {
            let conn = Client::new();
            let r_contents = get_progress_file(&conn, &req.param("program").unwrap());
            let contents = match r_contents {
                Ok(contents) => contents,
                Err(err) => panic!("{:?}", err)
            };

            let file_references_regex = Regex::new(r"[-\w/\\]+?\.[pwi]").unwrap();
            let file_references = file_references_regex.find_iter(&contents).map(|(l, r)| String::from(&contents[l..r]).replace("\\", "/")).collect();

            let progress_json = ProgressRes {
                contents: contents,
                file_references: file_references,
            };

            return res.send(json::encode(&progress_json).unwrap());
        }
        get search_regex => |req, res| {
            let conn = Client::new();
            let find_results = find_progress_file(&conn, &req.param("contents").unwrap()).unwrap();
            let search_json = SearchRes {
                results: find_results
            };
            return res.send(json::encode(&search_json).unwrap());
        }
    });

    server.mount("/static/", StaticFilesHandler::new("public/"));

    server.get("/", StaticFilesHandler::new("public/html/"));

    server.listen("192.168.221.83:3000");
}

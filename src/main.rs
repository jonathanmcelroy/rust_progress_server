#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate ini;
extern crate regex;
extern crate url;

use std::io::Read;
use nickel::{Nickel, HttpRouter, Mountable, StaticFilesHandler};
use nickel::status::StatusCode as nStatusCode;
use hyper::Client;
use hyper::status::StatusCode as hStatusCode;
use rustc_serialize::json;
use regex::Regex;

mod error;
use error::{Error, ProgressResult, unwrap_or_exit};

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

fn get_progress_file(conn: &Client, path: &str) -> ProgressResult<String> {
    let mut url = String::from(HOST);
    url.push_str("file/");
    url.push_str(path);

    let mut res = try!(conn.get(&url).send());
    if res.status == hStatusCode::Ok {
        let mut ret = String::new();
        let _ = res.read_to_string(&mut ret);
        return Ok(ret);
    } else {
        return Err(Error::General("Could not get file"));
    }
}

fn find_progress_file(conn: &Client, path: &str) -> ProgressResult<Vec<String>> {
    let mut url = String::from(HOST);
    url.push_str("find/");
    url.push_str(path);
    println!("{}", url);

    let mut res = try!(conn.get(&url).send());
    if res.status == hStatusCode::Ok {
        let mut ret = String::new();
        let _ = res.read_to_string(&mut ret);
        return Ok(json::decode(&ret).unwrap());
    } else {
        return Err(Error::General("Could not get file"));
    }
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

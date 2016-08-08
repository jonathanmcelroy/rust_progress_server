#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate rustc_serialize;
extern crate ini;
extern crate regex;

use std::io::Read;
use nickel::{Nickel, HttpRouter, Mountable, StaticFilesHandler};
use hyper::{Client};
use rustc_serialize::json;
use ini::Ini;
use regex::Regex;

const HOST: &'static str = "http://192.168.221.80:3000/";

#[derive(RustcDecodable, RustcEncodable)]
struct ProgressRes {
    pub contents: String
}

#[derive(Debug)]
enum Error {
    Hyper(hyper::Error),
    General(String),
}

fn get_file(conn: &Client, path: &str) -> Result<String, Error> {
    let mut ret = String::new();
    let mut url = String::from(HOST);
    url.push_str("file/");
    url.push_str(path);

    conn.get(&url).send()
        .map(|mut res| {
            let _ = res.read_to_string(&mut ret);
            return ret;
        })
        .map_err(|err| Error::Hyper(err))

}

fn get_propath(conn: &Client) -> Vec<String> {
    let mut stec_ini = get_file(&conn, "stec.ini").unwrap();
    stec_ini = stec_ini.replace("\\", "/");

    let conf = Ini::load_from_str(&stec_ini).unwrap();

    conf.section(Some("Startup"))
        .and_then(|section| section.get("PROPATH"))
        .unwrap()
        .split(",").map(|s| String::from(s)).collect()
}

fn get_progress_file(conn: &Client, path: &str, propath: &Vec<String>) -> Result<String, Error> {
    for each_path in propath {
        let mut new_path = each_path.to_owned();
        new_path.push('/');
        new_path.push_str(path);
        println!("{}", new_path);

        let result = get_file(conn, &new_path);
        if result.is_ok() {
            println!("{:?}", result);
            return result;
        }
    }
    return Err(Error::General(String::from("Could not find file")));
}

fn progress_children<'a>(contents: &'a str) -> Vec<&'a str> {
    let re = Regex::new(r"[\w-]*?\.w").unwrap();
    re.find_iter(contents).map(|(start, end)| &contents[start..end]).collect()
}

fn main() {
    let conn = Client::new();
    let propath = get_propath(&conn);

    let mut server = Nickel::new();

    server.utilize(middleware! {
        |req, res| {
            println!("Resolving {:?}", req.path_without_query().unwrap());
            return res.next_middleware();
        }
    });

    let program_regex = Regex::new(r"/program/(?P<program>[-%~\w/\.]+)$").unwrap();
    server.mount("/api/", router! {
        get program_regex => |req, res| {
            let contents = get_progress_file(&conn, &req.param("program").unwrap().replace("%2F", "/"), &propath).unwrap();
            println!("{:?}", contents);

            let progress_json = ProgressRes {
                contents: contents
            };

            return res.send(json::encode(&progress_json).unwrap());
        }
    });

    server.mount("/static/", StaticFilesHandler::new("public/"));

    server.get("/", StaticFilesHandler::new("public/html/"));

    server.listen("192.168.221.83:3000");
}

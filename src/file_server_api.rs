use hyper::Client;
use hyper::status::StatusCode as hStatusCode;
use rocket::config::{active, Value};
use serde_json::from_str;
use std::io::Read;
use url::Url;

use error::{ProgressResult, Error, add_message};

pub fn get_procedure_contents(procedure: &str) -> ProgressResult<Vec<u8>> {
    let conn = Client::new();
    let file_server_url = get_file_server_address_from_config()?;
    get_progress_file(&conn, &file_server_url, procedure)
}

pub fn find_procedure(procedure: &str) -> ProgressResult<Vec<String>> {
    let conn = Client::new();
    let file_server_url = get_file_server_address_from_config()?;
    find_progress_file(&conn, &file_server_url, procedure)
}


fn get_file_server_address_from_config() -> ProgressResult<Url> {
    let config = active().ok_or(Error::new("No config file"))?;
    for (key, value) in config.extras() {
        if key == "file_server_address" {
            if let &Value::String(ref file_server_address) = value {
                return Url::parse(file_server_address).map_err(add_message(format!("file_server_address: '{}' in Rocket.tml is not a valid address", file_server_address)));
            } else {
                return Err(Error::new("stec_root not a string in config file"));
            }
        }
    }
    return Err(Error::new("No stec_root in config file"));

}

// Get the contents of the progress file from the path
fn get_progress_file(conn: &Client, base: &Url, path: &str) -> ProgressResult<Vec<u8>> {
    let url = base.clone();
    let url = url.join("file/")?;
    let url = url.join(path)?;

    let mut res = conn.get(url).send()?;
    if res.status == hStatusCode::Ok {
        let mut ret = Vec::new();
        let _ = res.read_to_end(&mut ret);
        return Ok(ret);
    } else {
        return Err(Error::new(format!("'{}' does not exist on the file server", path)));
    }
}

// Find a progress file based upon the search query. Right now only works on program file names
fn find_progress_file(conn: &Client, base: &Url, path: &str) -> ProgressResult<Vec<String>> {
    let url = base.clone();
    let url = url.join("find/")?;
    let url = url.join(path)?;

    let mut res = conn.get(url).send()?;
    if res.status == hStatusCode::Ok {
        let mut ret = String::new();
        let _ = res.read_to_string(&mut ret);
        return Ok(from_str(&ret)?);
    } else {
        return Err(Error::new("Could not get file"));
    }
}

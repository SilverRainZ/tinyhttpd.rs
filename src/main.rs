/******************************************************************************
 *  tinyhttpd.rs - Tiny http server implement via rust.
 *  SilverRainZ <silverrain.zhang at gmail dot com>
 *
 *  - [x] http requset parse
 *  - [x] url arguments reslove
 *  - [x] static file handle
 *  - [x] cgi execute
 *  - [ ] dot removal procedure (RFC 3986)
 *
 *****************************************************************************/

#[macro_use]
extern crate log;

mod tinylogger;

use std::io::prelude::*;
use std::io::Bytes;
use std::str;
// use std::io::Bytes;
use std::fs::File;
use std::net::{TcpStream, TcpListener};
use std::process::{Command, Stdio};
use std::os::unix::fs::PermissionsExt;

/* const string and bytes */

static RESPONSE_HEADER: &'static [u8] =
    b"HTTP/1.0 200 OK\r\n\
      Content-type: text/html; charset=utf-8\r\n\
      Server: tinyhttpd.rs/0.1.0\r\n\
      \r\n";

fn not_found() -> &'static [u8] {
    "HTTP/1.0 404 NOT FOUND\r\n\
     Server: tinyhttpd.rs/0.1.0\r\n\
     Content-Type: text/html; charset=utf-8\r\n\
     \r\n\
     <html>\
        <title>404 Not Found</title>\
        <body>\
            <center>\
                <h1>Not Found!</h1>\
                <p>忘れた</p>\
            </center>\
        </body>\
     </html>\r\n".as_bytes()
}

/* struct */

struct HttpRequestLine {
    method: String,
    uri: String,
    version: String,
    args: Option<String>,
}

struct HttpHeadEntry {
    key: String,
    val: String,
}

struct HttpRequest {
    req_line: HttpRequestLine,
    head_entrys: Vec<HttpHeadEntry>,
    body: Option<String>,
}

/* implement */

fn read_line(stream: &mut TcpStream) -> Option<String> {
    let mut bytes = stream.bytes();
    let mut crlf = false;
    let mut buf:Vec<u8> = Vec::new();

    loop {
        match bytes.next() {
            Some(Ok(b)) => {
                buf.push(b);
                crlf = match b {
                    b'\r' => true,
                    b'\n' =>
                        if crlf {
                            // pop "\r\n"
                            buf.pop();
                            buf.pop();
                            break
                        } else {
                            false
                        },
                    _ => false,
                }
            }
            Some(Err(e)) => {
                error!("read error: {}", e);
                return None
            }
            None => {
                error!("unterminated line");
                return None
            }
        }
    }

    match str::from_utf8(&buf) {
        Ok(s) => Some(s.to_string()),
        Err(e) => {
            error!("invail utf-8 sequence, {}", e);
            None
        }
    }
}

fn accept(mut stream: TcpStream) {
    let mut lines:Vec<String> = Vec::new();

    let line = match read_line(&mut stream) {
        Some(s) => s,
        None => return,
    };
    let req_line = match parse_req_line(&line) {
        Some(v) => v,
        None => return,
    };

    info!("{} {}", req_line.method, req_line.uri);

    let mut head_entrys:Vec<HttpHeadEntry> = Vec::new();
    loop {
        let line = match read_line(&mut stream) {
            Some(s) => s,
            None => return,
        };

        // lines.push(line);
        if line == "" { break }

        match parse_header_entry(line) {
            Some(v) => head_entrys.push(v),
            None => return,
        }
    }

    let mut request = HttpRequest {
        req_line: req_line,
        head_entrys: head_entrys,
        body: None,
    };

    response(stream, request);
}

macro_rules! unwrap_or_return {
    ($exp:expr, $msg: expr) => ({
            let v = match $exp {
                Some(v) => v,
                None => {
                    error!($msg);
                    return None
                }
            };
            v
        })
}

fn parse_req_line(req_line: &str) -> Option<HttpRequestLine> {
    let mut req_line =  req_line.split_whitespace();

    let method = unwrap_or_return!(req_line.next(), "no method found in request line");
    debug!("method: {}", method);

    let rawuri = unwrap_or_return!(req_line.next(), "no uri found in request line");
    debug!("rawuri: {}", rawuri);

    let version = unwrap_or_return!(req_line.next(), "no version found in request line");
    debug!("version: {}", version);

    Some(HttpRequestLine {
        method: method.to_string(),
        uri: rawuri.to_string(),
        version: version.to_string(),
        args: None,
    })
}

fn parse_header_entry(head_entry: String) -> Option<HttpHeadEntry> {
    let idx = unwrap_or_return!(head_entry.find(": "), "no value found in header entry");
    let (key, val) = head_entry.split_at(idx);
    let val = &val[2..]; // skip ": "
    debug!("header: key: {}, val: {}", key, val);

    Some(HttpHeadEntry { key: key.to_string(), val: val.to_string() })
}

fn parse_query_string(req_line: &mut HttpRequestLine) -> bool {
    match req_line.uri.find("?") {
        Some(idx) => {
            let rawuri = req_line.uri.clone();
            let (uri, args) = rawuri.split_at(idx);
            let args = &args[1..]; // skip "?"
            req_line.uri = uri.to_string();
            req_line.args = Some(args.to_string());
            true
        }
        None => false,
    }
}

fn response(mut stream: TcpStream, mut req: HttpRequest) {
    let mut cgi = match req.req_line.method.as_str() {
        "POST" => true,
        "GET" => parse_query_string(&mut req.req_line),
        _ => {
            warn!("unsupported method");
            return;
        }
    };

    let path = match req.req_line.uri.chars().last() {
        Some('/') =>  "root".to_string() + &req.req_line.uri + "index.html",
        _ => "root".to_string() + &req.req_line.uri,
    };

    debug!("cgi: {}, path: {}", cgi, path);

    match File::open(&path) {
        Ok(mut f) => {
            match f.metadata() {
                Ok(meta) => {
                    /* mode = ___x__x__x */
                    cgi = meta.permissions().mode() & 0o111 != 0;
                }
                Err(e) => {
                    error!("failed to get metadata of '{}': {}", &path, e);
                }
            }

            if cgi {
                stream.write(RESPONSE_HEADER).unwrap();
                stream.write(exec_cgi(path, req).as_bytes()).unwrap();
            } else {
                let mut buf = String::new();

                f.read_to_string(&mut buf).unwrap();
                stream.write(RESPONSE_HEADER).unwrap();
                stream.write(buf.as_bytes()).unwrap();
            }
        },
        Err(e) => {
            error!("failed to open '{}': {}", &path, e);
            stream.write(not_found()).unwrap();
        }
    }
}

fn serv_file(path: String, req: HttpRequest) {
}

fn exec_cgi(path: String, req: HttpRequest) -> String {
    // for GET: discard body
    // for POST: read according Content-Length, TODO
    info!("path: {}, args: {:?}", path, req.req_line.args);
    let child = Command::new(path)
        .env("REQUEST_METHOD", req.req_line.method)
        .env("QUERY_STRING", req.req_line.args.unwrap()) // TODO
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    let output = child.wait_with_output()
        .expect("failed to wait on child");

    str::from_utf8(&output.stdout).unwrap().to_string()
}

fn main() {
    tinylogger::init(log::LogLevelFilter::Debug).unwrap();

    let port = 30528;
    let addr = "127.0.0.1";
    let addr = addr.to_string() + ":" + &port.to_string();

    info!("listening on {}", addr);

    let listenser = TcpListener::bind(&*addr).unwrap();

    for stream in listenser.incoming() {
        match stream {
            Ok(stream) => {
                accept(stream);
            }
            Err(e) => {
                error!("listenser: {}", e);
            }
        }
    }
}

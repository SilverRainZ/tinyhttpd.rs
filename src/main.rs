/******************************************************************************
 *  tinyhttpd.rs - Tiny http server implement via rust.
 *  SilverRainZ <silverrain.zhang at gmail dot com>
 *
 *  - [x] http requset parse
 *  - [ ] url arguments reslove
 *  - [x] static file handle
 *  - [ ] cgi execute
 *  - [ ] dot removal procedure (RFC 3986)
 *
 *****************************************************************************/

#[macro_use]
extern crate log;

mod tinylogger;

use std::io::prelude::*;
use std::str;
use std::fs::File;
use std::net::{TcpStream, TcpListener};

/* const string and bytes */

static RESPONSE_HEADER: &'static [u8] =
    b"HTTP/1.0 200 OK\r\n\
      Content-type: text/html; charset=utf-8\r\n\
      Server: tinyhttpd.rs/0.1.0\r\n\
      \r\n";

fn welcome() -> &'static [u8] {
    "HTTP/1.0 200 OK\r\n\
     Content-type: text/html; charset=utf-8\r\n\
     \r\n\
     <html>\
        <head>\
            <title>君の名は!</title>\
        </head>\
        <body>\
            <center>\
                <h1>Welcome!</h1>\
                <p>君の名は!</p>\
            </center>\
        </body>\
     </html>".as_bytes()
}

fn not_found() -> &'static [u8] {
    "HTTP/1.0 404 NOT FOUND\r\n\
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

struct HttpHeadEntry<'req> {
    key: &'req str,
    val: &'req str,
}

struct HttpRequest<'req> {
    /* request line */
    method: &'req str,
    uri: &'req str,
    version: &'req str,
    args: &'req str,

    /* request header */
    head_entrys: Vec<HttpHeadEntry<'req>>,

    /* request body */
    body: &'req str,
}

/* implement */

fn accept(mut stream: TcpStream) {
    let mut buf = [0; 512];
    // let mut req: HttpRequest = HttpRequest{ .. };

    stream.read(&mut buf).unwrap();

    match str::from_utf8(&buf) {
        Ok(s) => {
            match parse(s) {
                Some(req) => response(stream, req),
                None => error!("failed to parse http request"),
            }
        },
        Err(e) => error!("invaild utf-8 sequcence {}", e),
    }
}

macro_rules! parse_abort {
    ($($arg:tt)*) => ({
            warn!($($arg)*);
            return None
        })
}

fn parse(req: &str) -> Option<HttpRequest> {
    let mut line = req.split("\r\n");

    /* read request line */
    let req_line = match line.next() {
        Some(v) => v,
        None => parse_abort!("no request line found"),
    };
    debug!("request line: {}", req_line);

    let mut req_line =  req_line.split_whitespace();

    let method = match req_line.next() {
        Some(v) => {
            if v != "POST" && v != "GET" {
                parse_abort!("unsupported method");
            } else {
                v
            }
        },
        None => parse_abort!("no method found in request line"),
    };
    debug!("method: {}", method);

    let rawuri = match req_line.next() {
        Some(v) => v,
        None => parse_abort!("no uri found in request line"),
    };
    debug!("rawuri: {}", rawuri);

    let version = match req_line.next() {
        Some(v) => v,
        None => parse_abort!("no version found in request line"),
    };
    debug!("version: {}", version);

    /* read request header */
    let mut head_entrys: Vec<HttpHeadEntry> = Vec::new();
    loop {
        let req_header = match line.next() {
            Some("") => break,  // end of header
            Some(v) => v,
            None => parse_abort!("no request header found"),
        };

        let head_entry = match req_header.find(": ") {
            Some(idx) => {
                let (key, val) = req_header.split_at(idx);
                let val = &val[2..]; // skip ": "
                HttpHeadEntry { key: key, val: val }
            },
            None => parse_abort!("no value found in header '{}'", req_header),
        };

        debug!("header: key: {}, val: {}", head_entry.key, head_entry.val);
        head_entrys.push(head_entry);
    }

    /* read request body */
    let req_body = match line.next() {
        Some(v) => v,
        None => "", // OK?
    };
    debug!("request body: {}", req_body);

    info!("{} {}", method, rawuri);

    Some(HttpRequest {
        method: method, uri: rawuri, version: version, args: "",
        head_entrys: head_entrys,
        body: req_body,
    })
}

fn response(mut stream: TcpStream, mut req: HttpRequest) {
    let mut cgi = match req.method {
        "POST" => true,
        "GET" => match req.uri.find("?") {
            Some(idx) => {
                let (uri, args) = req.uri.split_at(idx);
                    let args = &args[1..]; // skip "?"
                    req.uri = uri;
                    req.args = args;
                    true
            },
            None => false,
        },
        _ => {
            warn!("unsupported method");
            return;
        }
    };

    /* internal location */
    if req.uri == "/welcome" {
        stream.write(welcome()).unwrap();
        return;
    }

    let path = match req.uri.chars().last() {
        Some('/') =>  "root".to_string() + req.uri + "index.html",
        _ => "root".to_string() + req.uri,
    };

    debug!("cgi: {}, path: {}", cgi, path);

    let mut buf = String::new();

    if !cgi {
        match File::open(&path) {
            Ok(mut f) => {
                f.read_to_string(&mut buf).unwrap();
                stream.write(RESPONSE_HEADER).unwrap();
                stream.write(buf.as_bytes()).unwrap();
            },
            Err(e) => {
                error!("failed to open {}: {}", &path, e);
                stream.write(not_found()).unwrap();
            }
        }
    } else {
        stream.write(exec_cgi(req).as_bytes()).unwrap();
    }
}

fn exec_cgi(req: HttpRequest) -> String {
    "cgi".to_string()
}

fn main() {
    tinylogger::init().unwrap();

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

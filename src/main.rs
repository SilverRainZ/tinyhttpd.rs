use std::io::prelude::*;
use std::str;
use std::fs::File;
use std::net::{TcpStream, TcpListener};

/* const string and bytes */

static SERVER_HEADER: &'static [u8] =
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

    let s = match str::from_utf8(&buf) {
        Ok(v) => v,
        Err(e) => panic!("invaild utf-8 sequcence {}", e),
    };

    let req = match parse(s) {
        Some(v) => v,
        None => panic!("failed to parse http request"),
    };

    response(stream, req);
}

fn parse(req: &str) -> Option<HttpRequest> {
    let mut line = req.split("\r\n");

    /* read request line */
    let req_line = match line.next() {
        Some(v) => v,
        None => panic!("no request line found"),
    };
    println!("request line: {}", req_line);

    let mut req_line =  req_line.split_whitespace();

    let method = match req_line.next() {
        Some(v) => {
            if v != "POST" && v != "GET" {
                panic!("unsupported method");
            } else {
                v
            }
        },
        None => panic!("no method found in request line"),
    };
    println!("method: {}", method);

    let rawuri = match req_line.next() {
        Some(v) => v,
        None => panic!("no uri found in request line"),
    };
    println!("rawuri: {}", rawuri);

    let version = match req_line.next() {
        Some(v) => v,
        None => panic!("no version found in request line"),
    };
    println!("version: {}", version);

    /* read request header */
    let mut head_entrys: Vec<HttpHeadEntry> = Vec::new();
    loop {
        let req_header = match line.next() {
            Some("") => break,  // end of header
            Some(v) => v,
            None => panic!("no request header found"),
        };

        let head_entry = match req_header.find(": ") {
            Some(idx) => {
                let (key, val) = req_header.split_at(idx);
                let val = &val[2..]; // skip ": "
                HttpHeadEntry { key: key, val: val }
            }
            None => panic!("no value found in header '{}'", req_header),
        };

        println!("header: key: {}, val: {}", head_entry.key, head_entry.val);
        head_entrys.push(head_entry);
    }

    /* read request body */
    let req_body = match line.next() {
        Some(v) => v,
        None => "", // OK?
    };
    println!("request body: {}", req_body);

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
        _ => panic!("unsupported method"),
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

    println!("cgi: {}, path: {}", cgi, path);

    let mut buf = [0; 512];

    match File::open(path) {
        Ok(mut f) => {
            f.read(&mut buf).unwrap();
            stream.write(SERVER_HEADER).unwrap();
            stream.write(&buf).unwrap();
        },
        Err(e) => {
            // println!("failed to open {}: {}", path, e); // TODO
            println!("failed to open: {}", e);
            stream.write(not_found()).unwrap();
        }
    }
}

fn cgi(path: &str, args: &str) {

}

fn main() {
    let port = 30528;
    let addr = "127.0.0.1";
    let addr = addr.to_string() + ":" + &port.to_string();

    println!("listening on {}", addr);

    let listenser = TcpListener::bind(&*addr).unwrap();

    for stream in listenser.incoming() {
        match stream {
            Ok(stream) => {
                accept(stream);
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}

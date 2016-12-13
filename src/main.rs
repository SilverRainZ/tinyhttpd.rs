use std::str;
use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};

static WELCOME: &'static str = "HTTP/1.0 200 OK\r\n\
     Content-type: text/html; charset=utf-8\r\n\
     \r\n\
     <html>
        <head>
            <title>君の名は!</title>
        </head>
        <body>
            <center>
                <h1>Welcome!</h1>
                <p>君の名は!</p>
            </center>
        </body>
     </html>
     \0";

struct HttpRequest {
    /* request line */
    method: String,
    rawuri: String,
    version: String,
    // args

    /* request header */
    // header: HttpHeader,
    content_type: String,
    content_length: u32,

    /* request body */
    body: String,
}

fn accept_request(mut stream: TcpStream) {
    let mut buf = [0; 512];
    // let mut req: HttpRequest = HttpRequest{ .. };

    stream.read(&mut buf).unwrap();

    let s = match str::from_utf8(&buf) {
        Ok(v) => v,
        Err(e) => panic!("invaild utf-8 sequcence {}", e),
    };

    let mut line = s.split("\r\n");

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
    let req_header = match line.next() {
        Some(v) => v,
        None => panic!("no request header found"),
    };
    println!("request header: {}", req_header);

    stream.write(WELCOME.as_bytes()).unwrap();
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
                accept_request(stream);
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}

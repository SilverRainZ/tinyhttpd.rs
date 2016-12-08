use std::str;
use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};

static WELCOME: &'static str = "HTTP/1.0 200 OK\r\n\
     Content-type: text/html; charset=utf-8\r\n\
     \r\n\
     <html>\
        <head>\
            <title>君の名は!</title>
        </head>\
        <body>
            <center>
                <h1>Welcome!</h1>\
                <p>君の名は!</p>\
            </center>
        </body>
     </html>
     \0";

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 512];
    stream.read(&mut buf).unwrap();

    let s = match str::from_utf8(&buf) {
        Ok(v) => v,
        Err(e) => panic!("invaild utf-8 sequcence {}", e),
    };
    println!("recv: {}", s);

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
                handle_client(stream);
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}

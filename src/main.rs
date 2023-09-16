use std::{fs, thread};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use rust_htmx_ssr_template::ThreadPool;

fn main() {
    // unwrap to panic if there is an error variant
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        println!("Connection established!");
        match stream {
            Ok(unwrapped_stream) => {
                pool.execute(|| {
                    handle_connection(unwrapped_stream);
                });
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get_default = b"GET / HTTP/1.1\r\n";
    let get_default_but_throttled = b"GET /htmx-throttled HTTP/1.1\r\n";
    let static_sample_form_snippet = b"GET /sample-form-snippet HTTP/1.1\r\n";
    let editable_sample_form_snippet = b"GET /sample-form-snippet/edit HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(static_sample_form_snippet) {
        ("HTTP/1.1 200 OK", "src/components/sample-form-snippet/static.html")
    } else if buffer.starts_with(editable_sample_form_snippet) {
        ("HTTP/1.1 200 OK", "src/components/sample-form-snippet/editable.html")
    } else if buffer.starts_with(get_default) {
        ("HTTP/1.1 200 OK", "src/pages/home/index.html")
    } else if buffer.starts_with(get_default_but_throttled) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "src/pages/home/index.html")
    } else {
        ("HTTP/1.1 404 Not Found", "src/pages/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

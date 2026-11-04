use std::collections::HashMap;

use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;
use std::io::Read;
use std::io::Write;
use std::io::Result;
use f_string::t;
use mime_guess::MimeGuess;
use serde_yaml::to_string;

use std::net::{TcpListener, TcpStream};

use http_rust::httprequest::HttpRequest;
use http_rust::httpresponse::HttpResponse;
use http_rust::httprequest::Method;
use http_rust::httprequest::Resource;

use mime_guess;

mod args;
mod config_reader;

use crate::config_reader::AppConfig;
fn start_tcplistener(ip: &str, port: &str) -> TcpListener {
    TcpListener::bind(t!({ip}:{port})).unwrap()
}

fn handle_connection(mut stream: TcpStream, config: &AppConfig) {
    let mut data_buffer: [u8; 1024] = [0u8; 1024];
    let byte_read_attempt: Result<usize> = stream.read(&mut data_buffer);
    
    match byte_read_attempt {
        Ok(size) => {
           let http_request: String = String::from_utf8_lossy(&data_buffer[..size]).into_owned();
           handle_http_request(stream, http_request, config);

        }
        Err(_) => {
            return;
        }
    };

}

fn handle_http_request(stream: TcpStream, request: String, config: &AppConfig) {
    let http_request_object = HttpRequest::from(request);
    let mut mime_buffer = String::new();
    let http_response = match http_request_object.method {
        Method::Get => {
            handle_get(http_request_object.resource, config, &mut mime_buffer)
        }
        _ => { dbg!(http_request_object);
            return;
        }
    };

    write_http_request_to_tcp_stream(stream, http_response);
}

fn handle_get<'a>(resource: Resource, config: &'a AppConfig, mime_buffer: &'a mut String) -> HttpResponse<'a> { 
    let mut root = String::from(&config.server.root);
    let index = String::from(&config.server.index);
    let resource_path = match resource {
        Resource::Path(path) => path,
    };

    root.push_str(&resource_path);
    let mut actual_path = root;

    if Path::new(&actual_path).is_dir() { actual_path.push_str(&index); }
    dbg!(&actual_path);

    let extension = match Path::new(&actual_path).extension() {
        Some(x) => { x },
        None => {
            return HttpResponse::new("404", None, Some(String::from("Invalid extension type.")));
        }
    };
    let mime_type_for_resource = match get_mime_type(extension).first() {
        Some(x) => { x },
        None => {
            return HttpResponse::new("404", None, Some(String::from("Invalid MIME type.")));
        }
    };
    
    *mime_buffer = mime_type_for_resource.to_string();

    let headers = Some({
        let mut hashmap = HashMap::new();
        hashmap.insert("Content-Type", mime_buffer.as_str());
        hashmap
    });

    let file = File::open(&actual_path);
    let resolvedfile: Option<File>;
    resolvedfile = match file {
        Ok(x) => {Some(x)},
        Err(_) => {
            return HttpResponse::new("404", headers, Some(String::from("error")));
        }
    };

    let mut raw_bytes = Vec::new(); 
    resolvedfile.unwrap().read_to_end(&mut raw_bytes).unwrap();

    let file_content = String::from_utf8_lossy(&raw_bytes).to_string();

    HttpResponse::new("200", None, Some(file_content))
}


fn write_http_request_to_tcp_stream(mut stream: TcpStream, response: HttpResponse) {
    let mut write_stream = Vec::new();
    response.send_response(&mut write_stream).unwrap();
    stream.write_all(&write_stream).unwrap();
}

fn get_mime_type(extension: &OsStr) -> MimeGuess {
    mime_guess::from_ext(extension.to_str().unwrap())
}

fn main() {
    let args = args::get_args();
    let ip = args.ip;
    let port = args.port;

    let config = config_reader::get_config();

    let tcp_listener = start_tcplistener(ip.as_str(), port.as_str());
    println!("Started at {ip}:{port}");
    loop {
        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_connection(stream, &config);
                }
                Err(e) => {
                    println!("{}",t!(An error occured while trying to receive a connection: {e}))
                }
            }
        }
    }
}

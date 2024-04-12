use rust_webserver::{debug, error, info};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};
struct Config {
    address: String,
    port: String,
    path_to_resources: PathBuf,
}

enum HTTPStatus {
    Ok,
    InvalidRequest,
    NotFound,
}

fn get_status_line_and_file_from_http_status(error: HTTPStatus) -> (&'static str, &'static str) {
    /*
    Get the status line and file path for a given HTTP status.
     */
    match error {
        HTTPStatus::Ok => ("HTTP/1.1 200 OK", "index.html"),
        HTTPStatus::InvalidRequest => ("HTTP/1.1 400 Bad Request", "400.html"),
        HTTPStatus::NotFound => ("HTTP/1.1 404 Not Found", "404.html"),
    }
}

fn get_env_var(key: &str, default: &str) -> String {
    /*
    Get the value of an environment variable by key.
    If the key does not exist, return an empty string.
     */
    std::env::var(key).unwrap_or(default.to_string())
}

fn get_config() -> Config {
    /*
    Get the configuration for the webserver.
    The configuration is read from environment variables.
    If the environment variables are not set, default values are used.
     */

    // Set the path to the resources directory
    let mut path_to_resources = PathBuf::from(get_env_var("CARGO_MANIFEST_DIR", "."));
    path_to_resources.push("res");

    Config {
        address: get_env_var("ADDRESS", "127.0.0.1"),
        port: get_env_var("PORT", "8080"),
        path_to_resources,
    }
}

fn validate_request(request: BufReader<&TcpStream>) -> Option<HTTPStatus> {
    /* Validate the request from the client. */
    let request = match request.lines().next() {
        Some(line) => line,
        None => return None,
    };

    let request = match request {
        Ok(request) => request,
        Err(_) => return None,
    };

    let parts: Vec<&str> = request.split_whitespace().collect();
    if parts.len() != 3 {
        return Some(HTTPStatus::InvalidRequest);
    }

    let method = parts[0];
    let uri = parts[1];
    let version = parts[2];

    if method != "GET" {
        return Some(HTTPStatus::InvalidRequest);
    }

    if uri != "/" {
        return Some(HTTPStatus::NotFound);
    }

    if version != "HTTP/1.1" {
        return Some(HTTPStatus::InvalidRequest);
    }

    Some(HTTPStatus::Ok)
}

fn get_file_contents(path: PathBuf) -> String {
    /*
    Get the contents of a file. Returns the content of the file if found or 500 Internal Server Error as html.
     */

    match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            error!("Error reading file: {}", e);
            "<DOCTYPE html><html><head></head><body><h1>500 Internal Server Error</h1></body></html>".to_string()
        }
    }
}

fn construct_respoonse(status_line: &str, contents: &str) -> String {
    /*
    Construct the response to send to the client.
     */
    format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    )
}

fn handle_connection(
    mut stream: TcpStream,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = stream.peer_addr()?;
    debug!("Connection from {}", addr);

    // read the request from the client
    let buf_reader = BufReader::new(&stream);

    // validate the request
    match validate_request(buf_reader) {
        Some(HTTPStatus::Ok) => {
            let (status_line, file) = get_status_line_and_file_from_http_status(HTTPStatus::Ok);
            let contents = get_file_contents(config.path_to_resources.join(file));
            let response = construct_respoonse(status_line, &contents);
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }
        Some(HTTPStatus::InvalidRequest) => {
            let (status_line, file) =
                get_status_line_and_file_from_http_status(HTTPStatus::InvalidRequest);
            let contents = get_file_contents(config.path_to_resources.join(file));
            let response = construct_respoonse(status_line, &contents);
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }
        Some(HTTPStatus::NotFound) => {
            let (status_line, file) =
                get_status_line_and_file_from_http_status(HTTPStatus::NotFound);
            let contents = get_file_contents(config.path_to_resources.join(file));
            let response = construct_respoonse(status_line, &contents);
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }
        None => {
            let (status_line, file) =
                get_status_line_and_file_from_http_status(HTTPStatus::InvalidRequest);
            let contents = get_file_contents(config.path_to_resources.join(file));
            let response = construct_respoonse(status_line, &contents);
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get the configuration for the webserver
    let config = get_config();

    // start the webserver
    let listener = TcpListener::bind(format!("{}:{}", config.address, config.port))?;

    // log the address and port the webserver is listening on
    info!("Starting webserver on {}:{}", config.address, config.port);
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                error!("Error accepting connection: {}.", e);
                continue;
            }
        };

        if let Err(e) = handle_connection(stream, &config) {
            error!("Error handling connection: {}", e);
        }
    }

    Ok(())
}

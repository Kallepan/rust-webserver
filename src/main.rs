use rust_webserver::{debug, error, info, router::router::Router, thread::ThreadPool, warn};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::Arc,
};
struct Config {
    address: String,
    port: String,
    path_to_resources: PathBuf,
}

#[derive(Debug)]
enum HTTPError {
    InvalidRequest,
    NotFound,
}

fn get_status_line_and_file_from_http_status(error: HTTPError) -> (&'static str, &'static str) {
    /*
    Get the status line and file path for a given HTTP status.
     */
    match error {
        HTTPError::InvalidRequest => ("HTTP/1.1 400 Bad Request", "400.html"),
        HTTPError::NotFound => ("HTTP/1.1 404 Not Found", "404.html"),
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

fn validate_request(request: BufReader<&TcpStream>) -> Result<(String, String, String), HTTPError> {
    /* Validate the request from the client.
     * The request must be a GET request with the HTTP version 1.1.
     * If the request is valid, return the method, uri, and version.
     * If the request is invalid, return an error corresponding to the HTTP status code.
     */
    let request = match request.lines().next() {
        Some(line) => line,
        None => return Err(HTTPError::InvalidRequest),
    };

    let request = match request {
        Ok(request) => request,
        Err(_) => return Err(HTTPError::InvalidRequest),
    };

    let parts: Vec<&str> = request.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(HTTPError::InvalidRequest);
    }

    let method = parts[0];
    let uri = parts[1];
    let version = parts[2];

    if method != "GET" {
        return Err(HTTPError::InvalidRequest);
    }

    if version != "HTTP/1.1" {
        return Err(HTTPError::InvalidRequest);
    }

    Ok((method.to_string(), uri.to_string(), version.to_string()))
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
    router: &Router,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = stream.peer_addr()?;
    debug!("Connection from {}", addr);

    // read the request from the client
    let buf_reader = BufReader::new(&stream);

    // validate the request
    let (method, uri, version) = match validate_request(buf_reader) {
        Ok((method, uri, version)) => (method, uri, version),
        Err(e) => {
            warn!("Error validating request: {:?}", e);
            let (status_line, file) = get_status_line_and_file_from_http_status(e);
            let contents = get_file_contents(config.path_to_resources.join(file));
            let response = construct_respoonse(status_line, &contents);
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }
    };

    debug!("Request: {} {} {}", method, uri, version);
    let status_line = "HTTP/1.1 200 OK";
    let (status_line, file) = match router.get_route(&method, &uri) {
        Some(handler) => {
            let file = handler().unwrap();
            (status_line, file)
        }
        None => {
            let (status_line, file) =
                get_status_line_and_file_from_http_status(HTTPError::NotFound);
            (status_line, file.to_string())
        }
    };
    let contents = get_file_contents(config.path_to_resources.join(file));
    let response = construct_respoonse(status_line, &contents);
    stream.write_all(response.as_bytes())?;

    return Ok(());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get the configuration for the webserver
    let config = Arc::new(get_config());

    // configure the router
    let mut router = Router::new();
    router.add_route("GET", "/", || Some("index.html".to_string()));
    router.add_route("GET", "/sleep", || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        Some("index.html".to_string())
    });
    let router = Arc::new(router);

    // configure the thread pool
    let thread_pool = ThreadPool::new(4);

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

        let config = Arc::clone(&config);
        let router = Arc::clone(&router);
        thread_pool.execute(move || {
            if let Err(e) = handle_connection(stream, &config, &router) {
                error!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

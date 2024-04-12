use rust_webserver::{debug, error, info};
use std::{
    io::{BufRead, BufReader, Error, ErrorKind, Write},
    net::{TcpListener, TcpStream},
};
struct Config {
    address: String,
    port: String,
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
    Config {
        address: get_env_var("ADDRESS", "127.0.0.1"),
        port: get_env_var("PORT", "8080"),
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let addr = stream.peer_addr()?;
    debug!("Connection from {}", addr);

    let buf_reader = BufReader::new(&stream);

    // read request
    let http_request: Vec<_> = buf_reader
        .lines()
        .enumerate()
        // filter out invalid lines
        .filter_map(|(i, line)| match line {
            Ok(line) => Some(line),
            Err(_) => {
                error!("Error reading line {}", i);
                None
            }
        })
        // stop at the first empty line
        .take_while(|line| !line.is_empty())
        .collect();

    // check if request is empty
    if http_request.is_empty() {
        return Err(Box::new(Error::new(
            ErrorKind::InvalidInput,
            "Invalid request",
        )));
    }
    debug!("Request: {:#?}", http_request);

    // send response
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes())?;

    Ok(())
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

        if let Err(e) = handle_connection(stream) {
            error!("Error handling connection: {}", e);
        }
    }

    Ok(())
}

use std::{
    fs,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
};

use regex::Regex;

fn main() {
    let port = 7878;
    let address = format!("localhost:{port}");
    let listener =
        TcpListener::bind(&address).expect(&format!("Unable to bind to address {address}"));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) -> () {
    let reader = BufReader::new(&mut stream);
    let request: Vec<_> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Received request: {:#?}", request);

    let (status, response) = handle_request(request);
    stream.write_all(response.as_bytes()).unwrap();

    println!("Sent response status: {:#?}", status);
}

fn handle_request(request: Vec<String>) -> (String, String) {
    return match request.first().and_then(|r| decompose_request(&r)) {
        Some((_, _, ver)) if ver != "HTTP/1.1" => empty_response("505 HTTP Version Not Supported"),
        Some((method, _, _)) if method != "GET" => empty_response("405 Method Not Allowed"),
        Some((_, route, _)) => match retrieve_page_content(route) {
            Ok(content) => response_with_content("200 OK", &content),
            Err(e) if e.kind() == ErrorKind::NotFound => empty_response("404 Not Found"),
            _ => empty_response("500 Internal Server Error"),
        },
        _ => empty_response("400 Bad Request"),
    };
}

fn retrieve_page_content(route: &str) -> Result<String, std::io::Error> {
    let file_path = if route == "/" {
        format!("pages/index.html")
    } else {
        format!("pages/{route}.html")
    };

    fs::read_to_string(file_path)
}

fn decompose_request(request: &str) -> Option<(&str, &str, &str)> {
    let request_regex = Regex::new(r"^([A-Z]+) (\/.*) (HTTP\/1.1)$").unwrap();
    let matches = request_regex.captures_iter(request).next()?;
    let method = matches.get(1)?.as_str();
    let route = matches.get(2)?.as_str();
    let version = matches.get(3)?.as_str();

    Some((method, route, version))
}

fn empty_response(status: &str) -> (String, String) {
    let header = format!("HTTP/1.1 {status}");
    let response = format!("{status}\r\n\r\n");
    (header, response)
}

fn response_with_content(status: &str, content: &str) -> (String, String) {
    let status = format!("HTTP/1.1 {status}");
    let headers = format!("{status}\r\nContent-Length: {0}", content.len());
    let response = format!("{headers}\r\n\r\n{content}");
    (status, response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decompose_request_with_custom_route() {
        let request = "GET /test HTTP/1.1";
        let (method, route, version) = decompose_request(request).unwrap();
        assert_eq!(method, "GET");
        assert_eq!(route, "/test");
        assert_eq!(version, "HTTP/1.1");
    }
}

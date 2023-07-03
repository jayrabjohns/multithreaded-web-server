mod thread_pool;

use regex::Regex;
use std::{
    fs,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
};
use thread_pool::ThreadPool;

pub fn run(address: &str) {
    let listener =
        TcpListener::bind(address).expect(&format!("Unable to bind to address {address}"));
    let pool = ThreadPool::new(4).expect(&format!(
        "Unable to spawn threads for underlying thread pool"
    ));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
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

    println!("Sent response status: {status}");
}

fn handle_request(request: Vec<String>) -> (usize, String) {
    return match request.first().and_then(|r| decompose_request(&r)) {
        Some((_, _, ver)) if ver != "1.1" => {
            (505, empty_response("505 HTTP Version Not Supported"))
        }
        Some((method, _, _)) if method != "GET" => (405, empty_response("405 Method Not Allowed")),
        Some((_, route, _)) => match retrieve_page_content(route) {
            Ok(content) => (200, response_with_content("200 OK", &content)),
            Err(e) if e.kind() == ErrorKind::NotFound => (404, empty_response("404 Not Found")),
            _ => (500, empty_response("500 Internal Server Error")),
        },
        _ => (400, empty_response("400 Bad Request")),
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
    let request_regex = Regex::new(r"^([A-Z]+) (\/.*) HTTP\/(\d+.\d+)$").unwrap();
    let matches = request_regex.captures_iter(request).next()?;
    let method = matches.get(1)?.as_str();
    let route = matches.get(2)?.as_str();
    let version = matches.get(3)?.as_str();

    Some((method, route, version))
}

fn empty_response(status: &str) -> String {
    format!("HTTP/1.1 {status}\r\n\r\n")
}

fn response_with_content(status: &str, content: &str) -> String {
    let headers = format!("HTTP/1.1 {status}\r\nContent-Length: {0}", content.len());
    format!("{headers}\r\n\r\n{content}")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn handle_request_returns_400_for_malformed_body() {
        test_handle_request("'; SELECT * FROM users;", 400);
    }

    #[test]
    fn handle_request_returns_404_for_nonexistent_page() {
        test_handle_request("GET /doesntexist HTTP/1.1", 404)
    }

    #[test]
    fn handle_request_returns_505_for_incompatible_http_version() {
        test_handle_request("GET / HTTP/2.0", 505);
    }

    #[test]
    fn decompose_request_with_custom_route() {
        let request = "GET /test HTTP/1.1";
        let (method, route, version) = decompose_request(request).unwrap();

        assert_eq!(method, "GET");
        assert_eq!(route, "/test");
        assert_eq!(version, "1.1");
    }

    fn test_handle_request(request: &str, expected_status_code: usize) {
        let (status, _) = handle_request(vec![String::from(request)]);
        assert_eq!(status, expected_status_code);
    }
}

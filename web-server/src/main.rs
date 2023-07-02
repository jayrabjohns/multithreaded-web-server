fn main() {
    let port = 7878;
    let address = format!("localhost:{port}");
    web_server::run(&address)
}

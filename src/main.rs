#[macro_use]
extern crate rouille;

fn main() {
    println!("Hi there!");
    let server = rouille::Server::new_ssl(
        "127.0.0.1:8080",
        move |request| {
            router!(request,
                (GET) (/) => {
                    rouille::Response::text("Hi!")
                },
                _ => rouille::Response::empty_404()
            )
        },
        include_bytes!("cert.pem"),
        include_bytes!("key.pem"),
    );
    rouille::start_server("127.0.0.1:8080", move |request| {
        router!(request,
            (GET) (/) => {
                rouille::Response::text("Hi!")
            },
            _ => rouille::Response::empty_404()
        )
    });
}

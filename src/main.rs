#[macro_use]
extern crate rouille;

fn main() {
    println!("Hi there!");
    rouille::start_server("127.0.0.1:8080", move |request| {
        router!(request,
            (GET) (/) => {
                rouille::Response::text("hello world")
            },
            _ => rouille::Response::empty_404()
        )
    });
}

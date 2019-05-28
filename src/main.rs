#![deny(warnings)]

mod cert;
mod err;
mod fly;
mod serve;

use fly::*;

fn main() -> Try {
    let cert_pair = get_certs()?;
    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server(&cert_pair) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

#![deny(warnings)]

mod cert;
mod err;
mod fly;
mod serve;

use clap::{App, SubCommand};
use fly::*;

fn main() -> Try {
    let mut app = App::new("fly")
        .subcommand(SubCommand::with_name("join"));
    let matches = app.clone().get_matches();
    match matches.subcommand() {
        ("join", Some(_sub_matches)) => {
            run()
        }
        _ => {
            app.print_help()?;
            println!();
            Ok(())
        }
    }
}

fn run() -> Try {
    let cert_pair = get_certs()?;
    if let Err(e) = run_server(&cert_pair) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

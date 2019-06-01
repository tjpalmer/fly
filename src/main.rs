#![deny(warnings)]

mod cert;
mod err;
mod fly;
mod serve;

use clap::{App, Arg, SubCommand};
use fly::*;

fn main() -> Try {
    let mut app = App::new("fly")
        .subcommand(
            SubCommand::with_name("join")
            .arg(Arg::with_name("nodes").multiple(true)));
    let matches = app.clone().get_matches();
    match matches.subcommand() {
        ("join", Some(sub_matches)) => {
            if let Some(nodes) = sub_matches.values_of_lossy("nodes") {
                dbg!(&nodes);
                if nodes.len() > 0 {
                    let uri = format!("https://{}:1337", nodes[0]);
                    let _response = reqwest::get(&uri)?;
                }
                Ok(())
            } else {
                run()
            }
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

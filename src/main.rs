#![deny(warnings)]

mod cert;
mod err;
mod fetch;
mod fly;
mod serve;
mod spread;

use clap::{App, Arg, SubCommand};
use fly::*;

fn main() -> Try {
    let mut app = App::new("fly")
        .subcommand(
            SubCommand::with_name("join")
            .arg(Arg::with_name("nodes").multiple(true)))
        .subcommand(
            SubCommand::with_name("spread")
            .arg(Arg::with_name("nodes").multiple(true).required(true)))
        ;
    let matches = app.clone().get_matches();
    match matches.subcommand() {
        ("join", Some(sub_matches)) => {
            if let Some(nodes) = sub_matches.values_of_lossy("nodes") {
                dbg!(&nodes);
                if nodes.len() > 0 {
                    let uri = format!("https://{}:1337", nodes[0]);
                    fetch(&uri)?;
                }
                Ok(())
            } else {
                run()
            }
        }
        ("spread", Some(sub_matches)) => {
            // TODO This command might go away and be reorged after testing.
            // TODO Others, too, I guess ...
            // We can unwrap because required above.
            let nodes = sub_matches.values_of_lossy("nodes").unwrap();
            for node in &nodes {
                spread(&node)?;
            }
            Ok(())
        }
        _ => {
            app.print_help()?;
            println!();
            Ok(())
        }
    }
}

fn run() -> Try {
    let ca_cert_pair = get_certs("ca", None)?;
    let cert_pair = get_certs("node", Some(&ca_cert_pair))?;
    if let Err(e) = run_server(&cert_pair) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

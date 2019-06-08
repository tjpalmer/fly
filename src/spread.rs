use crate::fly::*;
use std::process::{Command, Stdio};
use std::{io::Write, path::Path, str};

// This uses external processes because:
// 1. I'm not convinced I like the Rust options for ssh/scp/sftp
// 2. It's more like a driver for certain init cases, not core usage
//
// TODO How best to spread to Windows nodes?

pub fn spread(host: &str) -> Try {
    // TODO First try to contact/update through an existing fly agent.
    // TODO Check out the general state of things.
    // Copy the exe.
    // TODO Copy it to the right place.
    if cfg!(windows) {
        // TODO First see what the host is, anyway.
        return Err(error("need a linux binary to spread to linux"));
    }
    scp(host, std::env::current_exe()?, "")?;
    // TODO Generate a new cert for the node, and copy it, too.
    // Test ssh commands.
    // TODO Kick off the server.
    // This hangs in Windows.
    // let output = Command::new("ssh").args(&[host, "ls", "-lh"]).output()?;
    // But the extra steps version doesn't ...
    // TODO Why not?
    let mut child =
        Command::new("ssh")
        .args(&[host, "bash", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdin = child.stdin.as_mut().ok_or(error("no ssh stdin"))?;
    write!(stdin, "ls -l")?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        println!("failed with status {:?}", output.status.code());
    }
    println!("{}", str::from_utf8(&output.stdout)?);
    println!("{}", str::from_utf8(&output.stderr)?);
    Ok(())
}

fn scp<P: AsRef<Path>>(host: &str, from: P, to: &str) -> Try {
    let child =
        Command::new("scp")
        .args(&[
            from.as_ref().to_str().ok_or(error("bad path"))?,
            &format!("{}:{}", host, to)
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        println!("failed with status {:?}", output.status.code());
    }
    Ok(())
}

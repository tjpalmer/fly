use crate::fly::*;
use std::process::{Command, Stdio};
use std::str;

pub fn spread(host: &str) -> Try {
    // TODO This isn't working in Windows, even though `ssh <host> ls -lh` works.
    // let output = Command::new("ssh").args(&[host, "ls", "-lh"]).output()?;
    let child =
        Command::new("ssh")
        .args(&[host, "ls", "-lh"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // let output = Command::new("ssh").output()?;
    // let output = Command::new("dir").output()?;
    // let output = Command::new("rustc").output()?;
    // let stdout = str::from_utf8(&output.stdout)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        println!("failed with status {:?}", output.status.code());
    }
    println!("{}", str::from_utf8(&output.stdout)?);
    println!("{}", str::from_utf8(&output.stderr)?);
    Ok(())
}

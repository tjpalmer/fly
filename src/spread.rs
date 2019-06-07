use crate::fly::*;
use std::process::Command;
use std::str;

pub fn spread(host: &str) -> Try {
  // TODO This isn't working in Windows, even though `ssh <host> ls -lh` works.
  let output = Command::new("ssh").args(&[host, "ls", "-lh"]).output()?;
  // let output = Command::new("ssh").output()?;
  // let output = Command::new("dir").output()?;
  let stdout = str::from_utf8(&output.stdout)?;
  println!("{}", stdout);
  Ok(())
}

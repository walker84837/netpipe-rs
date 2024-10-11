use anyhow::Result;
use log::info;
use std::{
    io::{self, Read, Write},
    process::{Command, Stdio},
};

pub fn execute_command<R: Read>(mut input: R, command: &str) -> Result<()> {
    info!("Executing command: {}", command);
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer)?;
        stdin.write_all(&buffer)?;
    }

    let output = child.wait_with_output()?;
    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;
    Ok(())
}

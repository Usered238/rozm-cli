//! CLI argument validation and synthesis settings.

use std::path::PathBuf;
use std::{ffi::OsString, io};

pub const DEFAULT_OUTPUT: &str = "./out.wav";
pub const DEFAULT_VOICE: u8 = 1;
pub const DEFAULT_SPEED: u8 = 5;

#[derive(Debug)]
pub struct SynthesisRequest {
    pub text: String,
    pub stdin: bool,
    pub output: PathBuf,
    pub voice: u8,
    pub speed: u8,
}

pub fn parse(
    arguments: impl IntoIterator<Item = OsString>,
) -> io::Result<Option<SynthesisRequest>> {
    let mut arguments = arguments.into_iter();
    let mut text = None;
    let mut stdin = false;
    let mut output = PathBuf::from(DEFAULT_OUTPUT);
    let mut voice = DEFAULT_VOICE;
    let mut speed = DEFAULT_SPEED;
    let mut seen = std::collections::HashSet::new();
    while let Some(argument) = arguments.next() {
        let option = argument.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "option is not valid Unicode")
        })?;
        if option == "--help" || option == "-h" {
            return Ok(None);
        }
        if option == "--stdin" {
            if !seen.insert(option.to_owned()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "duplicate argument --stdin",
                ));
            }
            stdin = true;
            continue;
        }
        let (name, inline) = option
            .split_once('=')
            .map_or((option, None), |(name, value)| {
                (name, Some(OsString::from(value)))
            });
        if !matches!(name, "--text" | "--output" | "--voice" | "--speed") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown argument {name}"),
            ));
        }
        if !seen.insert(name.to_owned()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("duplicate argument {name}"),
            ));
        }
        let value = inline.or_else(|| arguments.next()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("missing value for {name}"),
            )
        })?;
        match name {
            "--text" => {
                text = Some(value.into_string().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "text is not valid Unicode")
                })?)
            }
            "--output" => output = value.into(),
            "--voice" | "--speed" => {
                let number = value
                    .to_str()
                    .and_then(|s| s.parse::<u8>().ok())
                    .filter(|&n| n >= 1 && n <= if name == "--voice" { 3 } else { 9 })
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!(
                                "{name} must be an integer in 1..{}",
                                if name == "--voice" { 3 } else { 9 }
                            ),
                        )
                    })?;
                if name == "--voice" {
                    voice = number;
                } else {
                    speed = number;
                }
            }
            _ => unreachable!(),
        }
    }
    if stdin && text.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--stdin and --text are mutually exclusive",
        ));
    }
    let text = if stdin {
        String::new()
    } else {
        text.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "required --text TEXT or --stdin",
            )
        })?
    };
    Ok(Some(SynthesisRequest {
        text,
        stdin,
        output,
        voice,
        speed,
    }))
}

//! CLI configuration. Parsing will follow the contract in PLAN.md.

use std::path::PathBuf;

pub const DEFAULT_OUTPUT: &str = "./out.wav";
pub const DEFAULT_VOICE: u8 = 1;
pub const DEFAULT_SPEED: u8 = 5;

#[derive(Debug)]
pub struct SynthesisRequest {
    pub text: String,
    pub output: PathBuf,
    pub voice: u8,
    pub speed: u8,
}

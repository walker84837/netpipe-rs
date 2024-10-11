use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about = "A Rust port of netcat", long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub file: Option<PathBuf>,

    #[clap(short, long, default_value = "4")]
    pub ip_version: u8,

    #[clap(
        short,
        long,
        default_value = "tcp",
        help = "The protocol to use. Possible choices: TCP|UDP"
    )]
    pub protocol: String,

    #[clap(short, long, default_value = "0", help = "Timeout in seconds")]
    pub timeout: u64,

    #[clap(short, long, help = "Listen mode")]
    pub listen: bool,

    #[clap(short, long, help = "Execute command")]
    pub exec: Option<String>,

    #[clap(short, long, help = "Logs to stdout")]
    pub verbose: bool,

    pub address: Option<String>,
    pub port: Option<u16>,
}

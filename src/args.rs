use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about = "A Rust port of netcat", long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub file: Option<PathBuf>,

    #[clap(short, long, default_value = "4", value_parser = clap::value_parser!(IpVersion))]
    pub ip_version: IpVersion,

    #[clap(
        short,
        long,
        default_value = "tcp",
        help = "The protocol to use. Possible choices: TCP|UDP"
    )]
    pub protocol: Protocol,

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

#[derive(ValueEnum, Clone, Debug)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum IpVersion {
    #[clap(name = "4")]
    V4,
    #[clap(name = "6")]
    V6,
}

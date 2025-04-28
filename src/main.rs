#![feature(ip)]

mod args;
mod command;
mod network;

use crate::{
    args::{Args, IpVersion, Protocol},
    network::{run_client, run_server},
};
use anyhow::{bail, Result};
use clap::Parser;
use log::info;
use std::time::Duration;

fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter_level(if args.verbose {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Off
        })
        .init();

    let timeout_duration = Duration::from_secs(args.timeout);

    info!("Starting application with arguments: {:#?}", args);

    // Validate address and port for both modes
    if args.listen && (args.address.is_none() || args.port.is_none()) {
        bail!("Listening mode requires both address and port to be specified.");
    } else if !args.listen && (args.address.is_none() || args.port.is_none()) {
        bail!("Client mode requires both address and port to be specified.");
    }

    if let Some(address) = &args.address {
        let ip_version = match args.ip_version {
            IpVersion::V4 => 4,
            IpVersion::V6 => 6,
        };
        if !network::is_valid_address(address, &ip_version) {
            bail!("Invalid IP address: {} for version {}", address, ip_version);
        }
    }

    let protocol = match &args.protocol {
        Protocol::Tcp => "tcp",
        Protocol::Udp => "udp",
    };

    if args.listen {
        run_server(&args, protocol, timeout_duration)?;
    } else {
        run_client(&args, protocol, timeout_duration)?;
    }

    Ok(())
}

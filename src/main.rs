#![feature(ip)]

mod args;
mod command;
mod network;

use crate::{
    args::Args,
    network::{run_client, run_server},
};
use anyhow::{bail, Result};
use clap::Parser;
use log::{/*error,*/ info};
use std::time::Duration;

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        simple_logger::SimpleLogger::new().init().unwrap();
    }

    let protocol = args.protocol.to_lowercase();
    let timeout_duration = Duration::from_secs(args.timeout);

    info!("Starting application with arguments: {:#?}", args);

    let port_and_address_not_provided =
        args.listen && (args.address.is_none() || args.port.is_none());

    if port_and_address_not_provided {
        bail!("Listening mode requires both address and port to be specified.");
    }

    if !port_and_address_not_provided {
        bail!("Client mode requires both address and port to be specified.");
    }

    if let Some(address) = &args.address {
        if !network::is_valid_address(address, &args.ip_version) {
            bail!(
                "Invalid IP address: {} for version {}",
                address,
                args.ip_version
            );
        }
    }

    if args.listen {
        run_server(&args, &protocol, timeout_duration)?;
    } else {
        run_client(&args, &protocol, timeout_duration)?;
    }

    Ok(())
}

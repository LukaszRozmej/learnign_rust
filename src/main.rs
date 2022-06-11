extern crate log;

use crate::checker::{BlocklistChecker, BlocklistStore};
use env_logger::{Builder, Target};
use log::LevelFilter;
use warp::Filter;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

mod persister;
mod checker;
mod downloader;

#[tokio::main]
async fn main() {
    Builder::new()
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .init();

    let persister = persister::BlocklistPersister {};
    let checker = checker::BlocklistCheckerStore::new(persister);
    let checker = Arc::new(checker);
    downloader::start(checker.clone());
    let checker2 = checker.clone();

    let ips = warp::path!("ips" / String)
        .map(move |ip : String| {
            match Ipv4Addr::from_str(&ip)
            {
                Ok(address) => {
                    let checker = checker2;
                    String::from(if checker.contains(&address) { "true"} else { "false" })
                },
                Err(error) => format!("{} is not correct IP address, {}!", &ip, error)
            }
        });

    warp::serve(ips)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
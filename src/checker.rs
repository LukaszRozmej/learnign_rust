use std::sync::RwLock;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::net::Ipv4Addr;
use crate::persister::BlocklistPersister;

pub trait BlocklistChecker {
    fn contains(&self, ip: &Ipv4Addr) -> bool;
}

pub trait BlocklistStore {
    fn set_addresses<I>(&self, addresses : I) where I : Iterator<Item=Ipv4Addr>;
}

pub struct BlocklistCheckerStore {
    addresses : RwLock<HashSet<Ipv4Addr>>,
    persister: BlocklistPersister,
}

impl BlocklistCheckerStore {
    pub fn new(persister: BlocklistPersister) -> Self
    {
        match persister.load() {
            Ok(iterator) => {
                log::info!("Loaded addresses from DB");
                Self {
                    addresses: RwLock::new(HashSet::from_iter(iterator)),
                    persister,
                }
            }
            Err(error) => {
                log::warn!("Failed to load DB on startup: {}.", error);
                Self {
                    addresses: RwLock::new(HashSet::new()),
                    persister,
                }
            }
        }
    }
}

impl BlocklistChecker for BlocklistCheckerStore {
    fn contains(&self, ip: &Ipv4Addr) -> bool {
        self.addresses.read().unwrap().contains(&ip)
    }
}

impl BlocklistStore for BlocklistCheckerStore {
    fn set_addresses<I>(&self, addresses: I) where I: Iterator<Item=Ipv4Addr> {
        // this is not thread safe if addresses is not behind a mutex
        // self.addresses = HashSet::from_iter(addresses); 

        let mut val  = self.addresses.write().unwrap();
        *val = HashSet::from_iter(addresses);

        log::info!("Successfully refreshed blocklist with {} ips.", val.len());
        match self.persister.persist(val.iter().map(|i| *i)) {
            Ok(_) => log::info!("Saved blocklist to DB"),
            Err(error) => log::error!("Failed to save DB: {}.", error)
        }
    }
}

unsafe impl Send for BlocklistCheckerStore { }
unsafe impl Sync for BlocklistCheckerStore { }
use std::collections::HashSet;
use std::iter::FromIterator;
use crate::persister::BlocklistPersister;

pub trait BlocklistChecker {
    fn contains(&self, ip: u32) -> bool;
}

pub trait BlocklistStore {
    fn set_addresses<I>(&mut self, addresses : I) where I : Iterator<Item=u32>;
}

pub struct BlocklistCheckerStore {
    addresses : HashSet<u32>,
    persister: BlocklistPersister,
}

impl BlocklistCheckerStore {
    pub fn new(persister: BlocklistPersister) -> Self
    {
        match persister.load() {
            Ok(iterator) => {
                log::info!("Loaded addresses from DB");
                Self {
                    addresses: HashSet::from_iter(iterator),
                    persister,
                }
            }
            Err(error) => {
                log::warn!("Failed to load DB on startup: {}.", error);
                Self {
                    addresses: HashSet::new(),
                    persister,
                }
            }
        }
    }
}

impl BlocklistChecker for BlocklistCheckerStore {
    fn contains(&self, ip: u32) -> bool {
        self.addresses.contains(&ip)
    }
}

impl BlocklistStore for BlocklistCheckerStore {
    fn set_addresses<I>(&mut self, addresses: I) where I: Iterator<Item=u32> {
        self.addresses = HashSet::from_iter(addresses);
        log::info!("Successfully refreshed blocklist with {} ips.", self.addresses.len());
        match self.persister.persist(self.addresses.iter().map(|i| *i)) {
            Ok(_) => (),
            Err(error) => log::error!("Failed to save DB: {}.", error)
        }
    }
}
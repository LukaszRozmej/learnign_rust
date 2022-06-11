use std::sync::Mutex;
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
    addresses : Mutex<HashSet<Ipv4Addr>>,
    persister: BlocklistPersister,
}

impl BlocklistCheckerStore {
    pub fn new(persister: BlocklistPersister) -> Self
    {
        match persister.load() {
            Ok(iterator) => {
                log::info!("Loaded addresses from DB");
                Self {
                    addresses: Mutex::new(HashSet::from_iter(iterator)),
                    persister,
                }
            }
            Err(error) => {
                log::warn!("Failed to load DB on startup: {}.", error);
                Self {
                    addresses: Mutex::new(HashSet::new()),
                    persister,
                }
            }
        }
    }
}

impl BlocklistChecker for BlocklistCheckerStore {
    fn contains(&self, ip: &Ipv4Addr) -> bool {
        self.addresses.lock().unwrap().contains(&ip)
    }
}

impl BlocklistStore for BlocklistCheckerStore {
    fn set_addresses<I>(&self, addresses: I) where I: Iterator<Item=Ipv4Addr> {
        // this is not thread safe if addresses is not behind a mutex
        // self.addresses = HashSet::from_iter(addresses); 

        let mut val  = self.addresses.lock().unwrap();
        *val = HashSet::from_iter(addresses);

        log::info!("Successfully refreshed blocklist with {} ips.", self.addresses.lock().unwrap().len());
        match self.persister.persist(self.addresses.lock().unwrap().iter().map(|i| *i)) {
            Ok(_) => log::info!("Saved blocklist to DB"),
            Err(error) => log::error!("Failed to save DB: {}.", error)
        }
    }
}

unsafe impl Send for BlocklistCheckerStore { }
unsafe impl Sync for BlocklistCheckerStore { }
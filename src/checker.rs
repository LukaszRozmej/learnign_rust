use std::cell::UnsafeCell;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::net::Ipv4Addr;
use crate::persister::BlocklistPersister;

pub trait BlocklistChecker {
    fn contains(&self, ip: &Ipv4Addr) -> bool;
}

pub trait BlocklistStore {
    fn set_addresses<I>(&mut self, addresses : I) where I : Iterator<Item=Ipv4Addr>;
}

pub struct BlocklistCheckerStore {
    addresses: UnsafeCell<HashSet<Ipv4Addr>>,
    persister: BlocklistPersister,
}

impl BlocklistCheckerStore {
    pub fn new(persister: BlocklistPersister) -> Self
    {
        match persister.load() {
            Ok(iterator) => {
                log::info!("Loaded addresses from DB");
                Self {
                    addresses: UnsafeCell::new(HashSet::from_iter(iterator)),
                    persister,
                }
            }
            Err(error) => {
                log::warn!("Failed to load DB on startup: {}.", error);
                Self {
                    addresses: UnsafeCell::new(HashSet::new()),
                    persister,
                }
            }
        }
    }
}

impl BlocklistChecker for BlocklistCheckerStore {
    fn contains(&self, ip: &Ipv4Addr) -> bool { unsafe { (*self.addresses.get()).contains(&ip) } }
}

impl BlocklistStore for BlocklistCheckerStore {
    fn set_addresses<I>(&self, addresses: I) where I: Iterator<Item=Ipv4Addr> {
        unsafe { *self.addresses.get() = HashSet::from_iter(addresses); }

        let addresses = unsafe { &*self.addresses.get() };

        log::info!("Successfully refreshed blocklist with {} ips.", addresses.len());
        match self.persister.persist(addresses.iter().map(|i| *i)) {
            Ok(_) => log::info!("Saved blocklist to DB"),
            Err(error) => log::error!("Failed to save DB: {}.", error)
        }
    }
}

unsafe impl Send for BlocklistCheckerStore { }
unsafe impl Sync for BlocklistCheckerStore { }
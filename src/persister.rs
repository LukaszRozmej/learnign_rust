use std::fs::File;
use std::io::{Error, Read, Write};
use std::net::Ipv4Addr;

const DB: &str = "db";

pub struct BlocklistPersister {}

struct FileIterator {
    file: File,
    buffer: [u8; std::mem::size_of::<u32>()]
}

impl BlocklistPersister {
    pub fn persist(&self, addresses: impl Iterator<Item=Ipv4Addr>) -> Result<(), Error> {
        let mut file = File::create(DB)?;
        for address in addresses {
            file.write(&u32::to_be_bytes(u32::from(address)))?;
        }

        Ok(())
    }

    pub fn load(&self) -> Result<impl Iterator<Item=Ipv4Addr>, Error> {
        let file = File::open(DB)?;
        Ok(FileIterator::new(file))
    }
}

impl FileIterator {
    fn new(file: File) -> FileIterator {
        FileIterator {
            file,
            buffer: [0; std::mem::size_of::<u32>()]
        }
    }
}

impl Iterator for FileIterator {
    type Item = Ipv4Addr;

    fn next(&mut self) -> Option<Self::Item> {
        let read = self.file.read(&mut self.buffer);
        match read {
            Ok(read) => {
                if read == 4 {
                    Some(Ipv4Addr::from(u32::from_be_bytes(self.buffer)))
                } else {
                    None
                }
            }
            Err(error) => {
                log::warn!("DB might be corrupted: {}.", error);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let persister = BlocklistPersister {};
        let addresses = vec![0u32, 1u32];
        persister.persist(addresses.iter().copied())?;
        let iterator = persister.load()?;
        let it = addresses.into_iter();
        assert!(it.eq(iterator));
    }
}
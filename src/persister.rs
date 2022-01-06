use std::fs::File;
use std::io::{Error, Read, Write};

const DB: &str = "db";

pub struct BlocklistPersister {}

struct FileIterator {
    file: File,
    buffer: [u8; std::mem::size_of::<u32>()]
}

impl BlocklistPersister {
    pub fn persist(&self, addresses: impl Iterator<Item=u32>) -> Result<(), Error> {
        let file = File::create(DB);
        return match file {
            Ok(mut file) => {
                for address in addresses {
                    match file.write(&u32::to_be_bytes(address)) {
                        Ok(_) => {}
                        Err(error) => return Err(error)
                    }
                }

                Ok(())
            }
            Err(error) => Err(error)
        };
    }

    pub fn load(&self) -> Result<impl Iterator<Item=u32>, Error> {
        let file = File::open(DB);
        return match file {
            Ok(file) => {
                return Ok(FileIterator::new(file));
            }
            Err(error) => Err(error)
        };
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
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let read = self.file.read(&mut self.buffer);
        match read {
            Ok(read) => {
                if read == 4 {
                    Some(u32::from_be_bytes(self.buffer))
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
    fn test_roundtrip() -> Result<(), Error> {
        let persister = BlocklistPersister {};
        let addresses = vec![0u32, 1u32];
        match persister.persist(addresses.iter().copied())
        {
            Ok(_) => {
                match persister.load() {
                    Ok(iterator) => {
                        let it = addresses.into_iter();
                        assert!(it.eq(iterator));
                        Ok(())
                    }
                    Err(error) => Err(error)
                }
            }
            Err(error) => Err(error)
        }
    }
}
use crate::*;

const HASH_SIZE: usize = 16;

/// Hash is used for content-based hashing
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hash(Vec<bool>);

impl std::fmt::Display for Hash {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:x}", self)
    }
}

fn to_byte(b: &[bool]) -> u8 {
    let mut dest = 0;
    for (i, x) in b.iter().enumerate() {
        if *x {
            dest |= 1 << i;
        }
    }
    dest
}

impl std::fmt::LowerHex for Hash {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in self.0.chunks(8).map(to_byte) {
            write!(fmt, "{:x}", c)?
        }
        Ok(())
    }
}

impl Hash {
    /// Compute difference between two hashes
    pub fn diff(&self, other: &Hash) -> usize {
        let mut diff = 0;

        for (i, x) in self.0.iter().enumerate() {
            if other.0[i] != *x {
                diff += 1;
            }
        }

        diff
    }
}

impl From<Hash> for String {
    fn from(hash: Hash) -> String {
        format!("{}", hash)
    }
}

impl From<Hash> for Vec<bool> {
    fn from(hash: Hash) -> Vec<bool> {
        hash.0
    }
}

impl<T: Type, C: Color> Image<T, C> {
    /// Get image hash
    pub fn hash(&self) -> Hash {
        let small: Image<T, C> = self.resize((HASH_SIZE, HASH_SIZE));
        let mut hash = vec![false; HASH_SIZE * HASH_SIZE];
        let mut index = 0;
        let mut px = Pixel::new();
        for j in 0..HASH_SIZE {
            for i in 0..HASH_SIZE {
                small.pixel_at((i, j), &mut px);
                if px.iter().sum::<f64>() / C::CHANNELS as f64 > 0.5 {
                    hash[index] = true;
                }
                index += 1
            }
        }
        Hash(hash)
    }
}

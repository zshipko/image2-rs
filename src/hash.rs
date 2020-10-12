use crate::*;

/// Hash is used for content-based hashing
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hash(u128);

fn check_bit(number: u128, n: usize) -> bool {
    (number >> n) & 1 == 0
}

impl Hash {
    /// Compute difference between two hashes
    pub fn diff(&self, other: &Hash) -> u128 {
        let mut diff = 0;

        for i in 0..128 {
            if check_bit(self.0, i) != check_bit(other.0, i) {
                diff += 1;
            }
        }

        diff
    }
}

impl From<Hash> for String {
    fn from(hash: Hash) -> String {
        format!("{:016x}", hash.0)
    }
}

impl From<Hash> for u128 {
    fn from(hash: Hash) -> u128 {
        hash.0
    }
}

impl<T: Type, C: Color> Image<T, C> {
    /// Get image hash
    pub fn hash(&self) -> Hash {
        let small: Image<T, C> = self.resize((16, 8));
        let mut hash = 0u128;
        let mut index = 0;
        let mut px = Pixel::new();
        for j in 0..8 {
            for i in 0..16 {
                small.pixel_at((i, j), &mut px);
                let avg: f64 = px.iter().sum();
                let f = avg / C::CHANNELS as f64;
                if f > 0.5 {
                    hash |= 1 << index
                } else {
                    hash &= !(1 << index)
                }
                index += 1
            }
        }
        Hash(hash)
    }
}

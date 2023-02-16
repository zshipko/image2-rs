use crate::*;

/// Hash is used for content-based hashing
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hash(blockhash::Blockhash256);

impl std::str::FromStr for Hash {
    type Err = <blockhash::Blockhash256 as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Hash(blockhash::Blockhash256::from_str(s)?))
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0.to_string())
    }
}

impl std::fmt::LowerHex for Hash {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0.to_string())?;
        Ok(())
    }
}

impl Hash {
    /// Compute hamming distance between two hashes
    pub fn diff(&self, other: &Hash) -> u32 {
        self.0.distance(&other.0)
    }
}

impl From<Hash> for String {
    fn from(hash: Hash) -> String {
        hash.0.to_string()
    }
}

impl<'a, T: Type, C: 'a + Color> Image<T, C>
where
    &'a Image<T, C>: blockhash::Image,
{
    /// Get image hash
    pub fn hash(&'a self) -> Hash {
        Hash(blockhash::blockhash256(&self))
    }
}

impl<'a, T: Type, C: Color> blockhash::Pixel for Data<'a, T, C> {
    const MAX_BRIGHTNESS: u32 = u16::MAX as u32;

    fn brightness(self) -> u32 {
        let sum: u32 = self
            .as_slice()
            .iter()
            .map(|x| T::convert::<u16>(x) as u32)
            .sum();
        sum / C::CHANNELS as u32
    }
}

impl<'a, T: Type, C: Color> blockhash::Image for &'a Image<T, C> {
    type Pixel = Data<'a, T, C>;

    fn dimensions(&self) -> (u32, u32) {
        (self.width() as u32, self.height() as u32)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.get((x as usize, y as usize))
    }
}

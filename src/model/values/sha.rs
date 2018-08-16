use std::str;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone)]
pub struct Sha(pub [u8; 40]);

impl Sha {
    pub fn to_str(&self) -> &str {
        str::from_utf8(&self.0).unwrap_or("INVALID")
    }
}

impl PartialEq<Sha> for Sha {
    fn eq(&self, other: &Sha) -> bool {
        &self.0[..] == &other.0[..]
    }
}

impl Eq for Sha {
}

impl Hash for Sha {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.0)
    }
}

impl fmt::Debug for Sha {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Sha {{ {} }}", self.to_str())
    }
}


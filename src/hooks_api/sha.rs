use std::fmt;
use serde::de;
use serde::de::*;
use model::values::sha::Sha;

impl<'de> Deserialize<'de> for Sha {
    fn deserialize<D>(deserializer: D) -> Result<Sha, D::Error>
    where D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(ShaVisitor)
    }
}

struct ShaVisitor;

impl<'de> Visitor<'de> for ShaVisitor {
    type Value = Sha;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a 40 character Git SHA hash")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where E: de::Error,
    {
        let bytes = value.as_bytes();
        if bytes.len() == 40 {
            let mut sha = [0; 40];
            for i in 0..40 {
                sha[i] = bytes[i];
            }
            Ok(Sha(sha))
        } else {
            Err(de::Error::invalid_value(Unexpected::Str(value), &self))
        }
    }
}


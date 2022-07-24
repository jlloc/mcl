use std::fmt;

pub trait Checksum {
    fn verify_checksum(&self, v: &bytes::Bytes) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct ChecksumVerificationError;

impl fmt::Display for ChecksumVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "checksum verification failed")
    }
}

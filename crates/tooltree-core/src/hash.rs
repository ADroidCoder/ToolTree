//! 常用哈希：一次读取同时算出所有算法的结果，文件只读一遍

use md5::Md5;
use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Serialize, PartialEq)]
pub struct Hashes {
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha384: String,
    pub sha512: String,
    /// 参与计算的字节数
    pub size: u64,
}

#[derive(Default)]
struct Hasher {
    md5: Md5,
    sha1: Sha1,
    sha256: Sha256,
    sha384: Sha384,
    sha512: Sha512,
    size: u64,
}

impl Hasher {
    fn update(&mut self, data: &[u8]) {
        self.md5.update(data);
        self.sha1.update(data);
        self.sha256.update(data);
        self.sha384.update(data);
        self.sha512.update(data);
        self.size += data.len() as u64;
    }

    fn finish(self) -> Hashes {
        Hashes {
            md5: hex(&self.md5.finalize()),
            sha1: hex(&self.sha1.finalize()),
            sha256: hex(&self.sha256.finalize()),
            sha384: hex(&self.sha384.finalize()),
            sha512: hex(&self.sha512.finalize()),
            size: self.size,
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hash_bytes(data: &[u8]) -> Hashes {
    let mut h = Hasher::default();
    h.update(data);
    h.finish()
}

pub fn hash_reader(mut r: impl Read) -> io::Result<Hashes> {
    let mut h = Hasher::default();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finish())
}

pub fn hash_file(path: &Path) -> io::Result<Hashes> {
    hash_reader(File::open(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        let h = hash_bytes(b"abc");
        assert_eq!(h.md5, "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(h.sha1, "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(h.sha256, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(h.size, 3);
    }

    #[test]
    fn reader_matches_bytes() {
        let data = vec![7u8; 3 << 20];
        assert_eq!(hash_reader(&data[..]).unwrap(), hash_bytes(&data));
    }
}

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

/// Metainfo files (also known as .torrent files)
#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct Torrent {
    /// The URL of the tracker.
    pub announce: String,

    pub info: Info,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Hashes(Vec<[u8; 20]>);
struct HashesVisitor;

impl<'de> Visitor<'de> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte string whose length is a multiple of 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() % 20 != 0 {
            return Err(E::custom(format!("length is {}", v.len())));
        }

        Ok(Hashes(
            v.chunks_exact(20)
                .map(|slice| slice.try_into().expect("guaranteed to be length 20"))
                .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashesVisitor)
    }
}

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct Info {
    /// The suggested name to save the file (or directory) as.
    pub name: String,

    /// The length maps to the number of bytes in each piece the file is split into. For the
    /// purposes of transfer, files are split into fixed-size pieces which are all the same length
    /// except for possibly the last one which may be truncated. piece length is almost always a
    /// power of two, most commonly 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M
    /// as default).
    #[serde(rename = "piece length")]
    pub piece_length: usize,

    /// Each entry of `pieces` is the SHA1 hash of the piece at the corresponding index.
    pub pieces: Hashes,

    #[serde(flatten)]
    pub keys: Keys,
}

/// There is a key `length` or a key `files`, but not both or neither.
#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Keys {
    /// If `length` is present then the download represents a single file.
    SingleFile {
        /// The length of the file in bytes.
        length: usize,
    },
    /// Otherwise it represents a set of files which go in a directory structure.
    ///
    /// For the purposes of the other keys in `Info`, the multi-file case is treated as only having
    /// a single file by concatenating the files in the order they appear in the files list.
    MultiFile { files: Vec<File> },
}

#[allow(unused)]
#[derive(Deserialize, Debug, Clone)]
pub struct File {
    /// The length of the file, in bytes.
    pub length: usize,
    /// A list of UTF-8 encoded strings corresponding to subdirectory names, the last of which is
    /// the actual file name (a zero length list is an error case).
    pub path: Vec<String>,
}

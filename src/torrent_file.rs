use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha1::{Sha1, Digest};
use serde::{Deserialize, Serialize};
use serde_bencode::de;

#[derive(Serialize, Deserialize, Debug)]
struct BencodeInfo {
    #[serde(rename = "pieces")]
    pieces: Vec<u8>,
    #[serde(rename = "piece length")]
    piece_length: u64,
    #[serde(rename = "length")]
    length: u64,
    #[serde(rename = "name")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct BencodeTorrent {
    #[serde(rename = "announce")]
    announce: String,
    #[serde(rename = "info")]
    info: BencodeInfo,
}

struct TorrentFile {
    announce: String,
    piece_hashes: Vec<String>,
    piece_length: u64,
    length: u64,
    name: String,
    info_hash: String,
}

impl TorrentFile {
    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let bencode_torrent: BencodeTorrent = de::from_bytes(&contents).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Bencode error: {}", e))
        })?;

        let info_hash = compute_info_hash(&bencode_torrent.info);
        let piece_hashes = parse_piece_hashes(&bencode_torrent.info.pieces);

        Ok(TorrentFile {
            announce: bencode_torrent.announce,
            piece_hashes,
            piece_length: bencode_torrent.info.piece_length,
            length: bencode_torrent.info.length,
            name: bencode_torrent.info.name,
            info_hash,
        })
    }
}

fn compute_info_hash(info: &BencodeInfo) -> String {
    let encoded_info = serde_bencode::to_bytes(info).expect("Failed to bencode info");
    let mut hasher = Sha1::new();
    hasher.update(encoded_info);
    hex::encode(hasher.finalize())
}

fn parse_piece_hashes(pieces: &[u8]) -> Vec<String> {
    pieces.chunks(20).map(hex::encode).collect()
}


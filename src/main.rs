use anyhow::Context;
use clap::{Parser, Subcommand};
use torrent::{Keys, Torrent};
use tracker::{TrackerRequest, TrackerResponse};

use std::path::PathBuf;

mod bencode;
mod torrent;
mod tracker;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Decode { value: String },
    Info { torrent: PathBuf },
    Peers { torrent: PathBuf },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Decode { value } => {
            let v = bencode::decode_bencoded_value(&value).0;
            println!("{v}");
        }
        Command::Info { torrent } => {
            let dot_torrent = std::fs::read(torrent).context("read torrent file")?;
            let t: Torrent =
                serde_bencode::from_bytes(&dot_torrent).context("parse torrent file")?;

            println!("Tracker URL: {}", t.announce);

            if let Keys::SingleFile { length } = t.info.keys {
                println!("Length: {length}")
            }

            println!("Info Hash: {}", hex::encode(t.info.hash()));

            println!("Piece Length: {}", t.info.piece_length);
            println!("Piece Hashes:");
            for hash in t.info.pieces.0 {
                println!("{}", hex::encode(&hash));
            }
        }
        Command::Peers { torrent } => {
            let dot_torrent = std::fs::read(torrent).context("read torrent file")?;
            let t: Torrent =
                serde_bencode::from_bytes(&dot_torrent).context("parse torrent file")?;

            let length = if let torrent::Keys::SingleFile { length } = t.info.keys {
                length
            } else {
                todo!();
            };

            let request = TrackerRequest {
                peer_id: "00112233445566778899".to_owned(),
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: length,
                compact: 1,
            };
            let url_params =
                serde_urlencoded::to_string(&request).context("url-encode tracker parameters")?;

            let response = reqwest::get(format!(
                "{}?{}&info_hash={}",
                t.announce,
                url_params,
                urlencode(&t.info.hash()),
            ))
            .await
            .context("query tracker")?;
            let response = response.bytes().await.context("fetch tracker response")?;
            let response: TrackerResponse =
                serde_bencode::from_bytes(&response).context("parse tracker response")?;

            for peer in &response.peers.0 {
                println!("{}:{}", peer.ip(), peer.port());
            }
        }
    }

    Ok(())
}

fn urlencode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());
    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode(&[byte]));
    }
    encoded
}

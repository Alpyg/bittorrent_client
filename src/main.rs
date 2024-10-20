use anyhow::Context;
use bittorrent_starter_rust::{
    bencode,
    peer::Handshake,
    torrent::{self, Keys, Torrent},
    tracker::{TrackerRequest, TrackerResponse},
};
use clap::{Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::{net::SocketAddrV4, path::PathBuf};

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
    Handshake { torrent: PathBuf, peer: String },
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
                peer_id: "uwuwuwuwuwuwuwuwuwuw".to_owned(),
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
        Command::Handshake { torrent, peer } => {
            let dot_torrent = std::fs::read(torrent).context("read torrent file")?;
            let t: Torrent =
                serde_bencode::from_bytes(&dot_torrent).context("parse torrent file")?;

            let info_hash = t.info.hash();
            let peer = peer.parse::<SocketAddrV4>().context("parse peer address")?;

            let mut peer = tokio::net::TcpStream::connect(peer)
                .await
                .context("connect to peer")?;
            let mut handshake = Handshake::new(info_hash, *b"uwuwuwuwuwuwuwuwuwuw");
            {
                let handshake_bytes =
                    &mut handshake as *mut Handshake as *mut [u8; std::mem::size_of::<Handshake>()];
                let handshake_bytes: &mut [u8; std::mem::size_of::<Handshake>()] =
                    unsafe { &mut *handshake_bytes };
                peer.write_all(handshake_bytes)
                    .await
                    .context("write handshake")?;
                peer.read_exact(handshake_bytes)
                    .await
                    .context("read handshake")?;
            }
            assert_eq!(handshake.length, 19);
            assert_eq!(&handshake.bittorrent, b"BitTorrent protocol");
            println!("Peer ID: {}", hex::encode(&handshake.peer_id));
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

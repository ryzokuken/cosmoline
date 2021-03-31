use async_std::net::IpAddr;
use ed25519_dalek::PublicKey;

use crate::keypair::SSBPublicKey;

enum Protocol {
    Net,
    Ws,
    Wss,
}

pub struct Node {
    protocol: Protocol,
    host: IpAddr,
    port: u16,
    pubkey: PublicKey,
}

impl Node {
    pub fn to_base64(&self) -> String {
        let proto = match self.protocol {
            Protocol::Net => "net",
            Protocol::Ws => "ws",
            Protocol::Wss => "wss",
        };
        format!(
            "{}:{}:{}~shs:{}",
            proto,
            self.host,
            self.port,
            self.pubkey.to_base64()
        )
    }

    pub fn from_base64(packet: &str) -> Self {
        let mut packet = packet.splitn(4, ':');
        let protocol = match packet.next().unwrap() {
            "net" => Protocol::Net,
            "ws" => Protocol::Ws,
            "wss" => Protocol::Wss,
            _ => panic!("unknown protocol"),
        };
        let host = IpAddr::V4(packet.next().unwrap().parse().unwrap());
        let port = packet
            .next()
            .unwrap()
            .splitn(2, '~')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let pubkey = SSBPublicKey::from_base64(packet.next().unwrap());
        Node {
            protocol,
            host,
            port,
            pubkey,
        }
    }
}

use async_std::net::IpAddr;
use ed25519_dalek::PublicKey;

use crate::keypair::SSBPublicKey;

pub enum Protocol {
    Net,
    Ws,
    Wss,
}

pub enum Handshake {
    Shs,
    Shs2,
}

pub struct Address {
    protocol: Protocol,
    host: IpAddr,
    port: u16,
    handshake: Handshake,
}

impl Address {
    pub fn new(protocol: Protocol, host: IpAddr, port: u16, handshake: Handshake) -> Self {
        Self {
            protocol,
            host,
            port,
            handshake,
        }
    }
}

pub struct Peer {
    addresses: Vec<Address>,
    key: PublicKey,
}

impl Peer {
    pub fn new(addresses: Vec<Address>, key: PublicKey) -> Self {
        Self { addresses, key }
    }

    // TODO: do this properly
    pub fn to_discovery_packet(&self) -> String {
        self.addresses
            .iter()
            .map(|address| {
                let proto = match address.protocol {
                    Protocol::Net => "net",
                    Protocol::Ws => "ws",
                    Protocol::Wss => "wss",
                };
                let hs = match address.handshake {
                    Handshake::Shs => "shs",
                    Handshake::Shs2 => "shs2",
                };
                format!(
                    "{}:{}:{}~{}:{}",
                    proto,
                    address.host,
                    address.port,
                    hs,
                    self.key.to_base64(),
                )
            })
            .collect::<Vec<String>>()
            .join(";")
    }

    // TODO: do this properly
    pub fn from_discovery_packet(packet: &str) -> Self {
        let mut key = Option::None;
        let addresses = packet
            .split(';')
            .map(|address| {
                let mut address = address.splitn(2, '~');

                let mut network = address.next().unwrap().splitn(3, ':');
                let protocol = match network.next().unwrap() {
                    "net" => Protocol::Net,
                    "ws" => Protocol::Ws,
                    "wss" => Protocol::Wss,
                    _ => panic!("unknown protocol"),
                };
                let host = IpAddr::V4(network.next().unwrap().parse().unwrap());
                let port = network.next().unwrap().parse().unwrap();

                let mut info = address.next().unwrap().splitn(2, ':');
                let handshake = match info.next().unwrap() {
                    "shs" => Handshake::Shs,
                    "shs2" => Handshake::Shs2,
                    _ => panic!("unknown handshake"),
                };
                let pubkey = SSBPublicKey::from_base64(info.next().unwrap());
                if key == Option::None {
                    key = Some(pubkey);
                } else if key.unwrap() != pubkey {
                    panic!("unexpected pubkey");
                }

                Address {
                    protocol,
                    host,
                    port,
                    handshake,
                }
            })
            .collect();
        Peer {
            addresses,
            key: key.unwrap(),
        }
    }
}

use async_std::net::{IpAddr, UdpSocket};
use async_std::sync::Arc;
use async_std::task;
use ed25519_dalek::PublicKey;

use crate::keypair::SSBPublicKey;

enum Protocol {
    Net,
    Ws,
    Wss,
}

enum Handshake {
    Shs,
    Shs2,
}

struct Address {
    protocol: Protocol,
    host: IpAddr,
    port: u16,
    handshake: Handshake,
}

pub struct Peer {
    addresses: Vec<Address>,
    key: PublicKey,
}

impl Peer {
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

pub async fn peer_discovery_recv() {
    let socket = UdpSocket::bind(":::8008").await.unwrap();
    let mut buf = [0u8; 1024];

    loop {
        let (amt, peer) = socket.recv_from(&mut buf).await.unwrap();
        let buf = &mut buf[..amt];
        let packet = String::from_utf8(buf.to_vec()).unwrap();
        println!(
            "{} {}",
            peer,
            Peer::from_discovery_packet(&packet).to_discovery_packet()
        );
    }
}

pub async fn peer_discovery_send(pubkey: Arc<String>) {
    let socket = UdpSocket::bind(":::0").await.unwrap();
    let msg = format!("net:1.2.3.4:8023~shs:{}", &pubkey);
    let buf = msg.as_bytes();

    socket.set_broadcast(true).unwrap();

    loop {
        println!("Sending packet: {:?}", &msg);
        socket.send_to(&buf, "255.255.255.255:8008").await.unwrap();
        socket.send_to(&buf, "[FF02::1]:8008").await.unwrap();
        task::sleep(std::time::Duration::from_secs(1)).await;
    }
}

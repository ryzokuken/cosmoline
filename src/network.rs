use async_std::task;
use async_std::net::{IpAddr, UdpSocket};
use async_std::sync::Arc;
use ed25519_dalek::PublicKey;

use crate::keypair::SSBPublicKey;

enum Protocol {
    Net,
    Ws,
    Wss,
}

pub struct Peer {
    protocol: Protocol,
    host: IpAddr,
    port: u16,
    pubkey: PublicKey,
}

impl Peer {
    pub fn to_discovery_packet(&self) -> String {
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

    pub fn from_discovery_packet(packet: &str) -> Self {
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
        Peer {
            protocol,
            host,
            port,
            pubkey,
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

use async_std::net::UdpSocket;
use async_std::sync::Arc;
use async_std::task;
use ed25519_dalek::PublicKey;

use std::collections::HashSet;

use crate::keypair::SSBPublicKey;
use crate::peer::{Address, Handshake, Peer, Protocol};

pub async fn recv(sender: async_std::channel::Sender<Peer>) {
    let socket = UdpSocket::bind(":::8008").await.unwrap();
    let mut buf = [0u8; 1024];
    let mut old = HashSet::new();

    loop {
        let (amt, _) = socket.recv_from(&mut buf).await.unwrap();
        let buf = &mut buf[..amt];
        let packet = String::from_utf8(buf.to_vec()).unwrap();
        let peer = Peer::from_discovery_packet(packet.as_str());
        if !old.contains(&peer) {
            old.insert(peer.clone());
            sender.send(peer).await.unwrap();
        }
    }
}

pub async fn send(pubkey: Arc<String>) {
    let socket = UdpSocket::bind(":::0").await.unwrap();
    let msg = Peer::new(
        Vec::from([Address::new(
            Protocol::Net,
            "1.2.3.4".parse().unwrap(),
            8023,
            Handshake::Shs,
        )]),
        PublicKey::from_base64(pubkey.as_str()),
    )
    .to_discovery_packet();
    let buf = msg.as_bytes();

    socket.set_broadcast(true).unwrap();

    loop {
        socket.send_to(&buf, "255.255.255.255:8008").await.unwrap();
        socket.send_to(&buf, "[FF02::1]:8008").await.unwrap();
        task::sleep(std::time::Duration::from_secs(1)).await;
    }
}

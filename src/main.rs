use async_std::{fs, task};
use async_std::net::UdpSocket;
use async_std::path::PathBuf;
use async_std::sync::Arc;
use clap::{load_yaml, App};
use ed25519_dalek::Keypair;

mod keypair;
use keypair::{SSBKeypair, SSBPublicKey};

mod network;
use network::Peer;

type Config = toml::map::Map<String, toml::Value>;

async fn peer_discovery_recv() {
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

async fn peer_discovery_send(pubkey: Arc<String>) {
    let socket = UdpSocket::bind(":::0").await.unwrap();
    let msg = format!("net:1.2.3.4:8023~shs:{}", &pubkey);
    let buf = msg.as_bytes();

    socket.set_broadcast(true).unwrap();

    loop {
        println!("Sending packet: {:?}", &msg);
        socket.send_to(&buf, "255.255.255.255:8008").await.unwrap();
        task::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[async_std::main]
async fn main() {
    let options = load_yaml!("options.yaml");
    let options = App::from(options).get_matches();

    let config_file = match options.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(
            dirs::config_dir()
                .unwrap()
                .join("cosmoline")
                .join("config.toml"),
        ),
    };
    let config = fs::read_to_string(config_file).await.unwrap();
    let config: Config = toml::from_str(config.as_str()).unwrap();

    let path = match options.value_of("path") {
        Some(path) => PathBuf::from(path),
        None => match config.get("path") {
            Some(path) => PathBuf::from(path.as_str().unwrap()),
            None => PathBuf::from(dirs::home_dir().unwrap().join(".cosmoline")),
        },
    };
    let keypair = Keypair::read_or_generate(path.join("secret")).await;
    println!("{}", keypair.to_json().pretty(2));

    let pubkey = keypair.public.to_base64();

    task::spawn(peer_discovery_recv());
    task::spawn(peer_discovery_send(Arc::new(pubkey))).await;
}

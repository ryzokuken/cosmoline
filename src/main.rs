use async_std::fs;
use async_std::net::{IpAddr, UdpSocket};
use async_std::path::PathBuf;
use clap::{load_yaml, App};
use ed25519_dalek::{Keypair, PublicKey};

mod keypair;
use keypair::{SSBKeypair, SSBPublicKey};

type Config = toml::map::Map<String, toml::Value>;

enum Protocol {
    Net,
    Ws,
    Wss,
}

struct Node {
    protocol: Protocol,
    host: IpAddr,
    port: u16,
    pubkey: PublicKey,
}

impl Node {
    fn to_base64(&self) -> String {
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

    fn from_base64(packet: &str) -> Self {
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

    let socket = UdpSocket::bind("0.0.0.0:8008").await.unwrap();
    let mut buf = [0u8; 1024];

    loop {
        let (amt, peer) = socket.recv_from(&mut buf).await.unwrap();
        let buf = &mut buf[..amt];
        let packet = String::from_utf8(buf.to_vec()).unwrap();
        println!("{} {}", peer, Node::from_base64(&packet).to_base64());
    }
}

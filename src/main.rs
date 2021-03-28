use clap::{load_yaml, App};
use ed25519_dalek::Keypair;

use std::net::UdpSocket;
use std::path::PathBuf;

mod keypair;

use keypair::SSBKeypair;

type Config = toml::map::Map<String, toml::Value>;

#[derive(Debug)]
struct Host {
    protocol: String,
    host: String,
    port: String,
    pubkey: String,
}

fn parse_packet(packet: String) -> Host {
    let mut packet = packet.splitn(4, ':');
    let protocol = packet.next().unwrap();
    let host = packet.next().unwrap();
    let port = packet.next().unwrap().splitn(2, '~').next().unwrap();
    let pubkey = packet.next().unwrap();
    Host {
        protocol: protocol.to_string(),
        host: host.to_string(),
        port: port.to_string(),
        pubkey: pubkey.to_string(),
    }
}

fn main() {
    let options = load_yaml!("options.yaml");
    let options = App::from(options).get_matches();

    let config_file = match options.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => dirs::config_dir()
            .unwrap()
            .join("cosmoline")
            .join("config.toml"),
    };
    let config = std::fs::read_to_string(config_file).unwrap();
    let config: Config = toml::from_str(config.as_str()).unwrap();

    let path = match options.value_of("path") {
        Some(path) => PathBuf::from(path),
        None => match config.get("path") {
            Some(path) => PathBuf::from(path.as_str().unwrap()),
            None => dirs::home_dir().unwrap().join(".cosmoline"),
        },
    };
    let keypair = Keypair::read_or_generate(path.join("secret"));
    println!("{}", keypair.to_json().pretty(2));

    let socket = UdpSocket::bind("0.0.0.0:8008").unwrap();
    let mut buf = [0; 1024];
    let (amt, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &mut buf[..amt];
    let packet = String::from_utf8(buf.to_vec()).unwrap();
    println!("{:?}", parse_packet(packet));
}

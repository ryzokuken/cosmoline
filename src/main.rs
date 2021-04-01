use async_std::path::PathBuf;
use async_std::sync::Arc;
use async_std::{fs, task};
use clap::{load_yaml, App};
use ed25519_dalek::Keypair;

mod keypair;
use keypair::{SSBKeypair, SSBPublicKey};

mod network;

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
    let config: toml::map::Map<String, toml::Value> = toml::from_str(config.as_str()).unwrap();

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

    task::spawn(network::peer_discovery_recv());
    task::spawn(network::peer_discovery_send(Arc::new(pubkey))).await;
}

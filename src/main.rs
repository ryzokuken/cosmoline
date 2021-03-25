use clap::{load_yaml, App};
use ed25519_dalek::Keypair;

use std::fs::File;
use std::path::PathBuf;

use std::io::prelude::*;

mod keypair;

use keypair::SSBKeypair;

type Config = toml::map::Map<String, toml::Value>;

fn read_config(path: PathBuf) -> Config {
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    toml::from_str(content.as_str()).unwrap()
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
    let config = read_config(config_file);

    let path = match options.value_of("path") {
        Some(path) => PathBuf::from(path),
        None => match config.get("path") {
            Some(path) => PathBuf::from(path.as_str().unwrap()),
            None => dirs::home_dir().unwrap().join(".cosmoline"),
        },
    };
    let keypair = Keypair::read_or_generate(path.join("secret"));
    println!("{}", keypair.to_json().pretty(2));
}

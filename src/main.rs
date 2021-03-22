use clap::{load_yaml, App};
use ed25519_dalek::{SecretKey, PublicKey, Keypair};
use rand::rngs::OsRng;
use regex::Regex;

use std::fs::File;
use std::path::PathBuf;

use std::io::prelude::*;


fn main() {
    let options = load_yaml!("options.yaml");
    let matches = App::from(options).get_matches();

    let config_file = match matches.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => dirs::config_dir()
            .unwrap()
            .join("cosmoline")
            .join("config.toml"),
    };
    let mut config_file = File::open(config_file).unwrap();
    let mut config = String::new();
    config_file.read_to_string(&mut config).unwrap();
    let config: toml::Value = toml::from_str(config.as_str()).unwrap();

    let path = match config.as_table().unwrap().get("path") {
        Some(path) => PathBuf::from(path.as_str().unwrap()),
        None => dirs::home_dir().unwrap().join(".cosmoline"),
    };
    let secret_path = path.join("secret");
    let keypair = if secret_path.exists() {
        let mut secret_file = File::open(secret_path).unwrap();
        let mut secret = String::new();
        secret_file.read_to_string(&mut secret).unwrap();
        let re = Regex::new(r"\s*#[^\n]*").unwrap();
        let secret = re.replace_all(secret.as_str(), "");
        let secret = match json::parse(&secret).unwrap() {
            json::JsonValue::Object(obj) => obj,
            _ => panic!("invalid secret file"),
        };

        if secret.get("curve").unwrap().as_str().unwrap() != "ed25519" {
            panic!("wrong curve");
        }

        let pubkey = secret
            .get("public")
            .unwrap()
            .as_str()
            .unwrap()
            .replace(".ed25519", "");
        let pubkey = base64::decode(pubkey).unwrap();
        let pubkey = PublicKey::from_bytes(pubkey.as_slice()).unwrap();

        let privkey = secret
            .get("private")
            .unwrap()
            .as_str()
            .unwrap()
            .replace(".ed25519", "");
        let privkey = base64::decode(privkey).unwrap();
        let privkey = SecretKey::from_bytes(&privkey[00..32]).unwrap();

        Keypair { public: pubkey, secret: privkey }
    } else {
        let mut csprng = OsRng {};
        Keypair::generate(&mut csprng)
        // TODO: write this keypair to a fresh secret file
    };
    println!("{:?}", keypair.public.to_bytes());
    println!("{:?}", keypair.secret.to_bytes());
}

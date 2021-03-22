use clap::{load_yaml, App};
use ed25519_dalek::{SecretKey, PublicKey, Keypair};
use rand::rngs::OsRng;
use regex::Regex;
use json::{object, JsonValue};

use std::fs::File;
use std::path::PathBuf;

use std::io::prelude::*;

trait SSBKeypair {
    fn to_json(&self) -> JsonValue;
    fn from_json(obj: JsonValue) -> Self;
    fn read_or_generate(path: PathBuf) -> Self;
}

impl SSBKeypair for Keypair {
    fn to_json(&self) -> JsonValue {
        let pubstring = base64::encode(self.public.to_bytes());
        let privstring = base64::encode([self.secret.to_bytes(), self.public.to_bytes()].concat());
        object! {
            curve: "ed25519",
            public: format!("{}.ed25519", pubstring),
            private: format!("{}.ed25519", privstring),
            id: format!("@{}.ed25519", pubstring)
        }
    }

    fn from_json(obj: JsonValue) -> Self {
        if obj["curve"].as_str().unwrap() != "ed25519" {
            panic!("wrong curve");
        }

        let pubkey = obj["public"]
            .as_str()
            .unwrap()
            .replace(".ed25519", "");
        let pubkey = base64::decode(pubkey).unwrap();
        let pubkey = PublicKey::from_bytes(pubkey.as_slice()).unwrap();

        let privkey = obj["private"]
            .as_str()
            .unwrap()
            .replace(".ed25519", "");
        let privkey = base64::decode(privkey).unwrap();
        let privkey = SecretKey::from_bytes(&privkey[00..32]).unwrap();

        Keypair { public: pubkey, secret: privkey }
    }

    fn read_or_generate(path: PathBuf) -> Self {
        if path.exists() {
            let mut secret_file = File::open(path).unwrap();
            let mut secret = String::new();
            secret_file.read_to_string(&mut secret).unwrap();
            let re = Regex::new(r"\s*#[^\n]*").unwrap();
            let secret = re.replace_all(secret.as_str(), "");
            SSBKeypair::from_json(json::parse(&secret).unwrap())
        } else {
            let mut csprng = OsRng {};
            Keypair::generate(&mut csprng)
            // TODO: write this keypair to a fresh secret file
        }
    }
}

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
    let keypair = Keypair::read_or_generate(path.join("secret"));
    println!("{}", keypair.to_json().pretty(2));
}

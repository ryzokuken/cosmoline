
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use json::{object, JsonValue};
use rand::rngs::OsRng;
use regex::Regex;

use async_std::fs;
use async_std::path::PathBuf;
use async_trait::async_trait;

#[async_trait]
pub trait SSBKeypair {
    fn to_json(&self) -> JsonValue;
    fn from_json(obj: JsonValue) -> Self;
    async fn read_or_generate(path: PathBuf) -> Self;
}

#[async_trait]
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
            .strip_suffix(".ed25519")
            .unwrap();
        let pubkey = base64::decode(pubkey).unwrap();
        let pubkey = PublicKey::from_bytes(pubkey.as_slice()).unwrap();

        let privkey = obj["private"]
            .as_str()
            .unwrap()
            .strip_suffix(".ed25519")
            .unwrap();
        let privkey = base64::decode(privkey).unwrap();
        let privkey = SecretKey::from_bytes(&privkey[00..32]).unwrap();

        Keypair {
            public: pubkey,
            secret: privkey,
        }
    }

    async fn read_or_generate(path: PathBuf) -> Self {
        if path.exists().await {
            let secret = fs::read_to_string(path).await.unwrap();
            let re = Regex::new(r"\s*#[^\n]*").unwrap();
            let secret = re.replace_all(secret.as_str(), "");
            SSBKeypair::from_json(json::parse(&secret).unwrap())
        } else {
            let mut csprng = OsRng {};
            let keypair = Keypair::generate(&mut csprng);
            let keypair_json = keypair.to_json();
            fs::write(
                path,
                format!(
                    include_str!("warning.txt"),
                    keys = keypair_json.pretty(2),
                    id = keypair_json["id"]
                ),
            ).await
            .unwrap();
            keypair
        }
    }
}

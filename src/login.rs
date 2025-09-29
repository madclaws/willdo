use std::str::FromStr;

use veilid_core::{Crypto, CryptoKind, Encodable, KeyPair};

pub struct Credential {
    keypair: KeyPair,
}

impl Credential {
    pub fn new() -> Self {
        let keypair = Crypto::generate_keypair(CryptoKind::from_str("VLD0").unwrap())
            .expect("Failed to create keypair")
            .value;
        Credential { keypair }
    }

    pub fn from_hash(key_hash: &str) -> Result<Self, String> {
        if let Ok(keypair) = KeyPair::try_decode(key_hash) {
            Ok(Credential { keypair })
        } else {
            Err(String::from("Failed to decode keypair from given hash"))
        }
    }

    pub fn to_hash(&self) -> String {
        self.keypair.encode()
    }
}

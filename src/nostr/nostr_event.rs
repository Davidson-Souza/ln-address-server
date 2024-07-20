use hex_conservative::DisplayHex;
use secp256k1::schnorr::Signature;
use secp256k1::Keypair;
use secp256k1::Message;
use secp256k1::Secp256k1;
use secp256k1::SecretKey;
use serde::Serialize;
use serde_json::json;
use sha2::Digest;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct UnsignedEvent {
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
}

impl UnsignedEvent {
    pub fn id(&self) -> [u8; 32] {
        let UnsignedEvent {
            pubkey,
            created_at,
            kind,
            tags,
            content,
            ..
        } = self;
        let json = json!([0, pubkey, created_at, kind, tags, content]);
        let event_str: String = json.to_string();
        sha2::Sha256::digest(event_str).into()
    }

    pub fn sign(&self, key: &SecretKey) -> Signature {
        let secp = Secp256k1::new();
        let msg = Message::from_digest(self.id());
        let keypair = Keypair::from_secret_key(&secp, key);
        secp.sign_schnorr_no_aux_rand(&msg, &keypair)
    }

    pub fn into_signed(self, key: &SecretKey) -> Event {
        let id = self.id().to_lower_hex_string();
        let sig = self.sign(key);
        let UnsignedEvent {
            pubkey,
            created_at,
            kind,
            tags,
            content,
        } = self;

        Event {
            pubkey,
            created_at,
            kind,
            tags,
            content,
            id,
            sig,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub id: String,
    pub sig: Signature,
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use hex_conservative::DisplayHex;
    use secp256k1::Message;
    use secp256k1::Secp256k1;
    use secp256k1::SecretKey;

    use super::UnsignedEvent;

    #[test]
    pub fn test_sign() {
        let sec_key =
            SecretKey::from_str("d7bee682d987439fae91bdc5fed8bbf16d84ec077a2bd5cf7592e384668198f3")
                .unwrap();
        let secp = Secp256k1::new();

        let event = UnsignedEvent {
            pubkey: sec_key.x_only_public_key(&secp).0.to_string(),
            kind: 1,
            created_at: 0,
            tags: Vec::new(),
            content: "".into(),
        }
        .into_signed(&sec_key);

        let msg = Message::from_digest(
            hex_conservative::FromHex::from_hex(&event.id).expect("should be a valid hex"),
        );

        assert!(secp
            .verify_schnorr(&event.sig, &msg, &sec_key.x_only_public_key(&secp).0)
            .is_ok());

        assert_eq!(
            event.id,
            "2ed573ea3d5e2a1d9ca7c0b54f9660f6a04bc82fb1315be19604d5867694b602"
        );
        assert_eq!(event.sig.serialize().to_lower_hex_string(), "bb8921e8ab85da09a2839f56fd4d62822d227a75bbcb80c417a971d8ed00467cef1d24b033a0be99664c36f6d066527436f89668593b3804a48a5d98ee78e539");
    }
}

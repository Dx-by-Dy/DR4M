use bincode::{Decode, Encode, config};
use std::fmt::Display;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Encode, Decode)]
pub struct UserSID {
    pub sid: u64,
}

impl Display for UserSID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sid)
    }
}

#[derive(Clone, Encode, Decode)]
pub struct AuthMessage {
    pub from: UserSID,
}

#[derive(Clone, Encode, Decode)]
pub struct UserMessage {
    pub from: UserSID,
    pub to: UserSID,
    pub message: String,
}

#[derive(Clone, Encode, Decode)]
pub enum Data {
    Auth(AuthMessage),
    UserMessage(UserMessage),
}

impl Into<Vec<u8>> for Data {
    fn into(self) -> Vec<u8> {
        // TODO: fix unwrap()
        bincode::encode_to_vec(self, config::standard()).unwrap()
    }
}

impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        // TODO: fix unwrap()
        bincode::decode_from_slice(&value, config::standard())
            .unwrap()
            .0
    }
}

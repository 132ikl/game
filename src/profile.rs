use bincode::{deserialize, serialize};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sled::IVec;

#[derive(Serialize, Clone, Debug)]
pub struct Profile {
    pub id: String,
    pub data: UserData,
}

impl Profile {
    pub fn new(id: String, data: UserData) -> Profile {
        Profile { id, data }
    }

    pub fn update(&mut self) -> Option<String> {
        let next = self.data.next;
        if Utc::now() > next {
            self.data.next = Utc::now() + Duration::days(1);
            self.data.ready = true;
            return None;
        } else {
            self.data.ready = false;
            let secs = (next - Utc::now()).num_seconds();
            let h = secs / 3600;
            let rem = secs % 3600;
            let m = rem / 60;
            return Some(format!("come back in {}h {}m to get again", h, m));
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    pub username: String,
    pub hash: String,
    pub points: u16,
    pub next: DateTime<Utc>,
    pub ready: bool,
    pub items: Vec<ShopItem>,
}

impl UserData {
    pub fn new(username: String, hash: String) -> UserData {
        UserData {
            username,
            hash,
            points: 0,
            next: Utc::now(),
            ready: true,
            items: Vec::new(),
        }
    }
}

impl From<IVec> for UserData {
    fn from(bytes: IVec) -> Self {
        let vec: Vec<u8> = bytes.to_vec();
        let data: UserData = deserialize(&vec).expect("Failed to deserialize user data");
        data
    }
}

impl From<UserData> for IVec {
    fn from(data: UserData) -> Self {
        let vec: Vec<u8> = serialize(&data).expect("Failed to serialize user data");
        IVec::from(vec)
    }
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u8)]
pub enum ShopItem {
    One = 1,
    Two = 2,
    Three = 3,
}

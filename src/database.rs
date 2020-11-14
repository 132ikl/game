use bincode::{deserialize, serialize};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sled::open;
use sled::{Db, IVec};

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
        let next = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.data.next, 0), Utc);
        let now: DateTime<Utc> = Utc::now();
        if now > next {
            let next = (now + Duration::days(1)).timestamp();
            self.data.next = next;
            self.data.ready = true;
            return None;
        } else {
            self.data.ready = false;
            let secs = (next - now).num_seconds();
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
    pub next: i64,
    pub ready: bool,
    pub items: Vec<ShopItem>,
}

impl UserData {
    pub fn new(username: String, hash: String) -> UserData {
        UserData {
            username,
            hash,
            points: 0,
            next: 0,
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

pub struct Database {
    db: Db,
}

impl Database {
    pub fn open() -> Database {
        let path = "database";
        let db = open(path).expect("Unable to access database");
        Database { db }
    }

    pub fn save_profile(&self, profile: Profile) {
        self.db
            .insert(profile.id, profile.data)
            .expect("Failed to insert");
    }

    pub fn load_profile(&self, id: &str) -> Option<Profile> {
        let vec: IVec = self.db.get(&id).unwrap()?;
        let data: UserData = vec.into();
        Some(Profile::new(String::from(id), data))
    }

    pub fn get_profiles(&self) -> impl Iterator<Item = Profile> {
        self.db
            .iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| {
                string_from_bytes(&item.0)
                    .ok()
                    .map(|id| Profile::new(id, item.1.into()))
            })
    }

    pub fn from_username(&self, username: &str) -> Option<Profile> {
        self.get_profiles()
            .find(|profile| profile.data.username == username)
    }

    pub fn gen_id(&self) -> String {
        self.db.generate_id().unwrap().to_string()
    }
}

fn string_from_bytes(bytes: &IVec) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(bytes.to_vec())
}

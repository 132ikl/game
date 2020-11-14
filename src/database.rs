use crate::profile::{Profile, UserData};

use sled::open;
use sled::{Db, IVec};


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

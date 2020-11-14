pub mod database {
    use bincode::{deserialize, serialize};
    use serde::{Deserialize, Serialize};
    use sled::open;
    use sled::{Db, IVec};

    #[derive(Serialize, Debug)]
    pub struct Profile {
        pub id: String,
        pub data: UserData,
    }

    impl Profile {
        pub fn new(id: String, data: UserData) -> Profile {
            Profile { id, data }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserData {
        pub username: String,
        pub hash: String,
        pub points: u16,
    }

    impl UserData {
        pub fn new(username: String, hash: String) -> UserData {
            let points = 0;
            UserData {
                username,
                hash,
                points,
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

        pub fn load_profile(&self, id: String) -> Option<Profile> {
            let vec: IVec = self.db.get(&id).unwrap()?;
            let data: UserData = vec.into();
            Some(Profile { id, data })
        }

        pub fn get_profiles(&self) -> Vec<Profile> {
            let mut profiles: Vec<Profile> = Vec::new();
            for item in self.db.iter() {
                // TODO: make less ugly
                profiles.push(match item.ok() {
                    Some(x) => {
                        let id: String = match std::str::from_utf8(&x.0.to_vec()).ok() {
                            Some(x) => x.to_owned(),
                            None => continue,
                        };
                        let data: UserData = x.1.into();
                        Profile { id, data }
                    }
                    None => continue,
                });
            }
            profiles
        }

        pub fn get_id(&self, username: String) -> Option<String> {
            for profile in self.get_profiles() {
                if profile.data.username == username {
                    return Some(profile.id);
                }
            }
            None
        }

        pub fn gen_id(&self) -> u64 {
            self.db.generate_id().unwrap()
        }
    }
}

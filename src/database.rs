pub mod database {
    use bincode::{serialize, deserialize};
    use serde::{Serialize, Deserialize};
    use sled::{Db, IVec};
    use sled::open;

    #[derive(Debug)]
    pub struct Profile {
        username: String,
        data: UserData
    }

    impl Profile {
        pub fn new(username: String, data: UserData) -> Profile {
            Profile { username, data }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserData {
        hash: String,
        points: u16
    }

    impl UserData {
        pub fn new(hash: String, points: u16) -> UserData {
            UserData { hash, points }
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
        db: Db
    }

    impl Database {
        pub fn new() -> Database {
            let path = "database";
            let db = open(path).expect("Unable to access database");
            Database { db }
        }

        pub fn save_profile(&self, profile: Profile) {
            self.db.insert(profile.username, profile.data).expect("Failed to insert");
        }

        pub fn load_profile(&self, username: String) -> Profile {
            let vec: IVec = self.db.get(&username).unwrap().unwrap();
            let data: UserData = vec.into();
            Profile { username, data }
        }
    }
}

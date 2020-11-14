use std::fmt;

use bincode::{deserialize, serialize};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::de;
use serde::de::Visitor;
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
        let next = self.data.next.time;
        if Utc::now() > next {
            self.data.next.time = Utc::now() + Duration::days(1);
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
    pub next: TimeWrap,
    pub ready: bool,
    pub items: Vec<ShopItem>,
}

impl UserData {
    pub fn new(username: String, hash: String) -> UserData {
        UserData {
            username,
            hash,
            points: 0,
            next: TimeWrap::new(),
            ready: true,
            items: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TimeWrap {
    pub time: DateTime<Utc>
}

impl TimeWrap {
    pub fn new() -> TimeWrap {
        TimeWrap {
            time: Utc::now()
        }
    }
}

impl Serialize for TimeWrap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let time: i64 = self.time.timestamp();
        serializer.serialize_i64(time)
    }
}

struct TimeWrapVisitor;

impl<'de> Visitor<'de> for TimeWrapVisitor {
    type Value = TimeWrap;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a unix timestamp integer")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(value, 0), Utc);
        Ok(TimeWrap { time })
    }

}

impl<'de> Deserialize<'de> for TimeWrap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_i64(TimeWrapVisitor)
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

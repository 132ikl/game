use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use bincode::{deserialize, serialize};
use chrono::{DateTime, Duration, Utc};
use rand::{seq::SliceRandom, Rng};
use rocket::FromFormValue;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sled::IVec;

use crate::database::Database;

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
            let duration: i64 = if self.has_item(ShopItem::DoubleSpeed) {
                12
            } else {
                24
            };
            self.data.next = Utc::now() + Duration::hours(duration);
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

    pub fn owned_items(&self) -> HashMap<String, bool> {
        ShopItem::get_prices()
            .iter()
            .map(|(k, _)| (k.to_string(), self.data.items.contains(k)))
            .collect()
    }

    pub fn has_item(&self, item: ShopItem) -> bool {
        self.data.items.contains(&item)
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

impl From<&UserData> for IVec {
    fn from(data: &UserData) -> Self {
        let vec: Vec<u8> = serialize(data).expect("Failed to serialize user data");
        IVec::from(vec)
    }
}

#[derive(
    Serialize_repr, Deserialize_repr, Clone, Debug, Hash, PartialEq, Eq, Copy, FromFormValue,
)]
#[repr(u8)]
pub enum ShopItem {
    DarkMode = 1,
    GayButton = 2,
    DoubleSpeed = 3,
    FiftyFifty = 4,
    Thanos = 5,
}

impl ShopItem {
    pub fn get_prices() -> HashMap<ShopItem, u16> {
        let mut prices: HashMap<ShopItem, u16> = HashMap::new();
        prices.insert(ShopItem::FiftyFifty, 1);
        prices.insert(ShopItem::DarkMode, 3);
        prices.insert(ShopItem::GayButton, 10);
        prices.insert(ShopItem::DoubleSpeed, 20);
        prices.insert(ShopItem::Thanos, 50);
        prices
    }

    pub fn get_display_prices(profile: Profile) -> Vec<(String, u16, bool)> {
        // real name, display name, price, been purchased
        let mut prices = ShopItem::get_prices()
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    k.price_with_sell(*v, &profile),
                    profile.data.items.contains(k),
                )
            })
            .collect::<Vec<(String, u16, bool)>>();
        prices.sort_by(|a, b| a.0.cmp(&b.0)); // sort alphabetically, then by price
        prices.sort_by(|a, b| a.1.cmp(&b.1)); // ensures constant order even for same price items
        prices
    }

    pub fn get_price(&self, profile: &Profile) -> Option<u16> {
        Some(self.price_with_sell(*ShopItem::get_prices().get(self)?, profile))
    }

    fn price_with_sell(&self, price: u16, profile: &Profile) -> u16 {
        if profile.data.items.contains(self) {
            ((price as f32) * 0.8) as u16
        } else {
            price
        }
    }

    pub fn buy_hook(&self, profile: &mut Profile) {
        let mut rng = rand::thread_rng();
        match self {
            ShopItem::FiftyFifty => {
                let prize: u16 = rng.gen_range(0, 2) * 2; // add 0 or 2
                profile.data.points += prize;
                profile.data.items.retain(|x| x != &ShopItem::FiftyFifty); // don't allow sell
                Database::open().save_profile(profile);
            }
            ShopItem::Thanos => {
                let db: Database = Database::open();
                let mut profiles: Vec<Profile> = db.get_profiles().collect::<Vec<Profile>>();
                profiles.shuffle(&mut rng);
                let half: usize = ((profiles.len() as f32) / 2.0) as usize;
                profiles.iter().take(half).for_each(|p| {
                    let mut p = p.clone();
                    p.data.points = 0;
                    db.save_profile(&p);
                });
                profiles.len();

                profile.data.items.retain(|x| x != &ShopItem::Thanos); // don't allow sell
            }
            _ => Database::open().save_profile(profile),
        }
    }
}

impl fmt::Display for ShopItem {
    // for to_string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

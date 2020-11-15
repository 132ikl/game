use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use bincode::{deserialize, serialize};
use chrono::{DateTime, Duration, Utc};
use rocket::FromFormValue;
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

    pub fn owned_items(&self) -> HashMap<String, bool> {
        ShopItem::get_prices()
            .iter()
            .map(|(k, _)| (k.to_string(), self.data.items.contains(k)))
            .collect()
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

#[derive(
    Serialize_repr, Deserialize_repr, Clone, Debug, Hash, PartialEq, Eq, Copy, FromFormValue,
)]
#[repr(u8)]
pub enum ShopItem {
    DarkMode = 1,
    GayButton = 2,
}

impl ShopItem {
    pub fn get_prices() -> HashMap<ShopItem, u16> {
        let mut prices: HashMap<ShopItem, u16> = HashMap::new();
        prices.insert(ShopItem::DarkMode, 3);
        prices.insert(ShopItem::GayButton, 10);
        prices
    }

    pub fn get_display_prices(profile: Profile) -> Vec<(String, String, u16, bool)> {
        // real name, display name, price, been purchased
        let mut prices = ShopItem::get_prices()
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    camel_to_lowerspace(&k.to_string()),
                    ShopItem::price_with_sell(*v, &profile, k),
                    profile.data.items.contains(k),
                )
            })
            .collect::<Vec<(String, String, u16, bool)>>();
        prices.sort_by(|a, b| a.0.cmp(&b.0)); // sort alphabetically, then by price
        prices.sort_by(|a, b| a.2.cmp(&b.2)); // ensures constant order even for same price items
        prices
    }

    pub fn get_price(profile: &Profile, item: &ShopItem) -> Option<u16> {
        Some(ShopItem::price_with_sell(
            *ShopItem::get_prices().get(&item)?,
            profile,
            item,
        ))
    }

    fn price_with_sell(price: u16, profile: &Profile, item: &ShopItem) -> u16 {
        if profile.data.items.contains(item) {
            ((price as f32) * 0.8) as u16
        } else {
            price
        }
    }
}

impl fmt::Display for ShopItem {
    // for to_string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn camel_to_lowerspace(item: &str) -> String {
    item.chars()
        .into_iter()
        .map(|x| {
            if x.is_uppercase() {
                format!(" {}", &x.to_string().to_lowercase())
            } else {
                x.to_string()
            }
        })
        .collect::<String>()
        .trim_start()
        .to_string()
}

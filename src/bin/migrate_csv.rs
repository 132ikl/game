use std::error::Error;

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use csv::ReaderBuilder;
use game::data::{Profile, ShopItem, UserData};
use game::database::Database;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Row<'a> {
    pub username: &'a str,
    pub next: &'a str,
    pub points: u16,
    pub dark_mode: bool,
    pub gay_button: bool,
    pub hash: &'a str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let db = Database::open();
    let mut rdr = ReaderBuilder::new().from_path("./users.csv")?;
    for record in rdr.records() {
        let record = record?;
        let row: Row = record.deserialize(None)?;
        let next: DateTime<Utc> = DateTime::<Utc>::from_utc(
            NaiveDateTime::parse_from_str(row.next, "%Y-%m-%d %H:%M:%S%.6f").unwrap(),
            Utc,
        );
        let mut items: Vec<ShopItem> = Vec::new();
        if row.gay_button {
            items.push(ShopItem::GayButton)
        }
        if row.dark_mode {
            items.push(ShopItem::DarkMode)
        }
        let data: UserData = UserData {
            username: row.username.to_string(),
            hash: row.hash.to_string(),
            points: row.points,
            next,
            ready: false,
            items,
        };
        let id = db.gen_id();
        let profile = Profile::new(id, data);
        println!("{:?}", &profile);
        db.save_profile(profile);
    }
    Ok(())
}

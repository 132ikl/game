use game::database::Database;

fn main() {
    let db = Database::open();
    let mut profile = db.from_username("test").unwrap();
    profile.data.points = 10;
    db.save_profile(profile)
}

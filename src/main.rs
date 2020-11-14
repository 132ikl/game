#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

mod database;

use bcrypt;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_contrib::{serve::StaticFiles, templates::{tera::Context, Template}};
use serde::Serialize;
use database::database::{Database, Profile, UserData};

#[derive(Serialize)]
struct Player<'a> {
    username: &'a str,
    points: u8,
    ready: bool,
}

#[get("/")]
fn index(flash: Option<FlashMessage>) -> Template {
    let username = "test";
    let points = 10;
    let ready = false;
    let player = Player { username, points, ready };
    let mut context = Context::new();
    match flash {
        Some(x) => context.insert("message", x.msg()),
        None => {}
    }
    context.insert("player", &player);
    Template::render("game", &context)
}

#[get("/flash")]
fn flash() -> Flash<Redirect> {
    Flash::success(Redirect::to("/"), "hello world")
}

fn main() {
    // rocket::ignite()
    //     .mount("/", routes![index, flash])
    //     .mount("/static", StaticFiles::from("./static"))
    //     .attach(Template::fairing())
    //     .launch();
    let username = String::from("pog");
    let password = "password";
    let hash = bcrypt::hash(password, 12).unwrap();
    let points = 0;
    let data = UserData::new(hash, points);
    let profile = Profile::new(username.clone(), data);

    println!("{:?}", profile);
    let db = Database::new();
    db.save_profile(profile);

    let new_profile = db.load_profile(username);
    println!("{:?}", new_profile);
}

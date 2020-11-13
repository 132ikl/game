#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket_contrib::templates::tera::Context;
use rocket_contrib::{serve::StaticFiles, templates::Template};
use serde::Serialize;

#[derive(Serialize)]
struct Player<'a> {
    username: &'a str,
    points: u8,
    ready: bool
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
        None => println!("bruh")
    }
    context.insert("player", &player);
    Template::render("game", &context)
}

#[get("/flash")]
fn flash() -> Flash<Redirect> {
    Flash::success(Redirect::to("/"), "hello world")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, flash])
        .mount("/static", StaticFiles::from("./static"))
        .attach(Template::fairing())
        .launch();
}

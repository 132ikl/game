#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod data;
mod database;

use bcrypt;
use data::{Profile, ShopItem, UserData};
use database::Database;
use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::request;
use rocket::request::FlashMessage;
use rocket::request::Form;
use rocket::request::FromRequest;
use rocket::response::{Flash, Redirect};
use rocket::Outcome;
use rocket::Request;
use rocket_contrib::{
    serve::StaticFiles,
    templates::{tera::Context, Template},
};

#[derive(FromForm)]
struct Login {
    username: String,
    password: String,
}

#[derive(FromForm, Debug)]
struct BuyForm {
    pub buy: bool,
    pub item: ShopItem,
}

impl<'a, 'r> FromRequest<'a, 'r> for Profile {
    type Error = std::convert::Infallible;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Profile, Self::Error> {
        match request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| Database::open().load_profile(cookie.value()))
        {
            Some(x) => Outcome::Success(x),
            None => Outcome::Forward(()),
        }
    }
}

#[get("/")]
fn index(mut profile: Profile, flash: Option<FlashMessage>) -> Template {
    let mut context = Context::new();

    // if profile.update() returns Some(msg), early exit, else get Option<msg> from flash
    if let Some(msg) = profile
        .update()
        .as_deref()
        .or_else(|| flash.as_ref().map(|item| item.msg()))
    {
        context.insert("message", msg);
    }

    context.insert("profile", &profile);
    Template::render("game", &context)
}

#[get("/", rank = 2)]
fn index_redir() -> Redirect {
    Redirect::to("/login")
}

#[get("/get")]
fn get(mut profile: Profile) -> Redirect {
    if profile.update().is_none() {
        profile.data.points += 1;
        Database::open().save_profile(profile)
    };
    Redirect::to("/")
}

#[get("/login")]
fn login_page(flash: Option<FlashMessage>) -> Template {
    let mut context = Context::new();

    if let Some(x) = flash {
        context.insert("message", x.msg())
    }

    Template::render("login", &context)
}

#[post("/register", data = "<form>")]
fn register(form: Form<Login>) -> Flash<Redirect> {
    if form.username.is_empty() || form.password.is_empty() {
        return Flash::error(
            Redirect::to("/login"),
            "Username and password cannot be empty",
        );
    }

    let db = Database::open();

    if db.from_username(&form.username).is_some() {
        return Flash::error(Redirect::to("/login"), "Account already exists");
    }

    let hash = bcrypt::hash(&form.password, 4).unwrap();
    let data = UserData::new(form.username.clone(), hash);
    let profile = Profile::new(db.gen_id().to_string(), data);
    db.save_profile(profile);

    Flash::success(Redirect::to("/login"), "Account creation successful")
}

#[post("/login", data = "<form>")]
fn login(mut cookies: Cookies, form: Form<Login>) -> Result<Redirect, Flash<Redirect>> {
    let err = || Flash::error(Redirect::to("/login"), "Incorrect username/password");

    if form.username.is_empty() || form.password.is_empty() {
        return Err(err());
    }

    let profile: Profile = Database::open()
        .from_username(&form.username)
        .ok_or(err())?;

    let success =
        bcrypt::verify(&form.password, &profile.data.hash).expect("Failed to verify password");
    if !success {
        return Err(err());
    }

    cookies.add_private(Cookie::new("user_id", profile.id));
    Ok(Redirect::to("/"))
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));
    Redirect::to("/")
}

#[get("/leaderboard")]
fn leaderboard(profile: Profile) -> Template {
    let mut sorted: Vec<_> = Database::open()
        .get_profiles()
        .map(|profile| (profile.data.username, profile.data.points))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let mut context = Context::new();
    context.insert("profile", &profile);
    context.insert("leaderboard", &sorted);
    Template::render("leaderboard", &context)
}

#[get("/shop")]
fn shop(profile: Profile) -> Template {
    let mut context = Context::new();
    context.insert("profile", &profile);
    context.insert("shop", &ShopItem::get_display_prices(profile));
    Template::render("shop", &context)
}

#[post("/buy", data = "<form>")]
fn buy(mut profile: Profile, form: Form<BuyForm>) -> Result<Redirect, Redirect> {
    let r = || Redirect::to("/shop");
    let price = ShopItem::get_price(&form.item).ok_or(r())?;
    if profile.data.items.contains(&form.item) {
        // sell if already owned
        profile.data.points += price;
        profile.data.items.retain(|x| x != &form.item);
    } else {
        if profile.data.points >= price {
            profile.data.points -= price;
            profile.data.items.push(form.item);
        }
    }
    println!("{:?}", profile.data.items);
    Database::open().save_profile(profile);
    Ok(r())
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                index_redir,
                login_page,
                login,
                register,
                logout,
                leaderboard,
                shop,
                get,
                buy
            ],
        )
        .mount("/static", StaticFiles::from("./static"))
        .attach(Template::fairing())
        .launch();
}

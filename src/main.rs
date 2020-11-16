#![feature(proc_macro_hygiene, decl_macro, try_trait)]
#[macro_use]
extern crate rocket;

mod data;
mod database;

use std::borrow::Cow;
use std::fs::File;
use std::io::Write;
use std::option::NoneError;
use std::path::PathBuf;

use bcrypt;
use data::{Profile, ShopItem, UserData};
use database::Database;
use rocket::Config;
use rocket::config::Environment;
use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::request;
use rocket::request::FlashMessage;
use rocket::request::Form;
use rocket::request::FromRequest;
use rocket::response::{Flash, Redirect};
use rocket::Outcome;
use rocket::Request;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::{tera::Context, Template};
use rust_embed::RustEmbed;
use tempfile::TempDir;

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

fn get_context(profile: &Profile) -> Context {
    let mut context = Context::new();
    context.insert("profile", &profile);
    profile
        .owned_items()
        .iter()
        .for_each(|(k, v)| context.insert(k, v));
    context
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
    let mut context = get_context(&profile);

    // if profile.update() returns Some(msg), early exit, else get Option<msg> from flash
    if let Some(msg) = profile
        .update()
        .as_deref()
        .or_else(|| flash.as_ref().map(|item| item.msg()))
    {
        context.insert("message", msg);
    }
    context.insert("profile", &profile); // overwrite old profile with newly ready-set profile
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

    let mut context = get_context(&profile);
    context.insert("leaderboard", &sorted);
    Template::render("leaderboard", &context)
}

#[get("/shop")]
fn shop(profile: Profile) -> Template {
    let mut context = get_context(&profile);
    context.insert("shop", &ShopItem::get_display_prices(profile));
    Template::render("shop", &context)
}

#[post("/buy", data = "<form>")]
fn buy(mut profile: Profile, form: Form<BuyForm>) -> Result<Redirect, Redirect> {
    let r = || Redirect::to("/shop");
    let price = ShopItem::get_price(&profile, &form.item).ok_or(r())?;
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
    Database::open().save_profile(profile);
    Ok(r())
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Static;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

fn extract_embedded<A: RustEmbed>(_: A) -> Option<TempDir> {
    let dir: TempDir = TempDir::new().ok()?;
    let dir_path = dir.path();
    for filename in A::iter() {
        let name: &str = &*filename;
        let path: PathBuf = dir_path.join(name);
        let data: Cow<[u8]> = A::get(&name)?;
        let mut file = File::create(path).ok()?;
        file.write_all(&data).ok();
    }
    Some(dir)
}

fn main() -> Result<(), NoneError> {
    let static_dir: TempDir = extract_embedded(Static)?;
    let static_path: String = static_dir.into_path().to_str()?.to_owned();
    let template_dir: TempDir = extract_embedded(Templates)?;
    let template_path: String = template_dir.into_path().to_str()?.to_owned();

    let config = Config::build(Environment::Staging)
        .extra("template_dir", template_path)
        .finalize().ok()?;

    rocket::custom(config)
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
                buy,
            ],
        )
        .mount("/static", StaticFiles::from(static_path))
        .attach(Template::fairing())
        .launch();
    Ok(())
}

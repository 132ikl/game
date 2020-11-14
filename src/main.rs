#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod database;

use bcrypt;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use database::database::{Database, Profile, UserData};
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

impl<'a, 'r> FromRequest<'a, 'r> for Profile {
    type Error = std::convert::Infallible;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Profile, Self::Error> {
        let db = Database::open();
        let id_opt: Option<String> = request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok());
        let id = match id_opt {
            Some(x) => x,
            None => return Outcome::Forward(()),
        };
        let profile = db.load_profile(id);
        match profile {
            Some(x) => Outcome::Success(x),
            None => Outcome::Forward(()),
        }
    }
}

#[get("/")]
fn index(profile: Profile, flash: Option<FlashMessage>) -> Template {
    let mut context = Context::new();
    if let Some(x) = flash {
        context.insert("message", x.msg());
    }
    match update_profile(profile.clone()) {
        Ok(new) => context.insert("profile", &new),
        Err((new, msg)) => {
            context.insert("profile", &new);
            context.insert("message", &msg);
        }
    };
    Template::render("game", &context)
}

#[get("/", rank = 2)]
fn index_redir() -> Redirect {
    Redirect::to("/login")
}

fn update_profile(profile: Profile) -> Result<Profile, (Profile, String)> {
    let mut profile = profile.clone();
    let next = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(profile.data.next, 0), Utc);
    let now: DateTime<Utc> = Utc::now();
    if now > next {
        let next = (now + Duration::days(1)).timestamp();
        profile.data.next = next;
        profile.data.ready = true;
        return Ok(profile);
    } else {
        profile.data.ready = false;
        let secs = (next - now).num_seconds();
        let h = secs / 3600;
        let rem = secs % 3600;
        let m = rem / 60;
        return Err((profile, format!("come back in {}h {}m to get again", h, m)));
    }
}

#[get("/get")]
fn get(profile: Profile) -> Redirect {
    let rd = Redirect::to("/");
    let mut new_profile: Profile = match update_profile(profile) {
        Ok(profile) => profile,
        Err(_) => return rd,
    };
    new_profile.data.points = new_profile.data.points + 1;
    let db = Database::open();
    db.save_profile(new_profile);
    rd
}

#[get("/login")]
fn login_page(flash: Option<FlashMessage>) -> Template {
    let mut context = Context::new();
    match flash {
        Some(x) => context.insert("message", x.msg()),
        None => {}
    }
    Template::render("login", &context)
}

#[post("/register", data = "<form>")]
fn register(form: Form<Login>) -> Flash<Redirect> {
    let db = Database::open();
    let id: Option<String> = db.get_id(form.username.clone());
    if id.is_some() {
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
    let err = Err(Flash::error(
        Redirect::to("/login"),
        "Incorrect username/password",
    ));
    let db = Database::open();
    let id: Option<String> = db.get_id(form.username.clone());
    let profile: Profile = match id {
        Some(x) => match db.load_profile(x) {
            Some(x) => x,
            None => return err,
        },
        None => return err,
    };
    let success =
        bcrypt::verify(&form.password, &profile.data.hash).expect("Failed to verify password");
    if !success {
        return err;
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
    let db = Database::open();
    let mut sorted: Vec<_> = db
        .get_profiles()
        .into_iter()
        .map(|profile| (profile.data.username, profile.data.points))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let mut context = Context::new();
    context.insert("profile", &profile);
    context.insert("leaderboard", &sorted);
    Template::render("leaderboard", &context)
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
                get
            ],
        )
        .mount("/static", StaticFiles::from("./static"))
        .attach(Template::fairing())
        .launch();
}

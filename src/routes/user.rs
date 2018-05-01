use rocket::request::Form;
use rocket::response::Redirect;
use rocket_contrib::Template;
use serde_json;
use std::collections::HashMap;

use activity_pub::ActivityPub;
use activity_pub::actor::Actor;
use activity_pub::inbox::Inbox;
use activity_pub::outbox::Outbox;
use db_conn::DbConn;
use models::instance::Instance;
use models::users::*;

#[get("/me")]
fn me(user: User) -> String {
    format!("Logged in as {}", user.username.to_string())
}

#[get("/@/<name>", rank = 2)]
fn details(name: String, conn: DbConn) -> Template {
    let user = User::find_by_fqn(&*conn, name).unwrap();
    Template::render("users/details", json!({
        "user": serde_json::to_value(user).unwrap()
    }))
}

#[get("/@/<name>", format = "application/activity+json", rank = 1)]
fn activity_details(name: String, conn: DbConn) -> ActivityPub {
    let user = User::find_local(&*conn, name).unwrap();
    user.as_activity_pub(&*conn)
}

#[get("/users/new")]
fn new() -> Template {
    Template::render("users/new", HashMap::<String, i32>::new())
}

#[derive(FromForm)]
struct NewUserForm {
    username: String,
    email: String,
    password: String,
    password_confirmation: String
}

#[post("/users/new", data = "<data>")]
fn create(conn: DbConn, data: Form<NewUserForm>) -> Redirect {
    let inst = Instance::get_local(&*conn).unwrap();
    let form = data.get();

    if form.password == form.password_confirmation {
        User::insert(&*conn, NewUser::new_local(
            form.username.to_string(),
            form.username.to_string(),
            !inst.has_admin(&*conn),
            String::from(""),
            form.email.to_string(),
            User::hash_pass(form.password.to_string()),
            inst.id
        )).update_boxes(&*conn);
    }
    
    Redirect::to(format!("/@/{}", data.get().username).as_str())
}

#[get("/@/<name>/outbox")]
fn outbox(name: String, conn: DbConn) -> Outbox {
    let user = User::find_local(&*conn, name).unwrap();
    user.outbox(&*conn)
}

#[post("/@/<name>/inbox", data = "<data>")]
fn inbox(name: String, conn: DbConn, data: String) -> String {
    let user = User::find_local(&*conn, name).unwrap();
    let act: serde_json::Value = serde_json::from_str(&data[..]).unwrap();
    user.received(&*conn, act);
    String::from("")
}

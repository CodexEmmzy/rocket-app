// #![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
// #[macro_use] extern crate rocket_sync_db_pools;
// #[macro_use] extern crate rocket_contrib;




mod auth;
mod models;
mod schema;
mod repositories;

// use diesel::connection::*;
// use diesel_migrations::EmbedMigrations;
use auth::BasicAuth;
use diesel_migrations::embed_migrations;
use models::{Rustacean, NewRustacean};

use rocket::Build;
use rocket::Rocket;
use rocket::serde::json::{Value, json, Json};
use rocket::response::status;
use rocket::http::Status;
use rocket::fairing::AdHoc;
use rocket_sync_db_pools::database;
use repositories::RustaceanRepository;
// use diesel_migrations::run_migrations;
use diesel::SqliteConnection;

embed_migrations!();

#[database("sqlite")]
struct DBConn(SqliteConnection);

impl diesel::connection::SimpleConnection for DBConn {
    fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        todo!()
    }
}



#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, db: DBConn) -> Result<Value, status::Custom<Value>>  {
    db.run(|c| {
        RustaceanRepository::find_mutiples(c, 100)
            .map(|rustacean| json!(rustacean))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}

#[get("/rustaceans/<id>")]
async fn view_rustaceans(id: i32, _auth: BasicAuth, db: DBConn) -> Result<Value, status::Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::find(c, id)  
            .map(|rustacean| json!(rustacean))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}

#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustaceans(_auth: BasicAuth, db: DBConn, new_rustacean: Json<NewRustacean>) -> Result<Value, status::Custom<Value>> {
    db.run(|c| {
        RustaceanRepository::create(c, new_rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}

#[put("/rustaceans/<id>", format = "json", data = "<rustacean>")]
async fn update_rustaceans(id: i32, _auth: BasicAuth, db: DBConn, rustacean: Json<Rustacean>) -> Result<Value, status::Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::save(c, id, rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}  

#[delete("/rustaceans/<id>")]
async fn delete_rustaceans(id: i32, _auth: BasicAuth, db: DBConn) -> Result<status::NoContent, status::Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::delete(c, id)
            .map(|_| status::NoContent)
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}

#[catch(404)]
fn not_found() -> Value {
    json!("Not Found!")
}


async fn run_db_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let conn = DBConn::get_one(&rocket).await
        .expect("failed to retrieve database connection");

    match embedded_migrations::run(&conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            println!("Failed to run database migration: {:?}", e);
            Err(rocket)
        }
    }

}



#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", routes![
            get_rustaceans,
            view_rustaceans,
            create_rustaceans,
            update_rustaceans,
            delete_rustaceans
        ])
        .register("/", catchers![
            not_found
        ])
        .attach(DBConn::fairing())
        .attach(AdHoc::try_on_ignite("Running DB migration", run_db_migrations))
        .launch()
        .await;
}

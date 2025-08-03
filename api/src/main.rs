use actix_web::{ App, HttpServer};
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use crate::routes::auth::{sign_in, sign_up};

mod routes;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // let connection = &mut es 



   HttpServer::new(move || {
        App::new()
            .service(sign_in)
            .service(sign_up)
   })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


fn establish_connection() -> PgConnection {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|e| panic!("Filed to connect, error: {e}"))
}
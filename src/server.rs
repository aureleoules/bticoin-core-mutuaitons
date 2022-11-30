use actix_cors::Cors;
use actix_web::{
    get, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use redis::{Commands, JsonCommands};
use time::OffsetDateTime;

use crate::{Mutation, MutationStatus};
fn store_mutation(redis_client: &redis::Client, mutation: Mutation) {
    let key = mutation.id.clone();
    let mut con = redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    if con.exists(&key).unwrap() {
        println!("Mutation already exists: {}", mutation.id);
        return;
    }

    let _: () = con
        .json_set(&key, "$", &mutation)
        .expect("Failed to store mutation");
    println!("Stored mutation: {}", mutation.id);
}
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
#[get("/mutations")]
async fn list_mutations(redis_client: web::Data<redis::Client>) -> impl Responder {
    let mut mutations = Vec::new();

    let mut con = redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let keys: Vec<String> = con.keys("*").expect("Failed to get keys");

    for key in keys {
        let mutation: String = con
            .json_get(&key, "$")
            .expect("Failed to get mutation from redis");

        let mutation: Vec<Mutation> =
            serde_json::from_slice(&mutation.as_bytes()).expect("Failed to deserialize mutation");
        let mutation = mutation[0].clone();
        mutations.push(mutation);
    }

    HttpResponse::Ok().json(mutations)
}
#[post("/get_work")]
async fn get_work(redis_client: web::Data<redis::Client>) -> impl Responder {
    let mut con = redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let keys: Vec<String> = con.keys("*").expect("Failed to get keys");

    for key in keys {
        let mutation: String = con
            .json_get(&key, "$")
            .expect("Failed to get mutation from redis");

        println!("Got mutation: {}", mutation);

        let mut mutation: Vec<Mutation> =
            serde_json::from_slice(&mutation.as_bytes()).expect("Failed to deserialize mutation");
        let mut mutation = mutation.pop().unwrap();
        if mutation.status == MutationStatus::Pending {
            mutation.status = MutationStatus::Running;
            mutation.start_time = Some(OffsetDateTime::now_utc());
            store_mutation(&redis_client, mutation.clone());
            return HttpResponse::Ok().json(mutation);
        }
    }

    HttpResponse::NoContent().finish()
}

#[post("/mutations/{id}")]
async fn submit_mutation_result(
    redis_client: web::Data<redis::Client>,
    id: web::Path<String>,
    status: web::Json<MutationStatus>,
) -> impl Responder {
    let key = id.into_inner();
    let mut con = redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let mutation: String = con
        .json_get(&key, "$")
        .expect("Failed to get mutation from redis");

    let mut mutation: Vec<Mutation> =
        serde_json::from_slice(&mutation.as_bytes()).expect("Failed to deserialize mutation");
    let mut mutation = mutation.pop().unwrap();
    mutation.status = status.into_inner();
    mutation.end_time = Some(OffsetDateTime::now_utc());
    store_mutation(&redis_client, mutation.clone());

    HttpResponse::NotFound().finish()
}

#[post("/mutations")]
async fn add_mutations(
    redis_client: web::Data<redis::Client>,
    mutations: web::Json<Vec<Mutation>>,
) -> impl Responder {
    for mutation in mutations.into_inner() {
        store_mutation(&redis_client, mutation);
    }

    HttpResponse::Ok().finish()
}

pub async fn run(host: String, port: u16, db: String) {
    println!("Starting server on {}:{}", host, port);

    let redis_client = redis::Client::open("redis://127.0.0.1:6379/").expect("err");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(redis_client.clone()))
            .service(list_mutations)
            .service(get_work)
            .service(index)
            .service(add_mutations)
            .service(submit_mutation_result)
    })
    .bind(format!("{}:{}", host, port))
    .unwrap()
    .run()
    .await;
}

use actix_cors::Cors;
use actix_web::{
    get, post,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use redis::{Commands, JsonCommands};
use time::OffsetDateTime;

use crate::{Mutation, MutationResult, MutationStatus};
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
async fn list_mutations(ctx: web::Data<Context>) -> impl Responder {
    let mut mutations = Vec::new();

    let mut con = ctx
        .redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let keys: Vec<String> = con.keys("*").expect("Failed to get keys");

    for key in keys {
        let mutation: String = con
            .json_get(&key, "$")
            .expect("Failed to get mutation from redis");

        let mutation: Vec<Mutation> =
            serde_json::from_slice(mutation.as_bytes()).expect("Failed to deserialize mutation");
        let mutation = mutation[0].clone();
        mutations.push(mutation);
    }

    HttpResponse::Ok().json(mutations)
}

#[get("/mutations/{id}")]
async fn get_mutation(ctx: web::Data<Context>, req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap_or("0");

    let mut con = ctx
        .redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let mutation: String = con
        .json_get(&id, "$")
        .expect("Failed to get mutation from redis");

    let mutation: Vec<Mutation> =
        serde_json::from_slice(mutation.as_bytes()).expect("Failed to deserialize mutation");
    let mutation = mutation[0].clone();

    HttpResponse::Ok().json(mutation)
}

#[post("/get_work")]
async fn get_work(request: HttpRequest, ctx: web::Data<Context>) -> impl Responder {
    // Get Authorization header
    let auth_header = request.headers().get("Authorization");
    if auth_header.is_none() {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    }

    // Get the token from the Authorization header
    let auth_header = auth_header.unwrap().to_str().unwrap();
    if !is_authorized(auth_header.to_string(), ctx.tokens.clone()) {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    let mut con = ctx
        .redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let keys: Vec<String> = con.keys("*").expect("Failed to get keys");

    for key in keys {
        let mutation: String = con
            .json_get(&key, "$")
            .expect("Failed to get mutation from redis");

        println!("Got mutation: {}", mutation);

        let mut mutation: Vec<Mutation> =
            serde_json::from_slice(mutation.as_bytes()).expect("Failed to deserialize mutation");
        let mut mutation = mutation.pop().unwrap();
        if mutation.status == MutationStatus::Pending {
            mutation.status = MutationStatus::Running;
            mutation.start_time = Some(OffsetDateTime::now_utc());
            let _: () = con
                .json_set(&key, "$", &mutation)
                .expect("Failed to store mutation");
            return HttpResponse::Ok().json(mutation);
        }
    }

    HttpResponse::NoContent().finish()
}

#[post("/mutations/{id}")]
async fn submit_mutation_result(
    request: HttpRequest,
    ctx: web::Data<Context>,
    id: web::Path<String>,
    result: web::Json<MutationResult>,
) -> impl Responder {
    let auth_header = request.headers().get("Authorization");
    if auth_header.is_none() {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    }

    // Get the token from the Authorization header
    let auth_header = auth_header.unwrap().to_str().unwrap();
    if !is_authorized(auth_header.to_string(), ctx.tokens.clone()) {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    let key = id.into_inner();
    println!("Got key: {}", key);
    let mut con = ctx
        .redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let mutation: String = con
        .json_get(&key, "$")
        .expect("Failed to get mutation from redis");

    let mut mutation: Vec<Mutation> =
        serde_json::from_slice(mutation.as_bytes()).expect("Failed to deserialize mutation");
    let mut mutation = mutation.pop().unwrap();
    mutation.status = result.status.clone();
    mutation.stdout = result.stdout.clone();
    mutation.stderr = result.stderr.clone();
    mutation.end_time = Some(OffsetDateTime::now_utc());

    println!("Got mutation result: {:?}", mutation.status);

    let _: () = con
        .json_set(&key, "$", &mutation)
        .expect("Failed to store mutation");

    HttpResponse::Ok().finish()
}

#[post("/mutations")]
async fn add_mutations(
    request: HttpRequest,
    ctx: web::Data<Context>,
    mutations: web::Json<Vec<Mutation>>,
) -> impl Responder {
    let auth_header = request.headers().get("Authorization");
    if auth_header.is_none() {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    }

    // Get the token from the Authorization header
    let auth_header = auth_header.unwrap().to_str().unwrap();
    if !is_authorized(auth_header.to_string(), ctx.tokens.clone()) {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    for mutation in mutations.into_inner() {
        store_mutation(&ctx.redis_client, mutation);
    }

    HttpResponse::Ok().finish()
}

#[derive(Clone, Debug)]
struct Token {
    Owner: String,
    Token: String,
}

struct Context {
    redis_client: redis::Client,
    tokens: Vec<Token>,
}

fn is_authorized(token: String, tokens: Vec<Token>) -> bool {
    for t in tokens {
        if t.Token == token {
            return true;
        }
    }
    false
}

pub async fn run(host: String, port: u16, redis_ip: String, tokens: Vec<String>) {
    println!("Starting server on {}:{}", host, port);

    // Parse tokens : Owner:Token
    let mut parsed_tokens = Vec::new();
    for token in tokens {
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != 2 {
            panic!("Invalid token: {}", token);
        }
        parsed_tokens.push(Token {
            Owner: parts[0].to_string(),
            Token: parts[1].to_string(),
        });

        println!("Added token for {}", parts[0]);
    }

    let redis_client = redis::Client::open("redis://".to_string() + &redis_ip)
        .expect("Failed to connect to redis");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(Context {
                redis_client: redis_client.clone(),
                tokens: parsed_tokens.clone(),
            }))
            .service(list_mutations)
            .service(get_work)
            .service(index)
            .service(add_mutations)
            .service(submit_mutation_result)
            .service(get_mutation)
    })
    .bind(format!("{}:{}", host, port))
    .unwrap()
    .run()
    .await;
}

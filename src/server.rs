use actix_cors::Cors;
use actix_web::{
    get, post,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use redis::{Commands, JsonCommands};
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

use crate::{Mutation, MutationResult, MutationStatus};
fn store_mutation(ctx: &Context, mutation: Mutation) {
    let key = mutation.id.clone();
    let mut con = ctx.redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    if con.exists(&key).unwrap() {
        println!("Mutation already exists: {}", mutation.id);
        return;
    }

    // Run bash script
    let mut cmd = std::process::Command::new("bash");
    // checkout master, create branch, commit, push
    let patch_path = format!("/tmp/{}.patch", mutation.id);
    std::fs::write(&patch_path, mutation.patch.clone()).expect("Failed to write patch file");

    cmd.arg("-c")
        .arg(format!(
            "cd {} && git checkout master && git checkout -b mutation-{} && patch {} {} && git add . && git commit -m \"{}\" && git push -f origin mutation-{}",
            ctx.mutation_repo, mutation.id, mutation.file, patch_path, mutation.id, mutation.id
        ));

    let output = cmd.output().expect("Failed to execute command");
    println!("Output: {}", String::from_utf8_lossy(&output.stdout));

    let status = output.status;
    if !status.success() {
        println!("Failed to run command: {}", status);
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Params {
    status: Option<MutationStatus>
}

#[get("/mutations")]
async fn list_mutations(req: HttpRequest, ctx: web::Data<Context>) -> impl Responder {
    let mut mutations = Vec::new();

    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let status_filter = params.status.as_ref();

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
        let mut mutation = mutation[0].clone();
        if status_filter.is_some() && &mutation.status != status_filter.unwrap() {
            continue;
        }

        mutation.stdout = None;
        mutation.stderr = None;
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
    let owner = is_authorized(auth_header.to_string(), ctx.tokens.clone());
    if owner.is_none() {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    let key = id.into_inner();
    println!("Received result for mutation {} from {}: {:?}", key, owner.unwrap(), result.status);
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
    if result.stderr.is_some() {
        mutation.stderr = result.stderr.clone();
    }
    if result.stdout.is_some() {
        mutation.stdout = result.stdout.clone();
    }
    mutation.end_time = Some(OffsetDateTime::now_utc());

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
    if is_authorized(auth_header.to_string(), ctx.tokens.clone()).is_none() {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    for mutation in mutations.into_inner() {
        store_mutation(&ctx, mutation);
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
    mutation_repo: String,
}

fn is_authorized(token: String, tokens: Vec<Token>) -> Option<String> {
    for t in tokens {
        if t.Token == token {
            return Some(t.Owner);
        }
    }

    None
}

pub async fn run(host: String, port: u16, redis_ip: String, tokens: Vec<String>, mutation_repo: String) {
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
            .app_data(web::JsonConfig::default().limit(1024 * 1024 * 50))
            .app_data(web::PayloadConfig::new(1 << 25))
            .app_data(web::Data::new(Context {
                redis_client: redis_client.clone(),
                tokens: parsed_tokens.clone(),
                mutation_repo: mutation_repo.clone(),
            }))
            .service(list_mutations)
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

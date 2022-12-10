use actix_cors::Cors;
use actix_web::{
    get, post,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use common::{Mutation, MutationResult, MutationStatus};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{self, SqlitePool};

async fn store_mutation(ctx: &Context, mutation: Mutation) {
    let exists = sqlx::query!(
        "SELECT id FROM mutations WHERE patch_md5 = ?",
        mutation.patch_md5
    )
    .fetch_one(&ctx.pool)
    .await
    .is_ok();

    if exists {
        println!("Mutation already exists");
        return;
    }

    let r = sqlx::query(
        "INSERT INTO mutations (patch_md5, file, line, patch, branch, pr_number, status) VALUES (?, ?, ?, ?, ?, ?, ?)",
    ).bind(mutation.patch_md5)
        .bind(mutation.file)
        .bind(mutation.line)
        .bind(mutation.patch)
        .bind(mutation.branch)
        .bind(mutation.pr_number)
        .bind(mutation.status)
        .execute(&ctx.pool)
        .await;

    match r {
        Ok(_) => println!("Mutation stored"),
        Err(e) => println!("Error storing mutation: {}", e),
    }
}
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Params {
    status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MutationListItem {
    id: i64,
    patch_md5: String,
    file: String,
    line: i64,
    patch: String,
    branch: Option<String>,
    pr_number: Option<i64>,
    status: String,
    start_time: Option<i64>,
    end_time: Option<i64>,
}

#[get("/mutations")]
async fn list_mutations(req: HttpRequest, ctx: web::Data<Context>) -> impl Responder {
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();
    let status_filter = params.status.as_ref();

    let default_status = MutationStatus::NotKilled.to_string();
    let filter = status_filter.unwrap_or(&default_status);
    let mutations = sqlx::query_as!(MutationListItem, "SELECT id, patch_md5, file, line, patch, branch, pr_number, status, start_time, end_time FROM mutations WHERE status = ?", filter)
        .fetch_all(&ctx.pool)
        .await
        .unwrap();

    HttpResponse::Ok().json(mutations)
}

#[get("/mutations/{id}")]
async fn get_mutation(ctx: web::Data<Context>, req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap_or("0");

    let mutation = sqlx::query_as!(Mutation, "SELECT * FROM mutations WHERE id = ?", id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

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
    let owner = is_authorized(auth_header.to_string(), ctx.tokens.clone());
    if owner.is_none() {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    let status = MutationStatus::Pending.to_string();
    let mutation = sqlx::query_as!(
        Mutation,
        "SELECT * FROM mutations WHERE status = ? LIMIT 1",
        status
    )
    .fetch_one(&ctx.pool)
    .await;

    match mutation {
        Ok(mutation) => {
            let r = sqlx::query("UPDATE mutations SET status = ?, start_time = ? WHERE id = ?")
                .bind(MutationStatus::Running.to_string())
                .bind(chrono::Utc::now().timestamp())
                .bind(mutation.id)
                .execute(&ctx.pool)
                .await;

            match r {
                Ok(_) => HttpResponse::Ok().json(mutation),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Error updating mutation: {}", e)),
            }
        }
        Err(e) => HttpResponse::NoContent().body(format!("No work available: {}", e)),
    }
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
    println!(
        "Received result for mutation {} from {}: {:?}",
        key,
        owner.unwrap(),
        result.status
    );

    let mutation = sqlx::query_as!(Mutation, "SELECT * FROM mutations WHERE id = ?", key)
        .fetch_one(&ctx.pool)
        .await;

    match mutation {
        Ok(mut mutation) => {
            if result.stderr.is_some() {
                mutation.stderr = result.clone().stderr;
            }

            if result.stdout.is_some() {
                mutation.stdout = result.clone().stdout;
            }

            let r = sqlx::query(
                "UPDATE mutations SET status = ?, end_time = ?, stderr = ?, stdout = ? WHERE id = ?",
            ).bind(result.status.to_string())
                .bind(chrono::Utc::now().timestamp())
                .bind(mutation.stderr)
                .bind(mutation.stdout)
                .bind(mutation.id)
                .execute(&ctx.pool)
                .await;

            match r {
                Ok(_) => HttpResponse::Ok().body("Mutation result stored"),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Error updating mutation: {}", e)),
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error fetching mutation: {}", e))
        }
    }
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
        println!("Received mutation: {:?}:{:?}", mutation.file, mutation.line);
        store_mutation(&ctx, mutation).await;
    }

    HttpResponse::Ok().finish()
}

#[derive(Clone, Debug)]
struct Token {
    owner: String,
    token: String,
}

struct Context {
    pool: SqlitePool,
    tokens: Vec<Token>,
}

fn is_authorized(token: String, tokens: Vec<Token>) -> Option<String> {
    for t in tokens {
        if t.token == token {
            return Some(t.owner);
        }
    }

    None
}

pub async fn run(host: String, port: u16, db: String, tokens: Vec<String>) -> std::io::Result<()> {
    println!("Starting server on {}:{}", host, port);

    // Parse tokens : Owner:Token
    let mut parsed_tokens = Vec::new();
    for token in tokens {
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != 2 {
            panic!("Invalid token: {}", token);
        }
        parsed_tokens.push(Token {
            owner: parts[0].to_string(),
            token: parts[1].to_string(),
        });

        println!("Added token for {}", parts[0]);
    }

    let pool = sqlite::SqlitePool::connect(&db)
        .await
        .expect("Failed to connect to database");

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
                pool: pool.clone(),
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
    .await?;

    Ok(())
}

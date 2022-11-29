use actix_cors::Cors;
use actix_web::{
    get, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use kv::*;
use time::OffsetDateTime;

use crate::{Mutation, MutationStatus};
fn store_mutation(bucket: Bucket<String, Json<Mutation>>, mutation: Mutation) {
    let key = mutation.id.clone();

    let m = bucket.get(&key);
    if m.is_ok() && m.unwrap().is_some() {
        println!("Mutation already exists: {}", mutation.id);
        return;
    }

    bucket.set(&key, &Json(mutation.clone())).unwrap();
    bucket.flush();

    println!("Stored mutation: {}", mutation.id);
}
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
#[get("/mutations")]
async fn list_mutations(bucket: web::Data<Bucket<'_, String, Json<Mutation>>>) -> impl Responder {
    let mut mutations = Vec::new();
    for r in bucket.iter() {
        let value = r.unwrap().value::<Json<Mutation>>().unwrap();
        mutations.push(value.0);
    }

    HttpResponse::Ok().json(mutations)
}
#[post("/get_work")]
async fn get_work(bucket: web::Data<Bucket<'_, String, Json<Mutation>>>) -> impl Responder {
    for r in bucket.iter() {
        let value = r.unwrap().value::<Json<Mutation>>().unwrap();
        if value.0.status == MutationStatus::Pending {
            bucket
                .set(
                    &value.0.id,
                    &Json(Mutation {
                        status: MutationStatus::Running,
                        start_time: Some(OffsetDateTime::now_utc()),
                        ..value.0.clone()
                    }),
                )
                .unwrap();
            bucket.flush();
            return HttpResponse::Ok().json(value.0);
        }
    }

    HttpResponse::NoContent().finish()
}

#[post("/mutations/{id}")]
async fn submit_mutation_result(
    bucket: web::Data<Bucket<'_, String, Json<Mutation>>>,
    id: web::Path<String>,
    status: web::Json<MutationStatus>,
) -> impl Responder {
    let key = id.into_inner();
    let m = bucket.get(&key);
    if let Ok(Some(m)) = m {
        println!("Mutation {}: {:?}", key, status.0);
        let value = m.0;
        bucket
            .set(
                &key,
                &Json(Mutation {
                    status: status.into_inner(),
                    end_time: Some(OffsetDateTime::now_utc()),
                    ..value.clone()
                }),
            )
            .unwrap();
        bucket.flush();
        return HttpResponse::Ok().json(value);
    }

    HttpResponse::NotFound().finish()
}

#[post("/mutations")]
async fn add_mutations(
    bucket: web::Data<Bucket<'_, String, Json<Mutation>>>,
    mutations: web::Json<Vec<Mutation>>,
) -> impl Responder {
    let bucket = bucket.as_ref();
    for mutation in mutations.into_inner() {
        store_mutation(bucket.clone(), mutation);
    }

    HttpResponse::Ok().finish()
}

pub async fn run(host: String, port: u16, db: String) {
    println!("Starting server on {}:{}", host, port);
    let cfg = Config::new(db);

    let store = Store::new(cfg);

    if let Err(e) = store {
        println!("Error: {}", e);
        return;
    }

    let store = store.unwrap();
    let mutations_store = store
        .bucket::<String, Json<Mutation>>(Some("mutations"))
        .unwrap();

    // Run periodic
    let store_clone = mutations_store.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        for r in store_clone.iter() {
            let value = r.unwrap().value::<Json<Mutation>>().unwrap();
            if value.0.status == MutationStatus::Running {
                if let Some(start_time) = value.0.start_time {
                    let now = OffsetDateTime::now_utc();
                    if now - start_time > time::Duration::hours(2) {
                        store_clone
                            .set(
                                &value.0.id,
                                &Json(Mutation {
                                    status: MutationStatus::Pending,
                                    ..value.0.clone()
                                }),
                            )
                            .unwrap();
                    }
                }
            }
        }
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(mutations_store.clone()))
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

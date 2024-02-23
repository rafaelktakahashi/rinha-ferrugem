mod env_reader;
mod model;

use std::{env, time::Duration};

use chrono::Utc;
use model::{ExtSd, Tr, TrReq, TrRes};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = match env::var("PORT") {
        Ok(port) => port,
        Err(e) => {
            println!("Could not read variable PORT. {:?}", e);
            println!("Using default port 7878.");
            String::from("7878")
        }
    };
    let address = String::from("0.0.0.0:") + &port; // Moves the string
    println!("Server will listen on {}", &address);

    let db_connection_string =
        env_reader::read_env_str("DATABASE_URL", "postgres://postgres:999@db:5432/postgres");

    let max_db_connections = env_reader::read_env_u32("MAX_DB_CONNECTIONS", 6);

    let db_pool_timeout = env_reader::read_env_u32("DB_POOL_TIMEOUT", 5000);

    let db_pool = match PgPoolOptions::new()
        .max_connections(max_db_connections)
        .acquire_timeout(Duration::from_millis(db_pool_timeout as u64))
        .connect(&db_connection_string)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            println!("Could not connect to database! {:?}", e);
            return Ok(());
        }
    };

    let keep_alive = env_reader::read_env_u32("KEEPALIVE_DURATION", 15000);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData {
                db_pool: db_pool.clone(),
            }))
            .service(tr)
            .service(ex)
    })
    .keep_alive(Duration::from_millis(keep_alive as u64))
    .bind(address)?
    .run()
    .await
}

struct AppData {
    /// Reference to the pool. The pool is essentially an Arc.
    db_pool: Pool<Postgres>,
}

#[post("/clientes/{id}/transacoes")]
async fn tr(
    path_params: web::Path<String>,
    tr: web::Json<TrReq>,
    app_data: web::Data<AppData>,
) -> impl Responder {
    // Validate id, must be 1, 2, 3, 4 or 5, as no other
    // possible ids can be added during the program's execution.
    let id = path_params.into_inner();
    // Is this cheating? Who knows. The fixed existence of these
    // five ids is literally a requirement, so I'm optimizing
    // around it. I thought about having a separate table for each
    // one, but that may be too shady for too little gain.
    let id: i8 = match id.as_str() {
        "1" => 0x41,
        "2" => 0x42,
        "3" => 0x43,
        "4" => 0x44,
        "5" => 0x45,
        _ => {
            return HttpResponse::NotFound().finish();
        }
    };

    // Next, validate the string length.
    // We have two options:
    // 1. Be a good American and ignore the niche category of "foreign".
    // Only ASCII matters. One character is one byte.
    // 2. Do it right and properly handle any UTF-8 string.
    //
    // We do it the right way because the performance penalty is minimal.
    //
    // Empty description is always disallowed.
    if tr.descricao.is_empty()
    //  More than 40 bytes is always disallowed. Each character is 1 to 4 bytes.
        || tr.descricao.len() > 40
    //  Lastly, count the characters. This requires an iteration through the
    //  string, but that's fine because here the string is known to be small.
        || tr.descricao.chars().count() > 10
    {
        return HttpResponse::UnprocessableEntity().finish();
    }

    // This function does everything in one and returns the row of U
    // when successful and zero rows otherwise.
    let sd = match sqlx::query("SELECT LIMITE, SALDO FROM insert_into_t($1, $2, $3, $4);")
        .bind(id) // Byte that will be mapped to the "char" type
        .bind(tr.valor as i32) // Value as signed integer; unsigned was used for serde validation
        .bind(tr.tipo == 'c') // True for 'c', false for 'd'
        .bind(&tr.descricao) // String that will be mapped to the TEXT type
        .fetch_optional(&app_data.db_pool)
        .await
    {
        Ok(row) => match row {
            // In case of allowed operation, everything will have been updated
            // in the database, and one row of U will be returned.
            Some(row) => TrRes {
                saldo: row.get(1),
                limite: row.get::<i32, usize>(0) as u32, // store as signed
            },
            None => {
                // The function returns 0 rows if the operation is not permitted.
                // The only reason should be insufficient limit in the user's account.
                return HttpResponse::UnprocessableEntity().finish();
            }
        },
        Err(e) => {
            // This must not happen because the function is not expected to fail.
            println!("{e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(sd)
}

#[get("/clientes/{id}/extrato")]
async fn ex(path_params: web::Path<String>, app_data: web::Data<AppData>) -> impl Responder {
    let id = path_params.into_inner();
    let id: i8 = match id.as_str() {
        "1" => 0x41,
        "2" => 0x42,
        "3" => 0x43,
        "4" => 0x44,
        "5" => 0x45,
        _ => {
            return HttpResponse::NotFound().finish();
        }
    };

    let mut tx = match app_data.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            println!("{e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Get balance for the user. This is always necessary.
    let bl = match sqlx::query("SELECT LIMITE, SALDO FROM U WHERE U.ID = $1;")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
    {
        Ok(row) => match row {
            Some(row) => ExtSd {
                total: row.get(1),
                limite: row.get::<i32, usize>(0) as u32, // store as signed
                data_extrato: Utc::now(),
            },
            None => {
                return HttpResponse::InternalServerError().finish();
            }
        },
        Err(_) => {
            // This must not happen.
            return HttpResponse::InternalServerError().finish();
        }
    };

    let ts: Vec<Tr> = match sqlx::query(
        "SELECT VALOR, TIPO, DESCRICAO, W FROM T WHERE U_ID=$1 ORDER BY W DESC LIMIT 10;",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows
            .iter()
            .map(|r| model::Tr {
                valor: r.get::<i32, usize>(0) as u32,
                tipo: r.get::<bool, usize>(1),
                descricao: r.get(2),
                realizada_em: r.get(3),
            })
            .collect(),
        Err(e) => {
            println!("{e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Always return 200
    HttpResponse::Ok().json(model::ExtRes {
        saldo: bl,
        ultimas_transacoes: ts,
    })
}

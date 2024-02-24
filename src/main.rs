mod env_reader;
mod model;
mod user_id_cache;

use std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Utc;
use model::{ExtSd, Tr, TrReq, TrRes};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use user_id_cache::UserIdCache;

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

    // Query the users table to get the list of user ids at the start of execution.
    let user_ids: Vec<i8> = match sqlx::query("SELECT ID FROM U;").fetch_all(&db_pool).await {
        Ok(rows) => rows.iter().map(|u| u.get::<i8, usize>(0)).collect(),
        Err(e) => {
            println!("{e}");
            Vec::new()
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData {
                db_pool: db_pool.clone(),
                id_cache: Arc::new(Mutex::new(UserIdCache::new(&user_ids))),
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
    /// In-memory cache of which user ids exist.
    id_cache: Arc<Mutex<UserIdCache>>,
}

#[post("/clientes/{id}/transacoes")]
async fn tr(
    path_params: web::Path<String>,
    tr: web::Json<TrReq>,
    app_data: web::Data<AppData>,
) -> impl Responder {
    let id = match validate_id_exists(path_params.into_inner().as_str(), &app_data).await {
        x if x >= 0 => x,
        _ => return HttpResponse::NotFound().finish(),
    };

    // Type can only be 'c' or 'd'
    if tr.tipo != 'd' && tr.tipo != 'c' {
        return HttpResponse::UnprocessableEntity().finish();
    }

    // The reason why TrReq has an optional string is because that's the
    // simplest way to explicitly fail with 422 (and not 400) when it's
    // missing.
    let descricao = match &tr.descricao {
        Some(s) => s,
        None => return HttpResponse::UnprocessableEntity().finish(),
    };

    // Next, validate the string length.
    // We have two options:
    // 1. Be a good American and ignore the niche category of "foreign".
    //    Only ASCII matters. One character is one byte.
    // 2. Do it right and properly handle any UTF-8 string.
    //
    // We do it the right way because the performance penalty is minimal.
    //
    // Empty description is always disallowed.
    if descricao.is_empty()
    //  More than 40 bytes is always disallowed. Each character is 1 to 4 bytes.
        || descricao.len() > 40
    //  Lastly, count the characters. This requires an iteration through the
    //  string, but that's fine because here the string is known to be small.
        || descricao.chars().count() > 10
    {
        return HttpResponse::UnprocessableEntity().finish();
    }

    // This function does everything in one and returns the row of U when
    // successful and zero rows otherwise.
    let sd = match sqlx::query("SELECT LIMITE, SALDO FROM insert_into_t($1, $2, $3, $4);")
        .bind(id) // Byte that will be mapped to the "char" type
        .bind(tr.valor as i32) // Value as signed integer; unsigned was declared for serde validation
        .bind(tr.tipo == 'c') // True for 'c', false for 'd'
        .bind(&descricao) // String that will be mapped to the TEXT type
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
    let id = match validate_id_exists(path_params.into_inner().as_str(), &app_data).await {
        x if x >= 0 => x,
        _ => return HttpResponse::NotFound().finish(),
    };

    // Get a transaction and reuse it. We'll need two queries.
    // I guess I could've made a function to perform the two (unrelated) queries at once,
    // but that's a _weird_ optimization and it wasn't really necessary.
    let mut tx = match app_data.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            println!("{e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Get balance for the user. This is always necessary.
    let bl = match sqlx::query(
        "SELECT LIMITE, SALDO, pg_advisory_xact_lock($2) FROM U WHERE U.ID = $1;",
    )
    .bind(id)
    .bind(id as i64)
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
        "SELECT VALOR, TIPO, DESCRICAO, W, pg_advisory_xact_lock($2) FROM T WHERE U_ID=$1 ORDER BY W DESC LIMIT 10;",
    )
    .bind(id)
    .bind(id as i64)
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

// Don't do this in serious code, seriously.
type FalseOrU7 = i8;

/// Check if the id exists in the database.
///
/// In case the id exists in the database, this function will return
/// the parsed byte as a positive number.
/// In case the id doesn't exist in the database, or any other error
/// occurred (such as an error parsing the string), this will return
/// a negative value.
async fn validate_id_exists(id: &str, app_data: &web::Data<AppData>) -> FalseOrU7 {
    let _false = -1;

    // First, parse the number.
    let parsed_byte: i8 = match id.parse() {
        Ok(b) => b,
        // Anything not representable by a byte is not in the database, because
        // the id column's type is one byte long.
        Err(_) => return _false,
    };
    // We store the byte in the database with 64 added to it,
    // for no particular reason.
    let u_id = parsed_byte + 0x40;

    // The return value of this function _looks_ like a boolean, but
    // actually contains the parsed id in case of success.
    let _true = u_id;

    // Check if the user id cache knows about this. If it does, we skip a
    // database check.
    let user_cache = match app_data.id_cache.lock() {
        Ok(z) => z,
        Err(_) => return _false, // If the mutex fails, just guess the answer.
    };

    match user_cache.check_id(u_id) {
        user_id_cache::UserIdCacheResult::Exists => {
            return _true;
        }
        user_id_cache::UserIdCacheResult::DoesNotExist => {
            return _false;
        }
        user_id_cache::UserIdCacheResult::CacheDoesNotKnow => {
            "↘️";
        }
    }

    // Getting here means the cache doesn't know, because it is currently
    // invalidated.
    // In that case, we have to query the database to check if the id exists.
    match sqlx::query("SELECT 1 FROM U WHERE ID=$1;")
        .fetch_optional(&app_data.db_pool)
        .await
    {
        Ok(opt) => match opt {
            Some(_) => _true,
            None => _false,
        },
        Err(_) => _false, // Guess false
    }
}

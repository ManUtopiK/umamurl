// SPDX-FileCopyrightText: 2023 Emmanuel Salomon <emmanuel.salomon@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{
    delete, get,
    http::StatusCode,
    post, put,
    web::{self, Redirect},
    Either, HttpRequest, HttpResponse, Responder,
};
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use log::{debug, info, warn};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::env;

use crate::AppState;
use crate::{auth, database};
use crate::{auth::is_session_valid, utils};
use UmamurlError::{ClientError, ServerError};

// Store the version number
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Error types
pub enum UmamurlError {
    ServerError,
    ClientError { reason: String },
}

// Define JSON struct for returning success/error data
#[derive(Serialize)]
pub struct JSONResponse {
    pub success: bool,
    pub error: bool,
    pub reason: String,
}

// Define JSON struct for returning backend config
#[derive(Serialize)]
struct BackendConfig {
    version: String,
    site_url: Option<String>,
    allow_capital_letters: bool,
    public_mode: bool,
    public_mode_expiry_delay: i64,
    slug_style: String,
    slug_length: usize,
    try_longer_slug: bool,
    password_configured: bool,
    umami_configured: bool,
}

// Needed to return the short URL to make it easier for programs leveraging the API
#[derive(Serialize)]
struct CreatedURL {
    success: bool,
    error: bool,
    shorturl: String,
    expiry_time: i64,
}

// Struct for returning information about a shortlink in expand
#[derive(Serialize)]
struct LinkInfo {
    success: bool,
    error: bool,
    longurl: String,
    expiry_time: i64,
}

// Struct for query params in /api/all
#[derive(Deserialize)]
pub struct GetReqParams {
    pub page_after: Option<String>,
    pub page_no: Option<i64>,
    pub page_size: Option<i64>,
}

// Define the routes

// Add new links
#[post("/api/new")]
pub async fn add_link(
    req: String,
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = auth::is_api_ok(http, config);
    // If success, add new link
    if result.success {
        match utils::add_link(&req, &data.db, config, false) {
            Ok((shorturl, expiry_time)) => {
                let site_url = config.site_url.clone();
                let shorturl = if let Some(url) = site_url {
                    format!("{url}/{shorturl}")
                } else {
                    let protocol = if config.port == 443 { "https" } else { "http" };
                    let port_text = if [80, 443].contains(&config.port) {
                        String::new()
                    } else {
                        format!(":{}", config.port)
                    };
                    format!("{protocol}://localhost{port_text}/{shorturl}")
                };
                let response = CreatedURL {
                    success: true,
                    error: false,
                    shorturl,
                    expiry_time,
                };
                HttpResponse::Created().json(response)
            }
            Err(ServerError) => {
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when adding the link.".to_string(),
                };
                HttpResponse::InternalServerError().json(response)
            }
            Err(ClientError { reason }) => {
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason,
                };
                HttpResponse::Conflict().json(response)
            }
        }
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If password authentication or public mode is used - keeps backwards compatibility
    } else {
        let result = if auth::is_session_valid(session, config, &data.db) {
            utils::add_link(&req, &data.db, config, false)
        } else if config.public_mode {
            utils::add_link(&req, &data.db, config, true)
        } else {
            return HttpResponse::Unauthorized().body("Not logged in!");
        };
        match result {
            Ok((shorturl, _)) => HttpResponse::Created().body(shorturl),
            Err(ServerError) => HttpResponse::InternalServerError()
                .body("Something went wrong when adding the link.".to_string()),
            Err(ClientError { reason }) => HttpResponse::Conflict().body(reason),
        }
    }
}

// Return all active links
#[get("/api/all")]
pub async fn getall(
    data: web::Data<AppState>,
    session: Session,
    params: web::Query<GetReqParams>,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = auth::is_api_ok(http, config);
    // If success, return all links
    if result.success {
        HttpResponse::Ok().body(utils::getall(&data.db, params.into_inner()))
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If password authentication is used - keeps backwards compatibility
    } else if auth::is_session_valid(session, config, &data.db) {
        HttpResponse::Ok().body(utils::getall(&data.db, params.into_inner()))
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Get information about a single shortlink
#[post("/api/expand")]
pub async fn expand(req: String, data: web::Data<AppState>, http: HttpRequest) -> HttpResponse {
    let result = auth::is_api_ok(http, &data.config);
    if result.success {
        match database::find_url(&req, &data.db) {
            Ok((longurl, expiry_time)) => {
                let body = LinkInfo {
                    success: true,
                    error: false,
                    longurl,
                    expiry_time,
                };
                HttpResponse::Ok().json(body)
            }
            Err(ServerError) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when finding the link.".to_string(),
                };
                HttpResponse::BadRequest().json(body)
            }
            Err(ClientError { reason }) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason,
                };
                HttpResponse::BadRequest().json(body)
            }
        }
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Get information about a single shortlink
#[put("/api/edit")]
pub async fn edit_link(
    req: String,
    session: Session,
    data: web::Data<AppState>,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    if result.success || is_session_valid(session, config, &data.db) {
        match utils::edit_link(&req, &data.db, config) {
            Ok(()) => {
                let body = JSONResponse {
                    success: true,
                    error: false,
                    reason: String::from("Edit was successful."),
                };
                HttpResponse::Created().json(body)
            }
            Err(ServerError) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when editing the link.".to_string(),
                };
                HttpResponse::InternalServerError().json(body)
            }
            Err(ClientError { reason }) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason,
                };
                HttpResponse::BadRequest().json(body)
            }
        }
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Get the site URL
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/siteurl")]
pub async fn siteurl(data: web::Data<AppState>) -> HttpResponse {
    if let Some(url) = &data.config.site_url {
        HttpResponse::Ok().body(url.clone())
    } else {
        HttpResponse::Ok().body("unset")
    }
}

// Get the version number
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/version")]
pub async fn version() -> HttpResponse {
    HttpResponse::Ok().body(format!("Umamurl v{VERSION}"))
}

// Get the user's current role
#[get("/api/whoami")]
pub async fn whoami(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    let acting_user = if result.success || is_session_valid(session, config, &data.db) {
        "admin"
    } else if config.public_mode {
        "public"
    } else {
        "nobody"
    };
    HttpResponse::Ok().body(acting_user)
}

// Get some useful backend config
#[get("/api/getconfig")]
pub async fn getconfig(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    if result.success || is_session_valid(session, config, &data.db) || data.config.public_mode {
        let backend_config = BackendConfig {
            version: VERSION.to_string(),
            allow_capital_letters: config.allow_capital_letters,
            public_mode: config.public_mode,
            public_mode_expiry_delay: config.public_mode_expiry_delay,
            site_url: config.site_url.clone(),
            slug_style: config.slug_style.clone(),
            slug_length: config.slug_length,
            try_longer_slug: config.try_longer_slug,
            password_configured: auth::is_password_configured(config, &data.db),
            umami_configured: config.umami_url.is_some() && config.umami_website_id.is_some()
                || database::get_setting("umami_url", &data.db)
                    .filter(|s| !s.is_empty())
                    .is_some()
                    && database::get_setting("umami_website_id", &data.db)
                        .filter(|s| !s.is_empty())
                        .is_some(),
        };
        HttpResponse::Ok().json(backend_config)
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Struct for Umami config response
#[derive(Serialize)]
struct UmamiConfig {
    umami_url: String,
    umami_website_id: String,
    env_configured: bool,
}

// Struct for Umami config input
#[derive(Deserialize)]
struct UmamiConfigInput {
    umami_url: String,
    umami_website_id: String,
}

// Get Umami analytics configuration
#[get("/api/umami-config")]
pub async fn get_umami_config(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    if result.success || is_session_valid(session, config, &data.db) {
        let env_configured =
            config.umami_url.is_some() || config.umami_website_id.is_some();
        let umami_url = config
            .umami_url
            .clone()
            .or_else(|| database::get_setting("umami_url", &data.db))
            .unwrap_or_default();
        let umami_website_id = config
            .umami_website_id
            .clone()
            .or_else(|| database::get_setting("umami_website_id", &data.db))
            .unwrap_or_default();
        HttpResponse::Ok().json(UmamiConfig {
            umami_url,
            umami_website_id,
            env_configured,
        })
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Set Umami analytics configuration
#[post("/api/umami-config")]
pub async fn set_umami_config(
    req: web::Json<UmamiConfigInput>,
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    if result.success || is_session_valid(session, config, &data.db) {
        // Reject if env vars are set
        if config.umami_url.is_some() || config.umami_website_id.is_some() {
            return HttpResponse::Forbidden().json(JSONResponse {
                success: false,
                error: true,
                reason: "Umami configuration is set via environment variables and cannot be changed from the UI.".to_string(),
            });
        }

        let url_result = database::set_setting("umami_url", &req.umami_url, &data.db);
        let id_result =
            database::set_setting("umami_website_id", &req.umami_website_id, &data.db);

        if url_result.is_ok() && id_result.is_ok() {
            info!("Umami configuration updated via API.");
            HttpResponse::Ok().json(JSONResponse {
                success: true,
                error: false,
                reason: "Umami configuration saved.".to_string(),
            })
        } else {
            HttpResponse::InternalServerError().json(JSONResponse {
                success: false,
                error: true,
                reason: "Failed to save Umami configuration.".to_string(),
            })
        }
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// 404 error page
pub async fn error404() -> impl Responder {
    NamedFile::open_async("./resources/static/404.html")
        .await
        .customize()
        .with_status(StatusCode::NOT_FOUND)
}

// Send analytics to Umami (fire-and-forget)
fn send_umami_event(
    req: &HttpRequest,
    shortlink: &str,
    config: &crate::config::Config,
    db: &Connection,
) {
    // Env var priority, then DB fallback
    let umami_url = config
        .umami_url
        .clone()
        .or_else(|| database::get_setting("umami_url", db).filter(|s| !s.is_empty()));
    let website_id = config
        .umami_website_id
        .clone()
        .or_else(|| database::get_setting("umami_website_id", db).filter(|s| !s.is_empty()));

    let (Some(umami_url), Some(website_id)) = (umami_url, website_id) else {
        return;
    };

    let url = format!("{}/api/send", umami_url);

    let hostname = config
        .site_url
        .as_deref()
        .and_then(|u| u.strip_prefix("https://"))
        .or_else(|| {
            config
                .site_url
                .as_deref()
                .and_then(|u| u.strip_prefix("http://"))
        })
        .unwrap_or("localhost")
        .to_string();

    let referrer = req
        .headers()
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let language = req
        .headers()
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| req.peer_addr().map(|a| a.ip().to_string()))
        .unwrap_or_default();

    let page_url = format!("/{shortlink}");

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "type": "event",
            "payload": {
                "website": website_id,
                "hostname": hostname,
                "url": page_url,
                "referrer": referrer,
                "language": language,
            }
        });

        let mut request = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", &user_agent)
            .json(&payload);

        if !ip.is_empty() {
            request = request.header("X-Forwarded-For", &ip);
        }

        if let Err(e) = request.send().await {
            warn!("Failed to send Umami event: {e}");
        } else {
            debug!("Umami event sent for {}", page_url);
        }
    });
}

// Handle a given shortlink
#[get("/{shortlink}")]
pub async fn link_handler(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let shortlink_str = shortlink.as_str();
    if let Ok(longlink) = database::find_url_for_redirect(shortlink_str, &data.db) {
        send_umami_event(&req, shortlink_str, &data.config, &data.db);
        if data.config.use_temp_redirect {
            Either::Left(Redirect::to(longlink))
        } else {
            // Defaults to permanent redirection
            Either::Left(Redirect::to(longlink).permanent())
        }
    } else {
        Either::Right(
            NamedFile::open_async("./resources/static/404.html")
                .await
                .customize()
                .with_status(StatusCode::NOT_FOUND),
        )
    }
}

// Handle login
#[post("/api/login")]
pub async fn login(req: String, session: Session, data: web::Data<AppState>) -> HttpResponse {
    let config = &data.config;
    // Check env var password first, then DB-stored password
    let authorized = if let Some(password) = &config.password {
        if config.hash_algorithm.is_some() {
            debug!("Using Argon2 hash for password validation.");
            let hash = PasswordHash::new(password).expect("The provided password hash is invalid.");
            Some(
                Argon2::default()
                    .verify_password(req.as_bytes(), &hash)
                    .is_ok(),
            )
        } else {
            Some(password == &req)
        }
    } else if database::get_setting("password", &data.db).is_some() {
        // Password stored in DB, validate with Argon2
        Some(auth::validate_db_password(&req, &data.db))
    } else {
        None
    };
    if config.api_key.is_some() {
        if let Some(valid_pass) = authorized {
            if !valid_pass {
                warn!("Failed login attempt!");
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Wrong password!".to_string(),
                };
                return HttpResponse::Unauthorized().json(response);
            }
        }
        session
            .insert("umamurl-auth", auth::gen_token())
            .expect("Error inserting auth token.");

        let response = JSONResponse {
            success: true,
            error: false,
            reason: "Correct password!".to_string(),
        };
        info!("Successful login.");
        HttpResponse::Ok().json(response)
    } else {
        if let Some(valid_pass) = authorized {
            if !valid_pass {
                warn!("Failed login attempt!");
                return HttpResponse::Unauthorized().body("Wrong password!");
            }
        }
        session
            .insert("umamurl-auth", auth::gen_token())
            .expect("Error inserting auth token.");

        info!("Successful login.");
        HttpResponse::Ok().body("Correct password!")
    }
}

// Set password (only works when no password is configured)
#[post("/api/set-password")]
pub async fn set_password(req: String, data: web::Data<AppState>) -> HttpResponse {
    let config = &data.config;

    // Reject if env var password is set
    if config.password.is_some() {
        return HttpResponse::Forbidden().json(JSONResponse {
            success: false,
            error: true,
            reason: "Password is already configured via environment variable.".to_string(),
        });
    }

    // Reject if DB password already exists
    if database::get_setting("password", &data.db).is_some() {
        return HttpResponse::Forbidden().json(JSONResponse {
            success: false,
            error: true,
            reason: "Password is already configured.".to_string(),
        });
    }

    if req.trim().is_empty() {
        return HttpResponse::BadRequest().json(JSONResponse {
            success: false,
            error: true,
            reason: "Password cannot be empty.".to_string(),
        });
    }

    // Hash with Argon2 and store in DB
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::PasswordHasher;
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(req.as_bytes(), &salt)
        .expect("Error hashing password.");

    match database::set_setting("password", &hash.to_string(), &data.db) {
        Ok(()) => {
            info!("Password configured via API.");
            HttpResponse::Ok().json(JSONResponse {
                success: true,
                error: false,
                reason: "Password has been set.".to_string(),
            })
        }
        Err(()) => HttpResponse::InternalServerError().json(JSONResponse {
            success: false,
            error: true,
            reason: "Failed to store password.".to_string(),
        }),
    }
}

// Handle logout
// There's no reason to be calling this route with an API key
#[delete("/api/logout")]
pub async fn logout(session: Session) -> HttpResponse {
    if session.remove("umamurl-auth").is_some() {
        info!("Successful logout.");
        HttpResponse::Ok().body("Logged out!")
    } else {
        HttpResponse::Unauthorized().body("You don't seem to be logged in.")
    }
}

// Delete a given shortlink
#[delete("/api/del/{shortlink}")]
pub async fn delete_link(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = auth::is_api_ok(http, config);
    // If success, delete shortlink
    if result.success {
        match utils::delete_link(&shortlink, &data.db, data.config.allow_capital_letters) {
            Ok(()) => {
                let response = JSONResponse {
                    success: true,
                    error: false,
                    reason: format!("Deleted {shortlink}"),
                };
                HttpResponse::Ok().json(response)
            }
            Err(ServerError) => {
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when deleting the link.".to_string(),
                };
                HttpResponse::InternalServerError().json(response)
            }
            Err(ClientError { reason }) => {
                let response = JSONResponse {
                    success: false,
                    error: true,
                    reason,
                };
                HttpResponse::NotFound().json(response)
            }
        }
    } else if result.error {
        HttpResponse::Unauthorized().json(result)
    // If using password - keeps backwards compatibility
    } else if auth::is_session_valid(session, config, &data.db) {
        if utils::delete_link(&shortlink, &data.db, data.config.allow_capital_letters).is_ok() {
            HttpResponse::Ok().body(format!("Deleted {shortlink}"))
        } else {
            HttpResponse::NotFound().body("Not found!")
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

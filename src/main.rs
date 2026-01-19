use axum::{
    Router,
    routing::{get, post},
};
use backend::{
    establish_connection,
    handlers::approval_handler::{
        add_comment, approve_request, create_approval, get_approval, get_logs, list_approvals,
        reject_request,
    },
};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // 1. 환경 변수 로드 (.env)
    dotenv().ok();

    // 2. 데이터베이스 연결 설정
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = establish_connection(&database_url).await;

    println!("✅ Connection to Database successful!");

    // 3. Router 설정
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    // Protected Routes (Require Auth)
    let protected_routes = Router::new()
        // Org Routes
        .route(
            "/org/my-manager/{user_id}",
            get(backend::handlers::org_handler::get_my_manager),
        )
        .route(
            "/org/users",
            get(backend::handlers::org_handler::list_users),
        )
        // Approval Routes
        .route("/approvals", post(create_approval).get(list_approvals))
        .route("/approvals/{id}", get(get_approval))
        .route("/approvals/{id}/approve", post(approve_request))
        .route("/approvals/{id}/reject", post(reject_request))
        .route("/approvals/{id}/comments", post(add_comment))
        .route("/approvals/{id}/logs", get(get_logs))
        // Template Routes
        .route(
            "/templates",
            post(backend::handlers::template_handler::create_template)
                .get(backend::handlers::template_handler::list_templates),
        )
        .route(
            "/templates/{id}",
            get(backend::handlers::template_handler::get_template),
        )
        .route(
            "/approvals/from-template/{template_id}",
            post(backend::handlers::template_handler::create_approval_from_template),
        )
        .layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            backend::utils::middleware::auth_middleware,
        ));

    let app = Router::new()
        .route("/", get(root))
        // Auth Routes (Public)
        .route(
            "/auth/register",
            post(backend::handlers::auth_handler::register),
        )
        .route("/auth/login", post(backend::handlers::auth_handler::login))
        // Merge Protected Routes
        .merge(protected_routes)
        .layer(cors)
        .with_state(pool);

    // 4. 서버 시작
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001)); // Listen on all interfaces
    println!("🚀 Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Routes initialized.");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, pxm Engine!"
}

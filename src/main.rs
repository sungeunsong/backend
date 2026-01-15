use axum::{
    Router,
    routing::{get, post},
};
use backend::{
    establish_connection,
    handlers::approval_handler::{create_approval, get_approval},
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
    let app = Router::new()
        .route("/", get(root))
        .route("/approvals", post(create_approval))
        .route("/approvals/{id}", get(get_approval))
        .with_state(pool);

    // 4. 서버 시작
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, pxm Engine!"
}

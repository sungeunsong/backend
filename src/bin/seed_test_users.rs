use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database.");

    // Common Password Hash for 'password123'
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(b"password123", &salt)
        .expect("Hash fail")
        .to_string();

    // 1. Kim Manager
    let kim_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, full_name, position, department_id, status)
        VALUES ($1, $2, $3, $4, $5, '33333333-3333-3333-3333-333333333333', 'ACTIVE')
        ON CONFLICT (email) DO NOTHING
        "#,
        kim_id,
        "kim@pxm.com",
        password_hash,
        "Kim Manager",
        "Manager"
    )
    .execute(&pool)
    .await?;
    println!("Seeded Kim Manager");

    // 2. Park Director
    let park_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, full_name, position, department_id, status)
        VALUES ($1, $2, $3, $4, $5, '11111111-1111-1111-1111-111111111111', 'ACTIVE')
        ON CONFLICT (email) DO NOTHING
        "#,
        park_id,
        "park@pxm.com",
        password_hash,
        "Park Director",
        "Director"
    )
    .execute(&pool)
    .await?;
    println!("Seeded Park Director");

    Ok(())
}

use backend::domain::approval::FlowProcess;
use backend::establish_connection;
use backend::repositories::approval_repository::ApprovalRepository;
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn test_create_and_find_approval() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = establish_connection(&database_url).await;
    let repo = ApprovalRepository::new(pool);

    // 1. 데이터 준비
    let requester_id = Uuid::new_v4();
    let title = "Integration Test Approval".to_string();
    let form_data = serde_json::json!({ "amount": 1000, "reason": "Test" });
    let flow_process = FlowProcess {
        current_step: 1,
        steps: vec![],
    };

    // 2. 생성 (Create)
    let created = repo
        .create(
            title.clone(),
            requester_id,
            form_data.clone(),
            flow_process.clone(),
        )
        .await
        .expect("Failed to create approval request");

    assert_eq!(created.title, title);
    assert_eq!(created.requester_id, requester_id);

    // 3. 조회 (Find)
    let found = repo
        .find_by_id(created.id)
        .await
        .expect("Failed to find approval request");

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);

    // JSON 매핑 확인 (Optionally check inner values)
    println!(
        "✅ Test Passed: Created and Found approval request {}",
        found.id
    );
}

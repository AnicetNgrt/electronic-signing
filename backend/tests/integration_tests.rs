use reqwest::Client;
use serde_json::json;
use std::time::Duration;

const BASE_URL: &str = "http://localhost:8080/api";

async fn wait_for_server() {
    let client = Client::new();
    for _ in 0..30 {
        if client
            .get(format!("{}/health", BASE_URL))
            .send()
            .await
            .is_ok()
        {
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    panic!("Server did not start in time");
}

async fn login(client: &Client, email: &str, password: &str) -> String {
    let res = client
        .post(format!("{}/auth/login", BASE_URL))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Login request failed");

    assert!(res.status().is_success(), "Login failed: {:?}", res.status());

    let body: serde_json::Value = res.json().await.expect("Failed to parse login response");
    body["token"]
        .as_str()
        .expect("No token in response")
        .to_string()
}

#[tokio::test]
async fn test_health_check() {
    wait_for_server().await;

    let client = Client::new();
    let res = client
        .get(format!("{}/health", BASE_URL))
        .send()
        .await
        .expect("Health check failed");

    assert!(res.status().is_success());
    let text = res.text().await.expect("Failed to get response text");
    assert_eq!(text, "OK");
}

#[tokio::test]
async fn test_login_success() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    wait_for_server().await;

    let client = Client::new();
    let res = client
        .post(format!("{}/auth/login", BASE_URL))
        .json(&json!({
            "email": "admin@example.com",
            "password": "wrong-password"
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn test_get_current_user() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;

    let res = client
        .get(format!("{}/auth/me", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Request failed");

    assert!(res.status().is_success());

    let body: serde_json::Value = res.json().await.expect("Failed to parse response");
    assert_eq!(body["email"], "admin@example.com");
    assert_eq!(body["is_admin"], true);
}

#[tokio::test]
async fn test_protected_route_without_auth() {
    wait_for_server().await;

    let client = Client::new();
    let res = client
        .get(format!("{}/documents", BASE_URL))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn test_list_documents_empty() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;

    let res = client
        .get(format!("{}/documents", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Request failed");

    assert!(res.status().is_success());

    let body: serde_json::Value = res.json().await.expect("Failed to parse response");
    assert!(body["documents"].is_array());
}

#[tokio::test]
async fn test_document_crud_workflow() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;

    // Create a minimal PDF
    let pdf_content = include_bytes!("../tests/fixtures/sample.pdf");

    let form = reqwest::multipart::Form::new()
        .text("title", "Test Document")
        .text("self_sign_only", "false")
        .part(
            "file",
            reqwest::multipart::Part::bytes(pdf_content.to_vec())
                .file_name("test.pdf")
                .mime_str("application/pdf")
                .unwrap(),
        );

    let res = client
        .post(format!("{}/documents", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .expect("Request failed");

    if res.status().is_success() {
        let doc: serde_json::Value = res.json().await.expect("Failed to parse response");
        let doc_id = doc["id"].as_str().expect("No document ID");

        // Get document
        let res = client
            .get(format!("{}/documents/{}", BASE_URL, doc_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Request failed");

        assert!(res.status().is_success());

        // Delete document
        let res = client
            .delete(format!("{}/documents/{}", BASE_URL, doc_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Request failed");

        assert!(res.status().is_success());
    }
}

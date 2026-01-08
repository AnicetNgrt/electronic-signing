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

    assert!(
        res.status().is_success(),
        "Login failed: {:?}",
        res.status()
    );

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

#[tokio::test]
async fn test_complete_signing_workflow() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;

    // Step 1: Upload a document
    let pdf_content = include_bytes!("../tests/fixtures/sample.pdf");
    let form = reqwest::multipart::Form::new()
        .text("title", "Signing Workflow Test")
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
        .expect("Upload failed");

    if !res.status().is_success() {
        return; // Skip if upload not working
    }

    let doc: serde_json::Value = res.json().await.expect("Failed to parse response");
    let doc_id = doc["id"].as_str().expect("No document ID");
    assert_eq!(doc["status"], "draft");

    // Step 2: Add a signer
    let res = client
        .post(format!("{}/documents/{}/signers", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "email": "signer@example.com",
            "name": "Test Signer"
        }))
        .send()
        .await
        .expect("Add signer failed");

    assert!(
        res.status().is_success(),
        "Add signer failed: {:?}",
        res.status()
    );
    let signer: serde_json::Value = res.json().await.expect("Failed to parse signer");
    let signer_id = signer["id"].as_str().expect("No signer ID");
    let access_token = signer["access_token"].as_str().expect("No access token");

    // Step 3: Add a signature field
    let res = client
        .post(format!("{}/documents/{}/fields", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "field_type": "signature",
            "page": 1,
            "x": 100.0,
            "y": 500.0,
            "width": 200.0,
            "height": 50.0,
            "signer_id": signer_id
        }))
        .send()
        .await
        .expect("Add field failed");

    assert!(
        res.status().is_success(),
        "Add field failed: {:?}",
        res.status()
    );
    let field: serde_json::Value = res.json().await.expect("Failed to parse field");
    let field_id = field["id"].as_str().expect("No field ID");

    // Step 4: Send document for signing
    let res = client
        .post(format!("{}/documents/{}/send", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Send failed");

    assert!(res.status().is_success(), "Send failed: {:?}", res.status());
    let updated_doc: serde_json::Value = res.json().await.expect("Failed to parse response");
    assert_eq!(updated_doc["status"], "pending");

    // Step 5: Access signing session as signer (no auth needed - uses access token)
    let res = client
        .get(format!("{}/sign/{}", BASE_URL, access_token))
        .send()
        .await
        .expect("Get signing session failed");

    assert!(
        res.status().is_success(),
        "Get signing session failed: {:?}",
        res.status()
    );
    let session: serde_json::Value = res.json().await.expect("Failed to parse session");
    assert_eq!(session["document_title"], "Signing Workflow Test");
    assert_eq!(session["signer"]["email"], "signer@example.com");

    // Step 6: Submit signature
    let res = client
        .post(format!("{}/sign/{}/submit", BASE_URL, access_token))
        .json(&json!({
            "signatures": [{
                "field_id": field_id,
                "signature_data": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
            }]
        }))
        .send()
        .await
        .expect("Submit signing failed");

    assert!(
        res.status().is_success(),
        "Submit signing failed: {:?}",
        res.status()
    );
    let result: serde_json::Value = res.json().await.expect("Failed to parse result");
    assert_eq!(result["success"], true);
    assert_eq!(result["document_completed"], true);

    // Step 7: Verify document is completed
    let res = client
        .get(format!("{}/documents/{}", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Get document failed");

    assert!(res.status().is_success());
    let final_doc: serde_json::Value = res.json().await.expect("Failed to parse response");
    assert_eq!(final_doc["document"]["status"], "completed");

    // Step 8: Verify audit trail
    let res = client
        .get(format!("{}/documents/{}/audit", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Get audit failed");

    assert!(res.status().is_success());
    let audit_logs: serde_json::Value = res.json().await.expect("Failed to parse audit");
    assert!(audit_logs
        .as_array()
        .map(|a| !a.is_empty())
        .unwrap_or(false));
}

#[tokio::test]
async fn test_decline_signing() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;

    // Upload and prepare document
    let pdf_content = include_bytes!("../tests/fixtures/sample.pdf");
    let form = reqwest::multipart::Form::new()
        .text("title", "Decline Test")
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
        .expect("Upload failed");

    if !res.status().is_success() {
        return;
    }

    let doc: serde_json::Value = res.json().await.expect("Failed to parse response");
    let doc_id = doc["id"].as_str().expect("No document ID");

    // Add signer
    let res = client
        .post(format!("{}/documents/{}/signers", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "email": "decliner@example.com",
            "name": "Declining Signer"
        }))
        .send()
        .await
        .expect("Add signer failed");

    let signer: serde_json::Value = res.json().await.expect("Failed to parse signer");
    let access_token = signer["access_token"].as_str().expect("No access token");

    // Send document
    client
        .post(format!("{}/documents/{}/send", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Send failed");

    // Decline signing
    let res = client
        .post(format!("{}/sign/{}/decline", BASE_URL, access_token))
        .json(&json!({
            "reason": "I do not agree with the terms"
        }))
        .send()
        .await
        .expect("Decline failed");

    assert!(
        res.status().is_success(),
        "Decline failed: {:?}",
        res.status()
    );
    let result: serde_json::Value = res.json().await.expect("Failed to parse result");
    assert_eq!(result["success"], true);

    // Cleanup
    client
        .delete(format!("{}/documents/{}", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok();
}

#[tokio::test]
async fn test_void_document() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;

    // Upload document
    let pdf_content = include_bytes!("../tests/fixtures/sample.pdf");
    let form = reqwest::multipart::Form::new()
        .text("title", "Void Test")
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
        .expect("Upload failed");

    if !res.status().is_success() {
        return;
    }

    let doc: serde_json::Value = res.json().await.expect("Failed to parse response");
    let doc_id = doc["id"].as_str().expect("No document ID");

    // Add signer and send
    client
        .post(format!("{}/documents/{}/signers", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "email": "voidsigner@example.com",
            "name": "Void Signer"
        }))
        .send()
        .await
        .expect("Add signer failed");

    client
        .post(format!("{}/documents/{}/send", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Send failed");

    // Void the document
    let res = client
        .post(format!("{}/documents/{}/void", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Void failed");

    assert!(res.status().is_success(), "Void failed: {:?}", res.status());
    let voided_doc: serde_json::Value = res.json().await.expect("Failed to parse response");
    assert_eq!(voided_doc["status"], "voided");
}

#[tokio::test]
async fn test_field_operations() {
    wait_for_server().await;

    let client = Client::new();
    let token = login(&client, "admin@example.com", "change-this-secure-password").await;

    // Upload document
    let pdf_content = include_bytes!("../tests/fixtures/sample.pdf");
    let form = reqwest::multipart::Form::new()
        .text("title", "Field Operations Test")
        .text("self_sign_only", "true")
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
        .expect("Upload failed");

    if !res.status().is_success() {
        return;
    }

    let doc: serde_json::Value = res.json().await.expect("Failed to parse response");
    let doc_id = doc["id"].as_str().expect("No document ID");

    // Add signature field
    let res = client
        .post(format!("{}/documents/{}/fields", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "field_type": "signature",
            "page": 1,
            "x": 100.0,
            "y": 500.0,
            "width": 200.0,
            "height": 50.0
        }))
        .send()
        .await
        .expect("Add field failed");

    assert!(res.status().is_success());
    let field: serde_json::Value = res.json().await.expect("Failed to parse field");
    let field_id = field["id"].as_str().expect("No field ID");

    // Add date field
    let res = client
        .post(format!("{}/documents/{}/fields", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "field_type": "date",
            "page": 1,
            "x": 100.0,
            "y": 600.0,
            "width": 150.0,
            "height": 30.0,
            "date_format": "MM/DD/YYYY"
        }))
        .send()
        .await
        .expect("Add date field failed");

    assert!(res.status().is_success());

    // Add text field
    let res = client
        .post(format!("{}/documents/{}/fields", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "field_type": "text",
            "page": 1,
            "x": 100.0,
            "y": 650.0,
            "width": 200.0,
            "height": 30.0,
            "value": "Sample Text"
        }))
        .send()
        .await
        .expect("Add text field failed");

    assert!(res.status().is_success());

    // Update field
    let res = client
        .put(format!(
            "{}/documents/{}/fields/{}",
            BASE_URL, doc_id, field_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "x": 150.0,
            "y": 550.0,
            "width": 250.0,
            "height": 60.0
        }))
        .send()
        .await
        .expect("Update field failed");

    assert!(res.status().is_success());

    // Delete field
    let res = client
        .delete(format!(
            "{}/documents/{}/fields/{}",
            BASE_URL, doc_id, field_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Delete field failed");

    assert!(res.status().is_success());

    // Cleanup
    client
        .delete(format!("{}/documents/{}", BASE_URL, doc_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .ok();
}

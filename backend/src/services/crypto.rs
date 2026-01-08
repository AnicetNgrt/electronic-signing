use sha2::{Digest, Sha256};
use std::io::Read;
use uuid::Uuid;

pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn hash_string(data: &str) -> String {
    hash_data(data.as_bytes())
}

pub fn hash_file<R: Read>(mut reader: R) -> anyhow::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

pub fn generate_access_token() -> String {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    format!("{}{}", uuid1.simple(), uuid2.simple())
}

pub fn compute_audit_hash(
    document_id: &Uuid,
    action: &str,
    timestamp: &str,
    previous_hash: Option<&str>,
    details: Option<&str>,
) -> String {
    let mut data = format!("{}:{}:{}", document_id, action, timestamp);

    if let Some(prev) = previous_hash {
        data.push_str(&format!(":{}", prev));
    }

    if let Some(det) = details {
        data.push_str(&format!(":{}", det));
    }

    hash_string(&data)
}

pub fn compute_certificate_hash(
    document_id: &Uuid,
    document_hash: &str,
    signers_data: &str,
    audit_data: &str,
    generated_at: &str,
) -> String {
    let data = format!(
        "CERT:{}:{}:{}:{}:{}",
        document_id, document_hash, signers_data, audit_data, generated_at
    );
    hash_string(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        let hash = hash_string("hello world");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_generate_access_token() {
        let token = generate_access_token();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_audit_hash_consistency() {
        let doc_id = Uuid::new_v4();
        let hash1 = compute_audit_hash(&doc_id, "created", "2024-01-01T00:00:00Z", None, None);
        let hash2 = compute_audit_hash(&doc_id, "created", "2024-01-01T00:00:00Z", None, None);
        assert_eq!(hash1, hash2);
    }
}

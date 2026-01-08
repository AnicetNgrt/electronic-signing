use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::document::{
    AddFieldRequest, Document, DocumentFieldRow, DocumentStatus, UpdateFieldRequest,
};

pub async fn create_document(
    pool: &PgPool,
    owner_id: Uuid,
    title: &str,
    original_filename: &str,
    file_path: &str,
    file_hash: &str,
    self_sign_only: bool,
) -> Result<Document> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        INSERT INTO documents (owner_id, title, original_filename, file_path, file_hash, self_sign_only)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, owner_id, title, original_filename, file_path, file_hash, status,
                  self_sign_only, total_signers, completed_signers, expires_at, completed_at,
                  created_at, updated_at
        "#,
    )
    .bind(owner_id)
    .bind(title)
    .bind(original_filename)
    .bind(file_path)
    .bind(file_hash)
    .bind(self_sign_only)
    .fetch_one(pool)
    .await?;

    Ok(doc)
}

pub async fn get_document_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Document>> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, owner_id, title, original_filename, file_path, file_hash, status,
               self_sign_only, total_signers, completed_signers, expires_at, completed_at,
               created_at, updated_at
        FROM documents
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(doc)
}

pub async fn get_documents_by_owner(
    pool: &PgPool,
    owner_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Document>> {
    let docs = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, owner_id, title, original_filename, file_path, file_hash, status,
               self_sign_only, total_signers, completed_signers, expires_at, completed_at,
               created_at, updated_at
        FROM documents
        WHERE owner_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(owner_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(docs)
}

pub async fn update_document_status(
    pool: &PgPool,
    id: Uuid,
    status: DocumentStatus,
) -> Result<Document> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        UPDATE documents
        SET status = $1
        RETURNING id, owner_id, title, original_filename, file_path, file_hash, status,
                  self_sign_only, total_signers, completed_signers, expires_at, completed_at,
                  created_at, updated_at
        "#,
    )
    .bind(status)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(doc)
}

pub async fn update_document_title(pool: &PgPool, id: Uuid, title: &str) -> Result<Document> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        UPDATE documents
        SET title = $1
        WHERE id = $2
        RETURNING id, owner_id, title, original_filename, file_path, file_hash, status,
                  self_sign_only, total_signers, completed_signers, expires_at, completed_at,
                  created_at, updated_at
        "#,
    )
    .bind(title)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(doc)
}

pub async fn mark_document_completed(pool: &PgPool, id: Uuid) -> Result<Document> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        UPDATE documents
        SET status = 'completed', completed_at = NOW()
        WHERE id = $1
        RETURNING id, owner_id, title, original_filename, file_path, file_hash, status,
                  self_sign_only, total_signers, completed_signers, expires_at, completed_at,
                  created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(doc)
}

pub async fn increment_completed_signers(pool: &PgPool, id: Uuid) -> Result<Document> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        UPDATE documents
        SET completed_signers = completed_signers + 1
        WHERE id = $1
        RETURNING id, owner_id, title, original_filename, file_path, file_hash, status,
                  self_sign_only, total_signers, completed_signers, expires_at, completed_at,
                  created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(doc)
}

pub async fn update_total_signers(pool: &PgPool, id: Uuid, total: i32) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE documents
        SET total_signers = $1
        WHERE id = $2
        "#,
    )
    .bind(total)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_document(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn add_field(
    pool: &PgPool,
    document_id: Uuid,
    req: &AddFieldRequest,
) -> Result<DocumentFieldRow> {
    let field = sqlx::query_as::<_, DocumentFieldRow>(
        r#"
        INSERT INTO document_fields (document_id, field_type, page, x, y, width, height,
                                     signer_id, value, font_size, font_family, date_format)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, document_id, field_type, page, x, y, width, height, signer_id,
                  value, font_size, font_family, date_format, created_at, updated_at
        "#,
    )
    .bind(document_id)
    .bind(&req.field_type)
    .bind(req.page)
    .bind(req.x)
    .bind(req.y)
    .bind(req.width)
    .bind(req.height)
    .bind(req.signer_id)
    .bind(&req.value)
    .bind(req.font_size.unwrap_or(12))
    .bind(req.font_family.as_deref().unwrap_or("Arial"))
    .bind(req.date_format.as_deref().unwrap_or("YYYY-MM-DD"))
    .fetch_one(pool)
    .await?;

    Ok(field)
}

pub async fn get_fields_by_document(
    pool: &PgPool,
    document_id: Uuid,
) -> Result<Vec<DocumentFieldRow>> {
    let fields = sqlx::query_as::<_, DocumentFieldRow>(
        r#"
        SELECT id, document_id, field_type, page, x, y, width, height, signer_id,
               value, font_size, font_family, date_format, created_at, updated_at
        FROM document_fields
        WHERE document_id = $1
        ORDER BY page, y, x
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    Ok(fields)
}

pub async fn get_field_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DocumentFieldRow>> {
    let field = sqlx::query_as::<_, DocumentFieldRow>(
        r#"
        SELECT id, document_id, field_type, page, x, y, width, height, signer_id,
               value, font_size, font_family, date_format, created_at, updated_at
        FROM document_fields
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(field)
}

pub async fn update_field(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateFieldRequest,
) -> Result<DocumentFieldRow> {
    let field = sqlx::query_as::<_, DocumentFieldRow>(
        r#"
        UPDATE document_fields
        SET x = COALESCE($1, x),
            y = COALESCE($2, y),
            width = COALESCE($3, width),
            height = COALESCE($4, height),
            value = COALESCE($5, value),
            font_size = COALESCE($6, font_size),
            font_family = COALESCE($7, font_family),
            date_format = COALESCE($8, date_format)
        WHERE id = $9
        RETURNING id, document_id, field_type, page, x, y, width, height, signer_id,
                  value, font_size, font_family, date_format, created_at, updated_at
        "#,
    )
    .bind(req.x)
    .bind(req.y)
    .bind(req.width)
    .bind(req.height)
    .bind(&req.value)
    .bind(req.font_size)
    .bind(&req.font_family)
    .bind(&req.date_format)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(field)
}

pub async fn update_field_value(pool: &PgPool, id: Uuid, value: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE document_fields
        SET value = $1
        WHERE id = $2
        "#,
    )
    .bind(value)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_field(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM document_fields WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn count_documents_by_owner(pool: &PgPool, owner_id: Uuid) -> Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM documents WHERE owner_id = $1")
        .bind(owner_id)
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}

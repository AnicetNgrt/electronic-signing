use anyhow::Result;
use lopdf::{Document, Object};
use std::fs;
use std::path::Path;

pub fn validate_pdf(path: &Path) -> Result<()> {
    let doc = Document::load(path)?;

    if doc.get_pages().is_empty() {
        return Err(anyhow::anyhow!("PDF has no pages"));
    }

    Ok(())
}

pub fn get_page_count(path: &Path) -> Result<usize> {
    let doc = Document::load(path)?;
    Ok(doc.get_pages().len())
}

pub fn get_pdf_metadata(path: &Path) -> Result<PdfMetadata> {
    let doc = Document::load(path)?;
    let pages = doc.get_pages();
    let page_count = pages.len();

    let mut page_sizes = Vec::new();

    for (_page_num, page_id) in pages {
        if let Ok(Object::Dictionary(page_dict)) = doc.get_object(page_id) {
            let media_box = get_media_box(&doc, page_dict);
            page_sizes.push(PageSize {
                width: media_box.2 - media_box.0,
                height: media_box.3 - media_box.1,
            });
        }
    }

    Ok(PdfMetadata {
        page_count,
        page_sizes,
    })
}

fn get_media_box(doc: &Document, page_dict: &lopdf::Dictionary) -> (f64, f64, f64, f64) {
    if let Ok(Object::Array(arr)) = page_dict.get(b"MediaBox") {
        if arr.len() >= 4 {
            let x1 = get_number(&arr[0]).unwrap_or(0.0);
            let y1 = get_number(&arr[1]).unwrap_or(0.0);
            let x2 = get_number(&arr[2]).unwrap_or(612.0);
            let y2 = get_number(&arr[3]).unwrap_or(792.0);
            return (x1, y1, x2, y2);
        }
    }

    if let Ok(Object::Reference(parent_ref)) = page_dict.get(b"Parent") {
        if let Ok(Object::Dictionary(parent_dict)) = doc.get_object(*parent_ref) {
            if let Ok(Object::Array(arr)) = parent_dict.get(b"MediaBox") {
                if arr.len() >= 4 {
                    let x1 = get_number(&arr[0]).unwrap_or(0.0);
                    let y1 = get_number(&arr[1]).unwrap_or(0.0);
                    let x2 = get_number(&arr[2]).unwrap_or(612.0);
                    let y2 = get_number(&arr[3]).unwrap_or(792.0);
                    return (x1, y1, x2, y2);
                }
            }
        }
    }

    (0.0, 0.0, 612.0, 792.0)
}

fn get_number(obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(n) => Some(*n as f64),
        Object::Real(n) => Some(f64::from(*n)),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct PdfMetadata {
    pub page_count: usize,
    pub page_sizes: Vec<PageSize>,
}

#[derive(Debug, Clone)]
pub struct PageSize {
    pub width: f64,
    pub height: f64,
}

pub fn copy_file(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}

pub fn delete_file(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

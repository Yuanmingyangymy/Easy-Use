use crate::errors::AppError;

pub fn ensure_upload_size(current_bytes: u64, next_chunk_bytes: usize, max_bytes: u64) -> Result<u64, AppError> {
    let next_total = current_bytes.saturating_add(next_chunk_bytes as u64);
    if next_total > max_bytes {
        return Err(AppError::FileTooLarge { max_bytes });
    }
    Ok(next_total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_uploads_within_limit() {
        assert_eq!(ensure_upload_size(90, 10, 100).expect("size ok"), 100);
    }

    #[test]
    fn rejects_uploads_over_limit() {
        assert!(matches!(
            ensure_upload_size(90, 11, 100),
            Err(AppError::FileTooLarge { max_bytes: 100 })
        ));
    }
}

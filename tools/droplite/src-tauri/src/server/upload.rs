use crate::errors::AppError;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

pub fn ensure_upload_size(
    current_bytes: u64,
    next_chunk_bytes: usize,
    max_bytes: u64,
) -> Result<u64, AppError> {
    let next_total = current_bytes.saturating_add(next_chunk_bytes as u64);
    if next_total > max_bytes {
        return Err(AppError::FileTooLarge { max_bytes });
    }
    Ok(next_total)
}

pub struct LimitedFileWriter {
    path: PathBuf,
    file: fs::File,
    bytes_written: u64,
}

impl LimitedFileWriter {
    pub async fn create(path: PathBuf) -> Result<Self, AppError> {
        let file = fs::File::create(&path).await?;
        Ok(Self {
            path,
            file,
            bytes_written: 0,
        })
    }

    pub async fn write_chunk(&mut self, chunk: &[u8], max_bytes: u64) -> Result<(), AppError> {
        self.bytes_written = ensure_upload_size(self.bytes_written, chunk.len(), max_bytes)?;
        self.file.write_all(chunk).await?;
        Ok(())
    }

    pub async fn finish(mut self) -> Result<u64, AppError> {
        if let Err(error) = self.file.flush().await {
            self.abort().await;
            return Err(AppError::Io(error));
        }

        if let Err(error) = self.file.sync_data().await {
            self.abort().await;
            return Err(AppError::Io(error));
        }

        Ok(self.bytes_written)
    }

    pub async fn abort(self) {
        drop(self.file);
        let _ = fs::remove_file(self.path).await;
    }
}

pub async fn rename_complete_upload(temp_path: &Path, target_path: &Path) -> Result<(), AppError> {
    match fs::rename(temp_path, target_path).await {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(temp_path).await;
            Err(AppError::Io(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs as std_fs, path::PathBuf};

    fn test_file(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "droplite-upload-{name}-{}.part",
            crate::server::now_epoch_secs()
        ))
    }

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

    #[tokio::test]
    async fn oversized_limited_writer_can_remove_partial_file() {
        let path = test_file("oversized");
        let mut writer = LimitedFileWriter::create(path.clone())
            .await
            .expect("create writer");

        writer.write_chunk(b"12345", 5).await.expect("first chunk");
        let result = writer.write_chunk(b"6", 5).await;
        assert!(matches!(
            result,
            Err(AppError::FileTooLarge { max_bytes: 5 })
        ));

        writer.abort().await;
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn finished_limited_writer_preserves_complete_size() {
        let path = test_file("complete");
        let mut writer = LimitedFileWriter::create(path.clone())
            .await
            .expect("create writer");

        writer.write_chunk(b"123", 10).await.expect("first chunk");
        writer.write_chunk(b"4567", 10).await.expect("second chunk");
        let size = writer.finish().await.expect("finish");

        assert_eq!(size, 7);
        assert_eq!(std_fs::metadata(&path).expect("metadata").len(), 7);
        let _ = std_fs::remove_file(path);
    }
}

use crate::errors::AppError;
use crate::server::debug_log;
use std::{
    fs as std_fs,
    path::{Path, PathBuf},
};
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

pub async fn ensure_receive_dir(path: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path).await?;
    Ok(())
}

pub struct TempFileGuard {
    path: PathBuf,
    keep: bool,
}

impl TempFileGuard {
    pub fn new(path: PathBuf) -> Self {
        Self { path, keep: false }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn keep(&mut self) {
        self.keep = true;
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if !self.keep {
            if std_fs::remove_file(&self.path).is_ok() {
                debug_log("cleanup performed for temporary upload file");
            }
        }
    }
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

    #[tokio::test]
    async fn rename_complete_upload_moves_part_to_final_file() {
        let temp_path = test_file("rename");
        let final_path = temp_path.with_extension("jpg");
        std_fs::write(&temp_path, b"image bytes").expect("write temp");

        rename_complete_upload(&temp_path, &final_path)
            .await
            .expect("rename");

        assert!(final_path.exists());
        assert!(!temp_path.exists());
        assert_eq!(
            std_fs::read(&final_path).expect("read final"),
            b"image bytes"
        );
        let _ = std_fs::remove_file(final_path);
    }

    #[tokio::test]
    async fn rename_failure_removes_part_file() {
        let temp_path = test_file("rename-fail");
        let missing_parent = std::env::temp_dir().join("droplite-missing-parent-for-rename");
        let final_path = missing_parent.join("final.jpg");
        let _ = std_fs::remove_dir_all(&missing_parent);
        std_fs::write(&temp_path, b"image bytes").expect("write temp");

        let result = rename_complete_upload(&temp_path, &final_path).await;

        assert!(result.is_err());
        assert!(!temp_path.exists());
        assert!(!final_path.exists());
    }

    #[tokio::test]
    async fn ensure_receive_dir_creates_missing_directory_for_first_upload() {
        let dir = std::env::temp_dir().join(format!(
            "droplite-missing-receive-dir-{}",
            crate::server::now_epoch_secs()
        ));
        let _ = std_fs::remove_dir_all(&dir);

        ensure_receive_dir(&dir).await.expect("create receive dir");

        assert!(dir.exists());
        let _ = std_fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn temp_file_guard_removes_uncommitted_file() {
        let path = test_file("guard");
        std_fs::write(&path, b"partial").expect("write temp");

        {
            let _guard = TempFileGuard::new(path.clone());
        }

        assert!(!path.exists());
    }
}

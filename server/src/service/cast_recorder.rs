use common::error::{CmdbError, CmdbResult};
use serde_json::json;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

/// asciinema cast v2 recorder
pub struct CastRecorderInner;

pub type CastRecorder = Arc<CastRecorderInner>;

impl CastRecorderInner {
    pub fn cast_dir() -> PathBuf {
        let dir =
            std::env::var("CMDB_CAST_DIR").unwrap_or_else(|_| "/var/log/cmdb/casts/".to_string());
        PathBuf::from(&dir)
    }

    pub fn ensure_cast_dir() -> CmdbResult<()> {
        let dir = Self::cast_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| CmdbError::Internal(format!("Failed to create cast dir: {}", e)))?;
        Ok(())
    }

    pub fn cast_path(session_id: &str) -> PathBuf {
        let dir = Self::cast_dir();
        dir.join(format!("{}.cast", session_id))
    }

    pub fn init_cast(session_id: &str, width: u64, height: u64) -> CmdbResult<()> {
        Self::ensure_cast_dir()?;
        let path = Self::cast_path(session_id);
        let file = fs::File::create(&path)
            .map_err(|e| CmdbError::Internal(format!("Failed to create cast file: {}", e)))?;
        let mut writer = BufWriter::new(file);

        let header = json!({
            "version": 2,
            "width": width,
            "height": height,
            "timestamp": chrono::Utc::now().timestamp(),
            "env": {
                "SHELL": "/bin/bash",
                "TERM": "xterm-256color"
            }
        });
        writeln!(writer, "{}", header)
            .map_err(|e| CmdbError::Internal(format!("Failed to write cast header: {}", e)))?;
        writer
            .flush()
            .map_err(|e| CmdbError::Internal(format!("Failed to flush cast file: {}", e)))?;
        Ok(())
    }

    pub fn append_frame(session_id: &str, elapsed: f64, data: &str) -> CmdbResult<()> {
        let path = Self::cast_path(session_id);
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .map_err(|e| CmdbError::Internal(format!("Failed to open cast file: {}", e)))?;

        let frame = json!([elapsed, "o", data]);
        writeln!(file, "{}", frame)
            .map_err(|e| CmdbError::Internal(format!("Failed to write cast frame: {}", e)))?;
        file.flush()
            .map_err(|e| CmdbError::Internal(format!("Failed to flush cast file: {}", e)))?;
        Ok(())
    }

    pub fn cast_file_exists(session_id: &str) -> bool {
        Self::cast_path(session_id).exists()
    }

    pub fn read_cast_file(session_id: &str) -> CmdbResult<Vec<u8>> {
        let path = Self::cast_path(session_id);
        fs::read(&path).map_err(|e| CmdbError::Internal(format!("Failed to read cast file: {}", e)))
    }

    pub fn read_cast_file_range(session_id: &str, offset: u64, length: u64) -> CmdbResult<Vec<u8>> {
        let path = Self::cast_path(session_id);
        let file = fs::File::open(&path)
            .map_err(|e| CmdbError::Internal(format!("Failed to open cast file: {}", e)))?;
        use std::io::{Read, Seek, SeekFrom};
        let mut reader = std::io::BufReader::new(file);
        reader
            .seek(SeekFrom::Start(offset))
            .map_err(|e| CmdbError::Internal(format!("Failed to seek cast file: {}", e)))?;
        let mut buf = vec![0u8; length as usize];
        let n = reader
            .read(&mut buf)
            .map_err(|e| CmdbError::Internal(format!("Failed to read cast file: {}", e)))?;
        buf.truncate(n);
        Ok(buf)
    }

    pub fn cast_file_size(session_id: &str) -> CmdbResult<u64> {
        let path = Self::cast_path(session_id);
        fs::metadata(&path)
            .map(|m| m.len())
            .map_err(|e| CmdbError::Internal(format!("Failed to get cast file size: {}", e)))
    }

    pub fn delete_cast(session_id: &str) -> CmdbResult<()> {
        match fs::remove_file(Self::cast_path(session_id)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CmdbError::Internal(format!(
                "Failed to delete cast file: {}",
                error
            ))),
        }
    }

    #[allow(dead_code)]
    pub fn delete_old_casts(retention_days: u64) -> CmdbResult<usize> {
        let dir = Self::cast_dir();
        if !dir.exists() {
            return Ok(0);
        }
        let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut deleted = 0;
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified: chrono::DateTime<chrono::Utc> = modified.into();
                        if modified < cutoff {
                            if fs::remove_file(entry.path()).is_ok() {
                                deleted += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(deleted)
    }
}

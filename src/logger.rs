use std::fs::OpenOptions;
use std::io::{Read, Write, Seek, SeekFrom};
use chrono::Local;
use crate::error::AppError;

#[derive(Clone)]
pub struct Logger {
    file_path: String,
}

impl Logger {
    pub fn new(file_path: &str) -> Result<Self, AppError> {
        // Ensure the directory exists
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError::FileError(format!("Failed to create directory: {}", e)))?;
            }
        }
        
        // Open the file for appending
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(file_path)
            .map_err(|e| AppError::FileError(format!("Failed to open log file: {}", e)))?;
            
        // Write a header to the log file
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let header = format!("[{}] [INFO] Log file opened\n", timestamp);
        
        // If the file is empty, write the header
        if file.metadata().map(|m| m.len() == 0).unwrap_or(true) {
            file.write_all(header.as_bytes())
                .map_err(|e| AppError::FileError(format!("Failed to write to log file: {}", e)))?;
        }
            
        Ok(Self {
            file_path: file_path.to_string(),
        })
    }

    pub fn update_file_path(&mut self, new_path: &str) -> Result<(), AppError> {
        self.file_path = new_path.to_string();
        
        Ok(())
    }

    pub fn info(&mut self, message: &str) -> Result<(), AppError> {
        self.log("INFO", message)
    }

    pub fn warn(&mut self, message: &str) -> Result<(), AppError> {
        self.log("WARN", message)
    }

    pub fn error(&mut self, message: &str) -> Result<(), AppError> {
        self.log("ERROR", message)
    }

    fn log(&mut self, level: &str, message: &str) -> Result<(), AppError> {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);

        // Open the file for reading and writing
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.file_path)
            .map_err(|e| AppError::FileError(format!("Failed to open log file: {}", e)))?;
        
        // Read the existing content
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| AppError::FileError(format!("Failed to read log file: {}", e)))?;
        
        // Create new content with the new log line at the top
        let new_content = format!("{}{}", log_line, content);
        
        // Clear the file and write the new content
        file.set_len(0)
            .map_err(|e| AppError::FileError(format!("Failed to clear log file: {}", e)))?;
        
        file.seek(SeekFrom::Start(0))
            .map_err(|e| AppError::FileError(format!("Failed to seek to beginning of log file: {}", e)))?;
        
        file.write_all(new_content.as_bytes())
            .map_err(|e| AppError::FileError(format!("Failed to write to log file: {}", e)))?;

        Ok(())
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }
}


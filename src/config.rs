use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use crate::error::AppError;

/// Default configuration file path
const CONFIG_FILE: &str = "config.json";

/// Default filenames
const DEFAULT_LOG_FILENAME: &str = "totp_app.log";
const DEFAULT_STORAGE_FILENAME: &str = "accounts.json";

/// Application configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Directory path for the storage file
    pub storage_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_dir: ".".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from file or create default if not exists
    pub fn load() -> Result<Self, AppError> {
        if Path::new(CONFIG_FILE).exists() {
            let mut file = File::open(CONFIG_FILE)
                .map_err(|e| AppError::FileError(format!("Failed to open config file: {}", e)))?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| AppError::FileError(format!("Failed to read config file: {}", e)))?;

            if contents.is_empty() {
                return Ok(Config::default());
            }

            serde_json::from_str(&contents)
                .map_err(|e| AppError::JsonError(format!("Failed to parse config JSON: {}", e)))
        } else {
            Ok(Config::default())
        }
    }

    /// Check if the configuration is using default values
    #[allow(dead_code)]
    pub fn is_using_defaults(&self) -> bool {
        self.storage_dir == "."
    }

    /// Get the full log file path (always in the same directory as the config file)
    pub fn get_log_file_path(&self) -> String {
        DEFAULT_LOG_FILENAME.to_string()
    }

    /// Get the full storage file path
    pub fn get_storage_file_path(&self) -> String {
        if self.storage_dir == "." {
            DEFAULT_STORAGE_FILENAME.to_string()
        } else {
            format!("{}/{}", self.storage_dir, DEFAULT_STORAGE_FILENAME)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::JsonError(format!("Failed to serialize config to JSON: {}", e)))?;

        match File::create(CONFIG_FILE) {
            Ok(mut file) => {
                file.write_all(json.as_bytes())
                    .map_err(|e| AppError::FileError(format!("Failed to write to config file: {}", e)))?;
                Ok(())
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(AppError::PermissionError(format!(
                        "Permission denied when creating config file '{}'. Please run with appropriate permissions.",
                        CONFIG_FILE
                    )))
                } else {
                    Err(AppError::FileError(format!("Failed to create config file: {}", e)))
                }
            }
        }
    }

    /// Create directories for log and storage files if they don't exist
    pub fn ensure_directories(&self) -> Result<(), AppError> {
        // Ensure storage directory exists
        if self.storage_dir != "." {
            self.create_and_verify_directory(Path::new(&self.storage_dir), "storage")?;
        }

        Ok(())
    }

    /// Creates a directory and verifies that we can write to it
    fn create_and_verify_directory(&self, dir: &Path, dir_type: &str) -> Result<(), AppError> {
        // If directory doesn't exist, create it
        if !dir.exists() {
            match fs::create_dir_all(dir) {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        return Err(AppError::PermissionError(format!(
                            "Permission denied when creating {} directory '{}'. Please choose a different location or run with appropriate permissions.",
                            dir_type, dir.display()
                        )));
                    } else {
                        return Err(AppError::FileError(format!(
                            "Failed to create {} directory '{}': {}. Please verify the path is valid and you have the necessary permissions.",
                            dir_type, dir.display(), e
                        )));
                    }
                }
            }
        }

        // Verify we can write to the directory by attempting to create a test file
        let test_file = dir.join(".write_test");
        match File::create(&test_file) {
            Ok(_) => {
                // Clean up the test file
                if let Err(e) = fs::remove_file(&test_file) {
                    eprintln!("Warning: Failed to remove test file: {}", e);
                }
                Ok(())
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(AppError::PermissionError(format!(
                        "Permission denied when writing to {} directory '{}'. Please choose a different location or run with appropriate permissions.",
                        dir_type, dir.display()
                    )))
                } else {
                    Err(AppError::FileError(format!(
                        "Cannot write to {} directory '{}': {}. Please verify the path is valid and you have the necessary permissions.",
                        dir_type, dir.display(), e
                    )))
                }
            }
        }
    }

    /// Validates and normalizes directory paths
    pub fn validate_paths(&mut self) -> Result<(), AppError> {
        // Normalize storage directory path
        self.storage_dir = self.normalize_path(&self.storage_dir)?;
        
        // Validate that paths are not pointing to files
        if Path::new(&self.storage_dir).is_file() {
            return Err(AppError::InvalidInput(format!(
                "Storage directory path '{}' points to a file. Please provide a directory path.",
                self.storage_dir
            )));
        }
        
        Ok(())
    }
    
    /// Normalizes a directory path and ensures it's valid
    fn normalize_path(&self, path: &str) -> Result<String, AppError> {
        // Convert to PathBuf for manipulation
        let path_buf = PathBuf::from(path);
        
        // Check if the path is empty
        if path_buf.as_os_str().is_empty() {
            return Err(AppError::InvalidInput("Directory path cannot be empty".to_string()));
        }
        
        // If it's just a dot, return as is
        if path == "." {
            return Ok(path.to_string());
        }
        
        // For paths with directories, we'll just normalize the path
        // We'll let the OS handle any invalid paths when we try to create files
        
        // Return the normalized path as a string
        Ok(path_buf.to_string_lossy().to_string())
    }
} 
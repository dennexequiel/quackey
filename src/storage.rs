use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use crate::account::Account;
use crate::error::AppError;
use crate::logger::Logger;
use std::sync::atomic::{AtomicBool, Ordering};

/// Storage manager for TOTP accounts
pub struct Storage {
    file_path: String,
    accounts: Vec<Account>,
    logger: Option<Logger>,
}

// Static flag to track if directory creation has been logged
static DIRECTORY_CREATED: AtomicBool = AtomicBool::new(false);

impl Storage {
    pub fn new_with_logger(file_path: &str, logger: Option<Logger>) -> Result<Self, AppError> {
        let mut storage = Self {
            file_path: file_path.to_string(),
            accounts: Vec::new(),
            logger,
        };

        // Ensure the directory exists
        storage.ensure_directory()?;

        // Load existing accounts if file exists
        if Path::new(file_path).exists() {
            match storage.load() {
                Ok(_) => {},
                Err(e) => {
                    // If there's an error loading the file, log it and start with an empty accounts list
                    eprintln!("Error loading accounts: {}. Starting with empty accounts list.", e);
                    // Optionally, you could rename the corrupted file here
                    if let Err(rename_err) = std::fs::rename(file_path, format!("{}.bak", file_path)) {
                        eprintln!("Failed to backup corrupted file: {}", rename_err);
                    }
                }
            }
        }

        Ok(storage)
    }

    /// Logs a message using the logger if available
    fn log(&mut self, level: &str, message: &str) -> Result<(), AppError> {
        if let Some(logger) = &mut self.logger {
            match level {
                "INFO" => logger.info(message)?,
                "WARN" => logger.warn(message)?,
                "ERROR" => logger.error(message)?,
                _ => logger.info(message)?,
            }
        }
        Ok(())
    }

    /// Ensures the directory for the storage file exists
    fn ensure_directory(&mut self) -> Result<(), AppError> {
        let path = Path::new(&self.file_path);
        
        // If the file path has a parent directory
        if let Some(parent) = path.parent() {
            // Check if the directory exists
            if !parent.exists() {
                // Use a static flag to ensure we only log this once
                let should_log = !DIRECTORY_CREATED.load(Ordering::SeqCst);
                
                if should_log {
                    // Log that we're creating the directory
                    let message = format!("Storage directory not found. Auto-creating: {}", parent.display());
                    eprintln!("{}", message);
                    
                    // Create the directory and all parent directories
                    fs::create_dir_all(parent)
                        .map_err(|e| AppError::FileError(format!("Failed to create directory: {}", e)))?;
                    
                    // Log successful creation
                    let success_message = format!("Successfully created storage directory: {}", parent.display());
                    self.log("WARN", &message)?;
                    self.log("INFO", &success_message)?;
                    
                    // Set the flag to indicate we've logged this
                    DIRECTORY_CREATED.store(true, Ordering::SeqCst);
                } else {
                    // Just create the directory without logging
                    fs::create_dir_all(parent)
                        .map_err(|e| AppError::FileError(format!("Failed to create directory: {}", e)))?;
                }
            }
        } else {
            // No parent directory (file is in current directory)
            // Check if the file exists
            if !path.exists() {
                // Log that we're creating the file
                let message = format!("Storage file not found. Will be created: {}", path.display());
                eprintln!("{}", message);
                
                // Make sure to log this message to the log file
                if self.logger.is_some() {
                    self.log("WARN", &message)?;
                }
            }
        }
        
        Ok(())
    }

    /// Gets the current storage file path
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Updates the storage file path
    pub fn update_file_path(&mut self, new_path: &str) -> Result<(), AppError> {
        let old_path = self.file_path.clone();
        
        // Update the file path
        self.file_path = new_path.to_string();
        
        // Log the path change
        let message = format!("Storage file path changed from '{}' to '{}'", old_path, new_path);
        self.log("INFO", &message)?;
        
        // Ensure the directory exists
        self.ensure_directory()?;
        
        // Load accounts from the new file
        self.load()
    }

    pub fn add_account(&mut self, account: Account) -> Result<(), AppError> {
        // Ensure the directory exists before saving
        self.ensure_directory()?;
        
        self.accounts.push(account.clone());
        
        // Log the account addition
        let message = format!("Added new account: {}", account.name());
        self.log("INFO", &message)?;
        
        self.save()
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>, AppError> {
        Ok(self.accounts.clone())
    }

    /// Deletes an account by name
    pub fn delete_account(&mut self, name: &str) -> Result<(), AppError> {
        // Find the account by name
        let position = self.accounts.iter().position(|a| a.name() == name);
        
        match position {
            Some(index) => {
                // Remove the account at the found position
                self.accounts.remove(index);
                
                // Log the account deletion
                let message = format!("Deleted account: {}", name);
                self.log("INFO", &message)?;
                
                // Save the updated accounts list
                self.save()
            },
            None => {
                let error_message = format!("Account '{}' not found", name);
                self.log("ERROR", &error_message)?;
                Err(AppError::InvalidInput(error_message))
            }
        }
    }

    /// Updates an account's details
    pub fn update_account(&mut self, old_name: &str, new_name: String, new_issuer: Option<String>) -> Result<(), AppError> {
        // Find the account by name
        let position = self.accounts.iter().position(|a| a.name() == old_name);
        
        match position {
            Some(index) => {
                // Get a reference to the account
                let account = &mut self.accounts[index];
                
                // Create a new account with updated details but same TOTP settings
                let updated_account = Account::new(
                    new_name.clone(),
                    account.secret().to_string(),
                    account.digits(),
                    account.period(),
                    account.algorithm(),
                    new_issuer.clone(),
                );
                
                // Replace the old account with the updated one
                self.accounts[index] = updated_account;
                
                // Log the account update
                let message = format!("Updated account from '{}' to '{}'", old_name, new_name);
                self.log("INFO", &message)?;
                
                // Save the updated accounts list
                self.save()
            },
            None => {
                let error_message = format!("Account '{}' not found", old_name);
                self.log("ERROR", &error_message)?;
                Err(AppError::InvalidInput(error_message))
            }
        }
    }

    fn load(&mut self) -> Result<(), AppError> {
        // Check if the file exists
        if !Path::new(&self.file_path).exists() {
            // If the file doesn't exist, start with an empty accounts list
            self.accounts = Vec::new();
            
            // Log that we're starting with an empty accounts list
            let message = format!("Storage file '{}' not found. Starting with empty accounts list.", self.file_path);
            self.log("WARN", &message)?;
            
            return Ok(());
        }
        
        let mut file = File::open(&self.file_path)
            .map_err(|e| {
                let error_message = format!("Failed to open file: {}", e);
                self.log("ERROR", &error_message).ok();
                AppError::FileError(error_message)
            })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| {
                let error_message = format!("Failed to read file: {}", e);
                self.log("ERROR", &error_message).ok();
                AppError::FileError(error_message)
            })?;

        if contents.is_empty() {
            self.log("WARN", "Storage file is empty. Starting with empty accounts list.")?;
            return Ok(());
        }

        match serde_json::from_str(&contents) {
            Ok(accounts) => {
                self.accounts = accounts;
                let count = self.accounts.len();
                self.log("INFO", &format!("Loaded {} accounts from storage", count))?;
                Ok(())
            },
            Err(e) => {
                let error_message = format!("Failed to parse JSON: {}", e);
                self.log("ERROR", &error_message)?;
                Err(AppError::JsonError(error_message))
            }
        }
    }

    fn save(&mut self) -> Result<(), AppError> {
        // Ensure the directory exists before saving
        self.ensure_directory()?;
        
        let json = serde_json::to_string_pretty(&self.accounts)
            .map_err(|e| {
                let error_message = format!("Failed to serialize to JSON: {}", e);
                self.log("ERROR", &error_message).ok();
                AppError::JsonError(error_message)
            })?;

        match File::create(&self.file_path) {
            Ok(mut file) => {
                file.write_all(json.as_bytes())
                    .map_err(|e| {
                        let error_message = format!("Failed to write to file: {}", e);
                        self.log("ERROR", &error_message).ok();
                        AppError::FileError(error_message)
                    })?;
                
                // More specific log message
                if self.accounts.len() == 1 {
                    self.log("INFO", "Saved 1 account to storage")?;
                } else {
                    self.log("INFO", &format!("Saved {} accounts to storage", self.accounts.len()))?;
                }
                Ok(())
            },
            Err(e) => {
                let error_message = format!("Failed to create file: {}", e);
                self.log("ERROR", &error_message)?;
                Err(AppError::FileError(error_message))
            }
        }
    }
}


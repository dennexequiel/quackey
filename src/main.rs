//! A TOTP (Time-based One-Time Password) generator application.
//! This application allows users to store and generate TOTP codes for various accounts directly from their terminal.

mod account;
mod config;
mod error;
mod logger;
mod storage;
mod ui;

use account::Account;
use colored::*;
use config::Config;
use dialoguer::{Confirm, Input, Select};
use error::AppError;
use logger::Logger;
use std::io::{self};
use std::thread;
use std::time::Duration;
use storage::Storage;
use totp_rs::{Algorithm, TOTP};
use ui::{display_screen, display_welcome_screen, display_exit_screen, 
         get_terminal_width, center_text, clear_screen, 
         create_spinner, wait_for_input,
         display_accounts_table, display_totp_results};

/// Application entry point that initializes the TOTP generator
fn main() -> Result<(), AppError> {
    let config = match run_onboarding() {
        Ok(config) => config,
        Err(AppError::PermissionError(msg)) => {
            eprintln!("{}", "Error:".red().bold());
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("{}", "Please run the application with appropriate permissions or choose a different location for your files.".bright_black());
            eprintln!("{}", "You can try running the application in a directory where you have write permissions.".bright_black());
            return Err(AppError::PermissionError(msg));
        }
        Err(e) => return Err(e),
    };

    let mut logger = match Logger::new(&config.get_log_file_path()) {
        Ok(logger) => logger,
        Err(AppError::PermissionError(msg)) => {
            eprintln!("{}", "Error:".red().bold());
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("{}", "Please run the application with appropriate permissions or choose a different location for your log file.".bright_black());
            return Err(AppError::PermissionError(msg));
        }
        Err(e) => return Err(e),
    };

    let mut storage = match Storage::new_with_logger(&config.get_storage_file_path(), Some(logger.clone())) {
        Ok(storage) => storage,
        Err(AppError::PermissionError(msg)) => {
            eprintln!("{}", "Error:".red().bold());
            eprintln!("{}", msg);
            eprintln!();
            eprintln!("{}", "Please run the application with appropriate permissions or choose a different location for your storage file.".bright_black());
            return Err(AppError::PermissionError(msg));
        }
        Err(e) => return Err(e),
    };

    logger.info("Application started")?;

    run_main_loop(&mut storage, &mut logger)?;

    Ok(())
}

/// Runs the onboarding process if configuration doesn't exist
fn run_onboarding() -> Result<Config, AppError> {
    let config = Config::load()?;

    if !std::path::Path::new("config.json").exists() {
        display_screen("Welcome to Quackey - Initial Setup");

        println!("{}", "Default Configuration:".bright_black());
        println!("{}", "  - Accounts will be saved in the same directory as the application".bright_black());
        println!("{}", "  - You can change these settings later from the menu".bright_black());
        println!();
        
        let use_defaults = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Would you like to use the default configuration?")
            .default(true)
            .interact()
            .unwrap_or(true);

        if use_defaults {
            println!();
            println!("{}", "Using default configuration.".bright_black());
            println!(
                "{}",
                "You can change these settings later from the menu.".bright_black()
            );
            println!();

            config.save()?;

            println!("{}", "✅ Configuration saved successfully!".green().bold());
            println!("{}", "Your Quackey TOTP generator is ready to use, quack quack!".bright_black());

            wait_for_input()?;

            return Ok(config);
        }

        let storage_dir = get_file_path("accounts storage file", ".")?;

        let mut new_config = Config { storage_dir };

        new_config.validate_paths()?;
        new_config.ensure_directories()?;
        new_config.save()?;

        println!();
        println!("{}", "✅ Configuration saved successfully!".green().bold());
        println!("{}", "Your Quackey TOTP generator is ready to use, quack quack!".bright_black());

        wait_for_input()?;

        Ok(new_config)
    } else {
        Ok(config)
    }
}

/// Gets a file path from user input with validation
fn get_file_path(prompt: &str, default: &str) -> Result<String, AppError> {
    println!();
    println!("{}", "Path format options:".bright_black());
    println!("{}", "  - Relative path (e.g., 'totp')".bright_black());
    println!(
        "{}",
        "  - Absolute path (e.g., '/home/user/quackey/totp' or 'D:/Quackey/totp')".bright_black()
    );
    println!();
    println!("{}", "Notes:".bright_black());
    println!(
        "{}",
        "  - Use forward slashes (/) even on Windows for consistency".bright_black()
    );
    println!(
        "{}",
        "  - Non-existent directories will be created automatically".bright_black()
    );
    println!(
        "{}",
        "  - You must have write permissions for the specified location".bright_black()
    );
    println!();

    let path: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!(
            "Directory path for {} (press Enter for default)",
            prompt
        ))
        .default(default.to_string())
        .interact_text()
        .unwrap_or_else(|_| default.to_string());

    if path.trim().is_empty() {
        return Err(AppError::InvalidInput(format!(
            "{} path cannot be empty",
            prompt
        )));
    }

    Ok(path.trim().to_string())
}

/// Runs the main application loop
fn run_main_loop(storage: &mut Storage, logger: &mut Logger) -> Result<(), AppError> {
    loop {
        clear_screen();
        display_welcome_screen();

        let selection = display_menu_and_get_selection()?;

        clear_screen();

        if handle_menu_selection(selection, storage, logger)? {
            break;
        }
    }
    Ok(())
}

/// Displays menu and gets user selection
fn display_menu_and_get_selection() -> Result<usize, AppError> {
    let selections = &[
        "🔢 Generate TOTP",
        "📂 Manage Accounts",
        "⚙️ Configure Settings",
        "🦆 Exit",
    ];

    Ok(
        Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select an option")
            .default(0)
            .items(selections)
            .interact()
            .unwrap_or(3),
    )
}

/// Displays the account management submenu and gets user selection
fn display_account_management_menu() -> Result<usize, AppError> {
    let selections = &[
        "👀 View saved accounts",
        "📄 Add new account",
        "📝 Edit account",
        "🗑️ Delete account",
        "👈 Back to main menu",
    ];

    Ok(
        Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select an account management option")
            .default(0)
            .items(selections)
            .interact()
            .unwrap_or(4),
    )
}

/// Handles the menu selection and returns whether the application should exit
fn handle_menu_selection(
    selection: usize,
    storage: &mut Storage,
    logger: &mut Logger,
) -> Result<bool, AppError> {
    match selection {
        0 => generate_totp(storage, logger)?,
        1 => {
            loop {
                clear_screen();
                display_screen("Account Management");

                let submenu_selection = display_account_management_menu()?;

                clear_screen();

                if submenu_selection == 4 {
                    break;
                }

                handle_account_management_selection(submenu_selection, storage, logger)?;
            }
        }
        2 => configure_settings(storage, logger)?,
        3 => {
            logger.info("Application exiting")?;
            display_exit_screen();

            println!("\n{}", "Press Enter to exit...".bright_black());
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;

            return Ok(true);
        }
        _ => unreachable!(),
    }
    Ok(false)
}

/// Handles the account management menu selection
fn handle_account_management_selection(
    selection: usize,
    storage: &mut Storage,
    logger: &mut Logger,
) -> Result<(), AppError> {
    match selection {
        0 => view_accounts(storage, logger)?,
        1 => add_account(storage, logger)?,
        2 => edit_account(storage, logger)?,
        3 => delete_account(storage, logger)?,
        4 => (), // Back to main menu
        _ => unreachable!(),
    }
    Ok(())
}

/// Adds a new TOTP account
fn add_account(storage: &mut Storage, logger: &mut Logger) -> Result<(), AppError> {
    display_screen("Add New Account");

    let (name, issuer) = match get_new_account_details() {
        Ok(details) => details,
        Err(e) => {
            println!("{}", format!("⛔ Error: {}", e).red().bold());
            println!();
            println!(
                "{}",
                "Please try again with a valid account name.".bright_black()
            );
            wait_for_input()?;
            return Ok(());
        }
    };

    let secret = match get_validated_secret() {
        Ok(secret) => secret,
        Err(e) => {
            println!("{}", format!("⛔ Error: {}", e).red().bold());
            println!();
            println!(
                "{}",
                "Please try again with a valid secret key.".bright_black()
            );
            wait_for_input()?;
            return Ok(());
        }
    };

    let (digits, period, algorithm) = match get_totp_parameters() {
        Ok(params) => params,
        Err(e) => {
            println!("{}", format!("⛔ Error: {}", e).red().bold());
            println!();
            println!(
                "{}",
                "Please try again with valid TOTP parameters.".bright_black()
            );
            wait_for_input()?;
            return Ok(());
        }
    };

    let account = Account::new(
        name.clone(),
        secret,
        digits,
        period,
        algorithm,
        issuer.clone(),
    );

    println!();
    let spinner = create_spinner("Saving account...".to_string());

    match storage.add_account(account.clone()) {
        Ok(_) => {
            thread::sleep(Duration::from_millis(500));
            spinner.finish_and_clear();

            logger.info(&format!("Added new account: {}", name))?;
            println!("{}", "👌 Account added successfully, quack!".green().bold());
        }
        Err(e) => {
            spinner.finish_and_clear();
            println!("{}", format!("⛔ Error saving account: {}", e).red().bold());
            println!();
            println!(
                "{}",
                "Please try again or check your storage file permissions.".bright_black()
            );
        }
    }

    wait_for_input()
}

/// Gets account name and issuer from user input for a new account
fn get_new_account_details() -> Result<(String, Option<String>), AppError> {
    loop {
        let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Account name (e.g., 'me@example.com', 'my-github-username')")
            .interact_text()
            .unwrap_or_default();

        let trimmed_name = name.trim().to_string();

        if trimmed_name.is_empty() {
            println!("{}", "⛔ Account name cannot be empty.".red());
            println!();
            continue;
        }

        let issuer: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Issuer (optional, e.g., 'Google', 'GitHub')")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        return Ok((
            trimmed_name,
            if issuer.trim().is_empty() {
                None
            } else {
                Some(issuer.trim().to_string())
            },
        ));
    }
}

/// Gets account name and issuer from user input for editing an existing account
fn get_edit_account_details(current_name: &str, current_issuer: Option<&str>) -> Result<(String, Option<String>), AppError> {
    loop {
        let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Account name (e.g., 'me@example.com', 'my-github-username')")
            .default(current_name.to_string())
            .interact_text()
            .unwrap_or_else(|_| current_name.to_string());

        let trimmed_name = name.trim().to_string();

        if trimmed_name.is_empty() {
            println!("{}", "⛔ Account name cannot be empty.".red());
            println!();
            continue;
        }

        let issuer: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Issuer (optional, e.g., 'Google', 'GitHub')")
            .default(current_issuer.unwrap_or("").to_string())
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();

        return Ok((
            trimmed_name,
            if issuer.trim().is_empty() {
                None
            } else {
                Some(issuer.trim().to_string())
            },
        ));
    }
}

/// Edits an account in storage
fn edit_account(storage: &mut Storage, logger: &mut Logger) -> Result<(), AppError> {
    display_screen("Edit Account");

    let accounts = storage.get_accounts()?;

    if accounts.is_empty() {
        let width = get_terminal_width();
        println!(
            "{}",
            center_text("🦉 No accounts saved yet.", width).bright_red()
        );
        logger.warn("Attempted to edit account with no accounts")?;
        return wait_for_input();
    }

    let account = select_account(&accounts)?;

    println!();
    println!("{}", "Current account details:".green().bold());
    println!("{} {}", "Name:".blue(), account.name());
    if let Some(issuer) = account.issuer() {
        println!("{} {}", "Issuer:".blue(), issuer);
    } else {
        println!("{} {}", "Issuer:".blue(), "None");
    }
    println!("{} {}", "Digits:".blue(), account.digits());
    println!("{} {} seconds", "Period:".blue(), account.period());
    println!(
        "{} {}",
        "Algorithm:".blue(),
        match account.algorithm() {
            Algorithm::SHA1 => "SHA1",
            Algorithm::SHA256 => "SHA256",
            Algorithm::SHA512 => "SHA512",
        }
    );
    println!();

    println!(
        "{}",
        "Enter new details (press Enter to keep current value):".bright_black()
    );

    let (name, issuer) = match get_edit_account_details(account.name(), account.issuer().map(|s| s.as_str())) {
        Ok(details) => details,
        Err(e) => return Err(e),
    };

    storage.update_account(account.name(), name.clone(), issuer.clone())?;
    logger.info(&format!("Updated account: {}", name))?;

    println!();
    println!("{}", "✅ Account updated successfully!".green().bold());

    wait_for_input()
}

/// Deletes an account from storage
fn delete_account(storage: &mut Storage, logger: &mut Logger) -> Result<(), AppError> {
    let accounts = storage.get_accounts()?;

    if accounts.is_empty() {
        display_screen("Delete Account");
        let width = get_terminal_width();
        println!(
            "{}",
            center_text("🦉 No accounts saved yet.", width).bright_red()
        );
        logger.warn("Attempted to delete account with no accounts")?;
        return wait_for_input();
    }

    display_screen("Delete Account");

    let account = select_account(&accounts)?;

    let confirm = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!(
            "Are you sure you want to delete the account '{}'?",
            account.name()
        ))
        .default(false)
        .interact()
        .unwrap_or(false);

    if !confirm {
        println!();
        println!("{}", "Account deletion cancelled.".bright_black());
        return wait_for_input();
    }

    storage.delete_account(account.name())?;
    logger.info(&format!("Deleted account: {}", account.name()))?;

    println!();
    println!("{}", "✅ Account deleted successfully!".green().bold());

    wait_for_input()
}

/// Displays all saved accounts in a formatted table
fn view_accounts(storage: &Storage, logger: &mut Logger) -> Result<(), AppError> {
    display_screen("Saved Accounts");

    let accounts = storage.get_accounts()?;

    if accounts.is_empty() {
        let width = get_terminal_width();
        println!(
            "{}",
            center_text("🦉 No accounts saved yet.", width).bright_red()
        );
        logger.info("Viewed accounts (none saved)")?;
        return wait_for_input();
    }

    display_accounts_table(&accounts);
    logger.info("Viewed all saved accounts")?;
    wait_for_input()
}

/// Generates a TOTP code for a selected account
fn generate_totp(storage: &Storage, logger: &mut Logger) -> Result<(), AppError> {
    let accounts = storage.get_accounts()?;

    if accounts.is_empty() {
        display_screen("Generate TOTP");
        let width = get_terminal_width();
        println!(
            "{}",
            center_text("🦉 No accounts saved yet.", width).bright_red()
        );
        logger.warn("Attempted to generate TOTP with no accounts")?;
        return wait_for_input();
    }

    display_screen("Generate TOTP");

    let account = select_account(&accounts)?;

    println!();
    let spinner = create_spinner("Generating TOTP code...".to_string());

    let totp_result = account.generate_totp();
    let remaining = account.time_remaining();

    thread::sleep(Duration::from_millis(500));
    spinner.finish_and_clear();

    match totp_result {
        Ok(totp) => {
            display_totp_results(&totp, remaining)?;
            logger.info(&format!("Generated TOTP for account: {}", account.name()))?;
        }
        Err(e) => {
            println!("{}", "⛔ Error generating TOTP code, quack... *sniff*".red().bold());
            println!(
                "{}",
                "This account may have an invalid secret key.".bright_black()
            );
            println!(
                "{}",
                "Please delete this account and add it again with a valid key.".bright_black()
            );
            logger.error(&format!(
                "Failed to generate TOTP for account {}: {}",
                account.name(),
                e
            ))?;
        }
    }

    wait_for_input()
}

/// Gets and validates the secret key from user input
fn get_validated_secret() -> Result<String, AppError> {
    loop {
        let secret_input: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Secret key")
            .interact_text()
            .unwrap_or_default();

        let cleaned_secret = secret_input.trim().replace(" ", "").to_uppercase();

        if cleaned_secret.is_empty() {
            println!("{}", "⛔ Secret key cannot be empty.".red());
            println!();
            continue;
        }

        if cleaned_secret.len() < 26 {
            println!(
                "{}",
                "⛔ Secret key is too short. It must be at least 26 characters long.".red()
            );
            println!();
            continue;
        }

        let spinner = create_spinner("Validating secret key...".to_string());
        let test_totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            cleaned_secret.clone().into_bytes(),
        );

        thread::sleep(Duration::from_millis(500));
        spinner.finish_and_clear();

        match test_totp {
            Ok(_) => return Ok(cleaned_secret),
            Err(e) => {
                println!("{} {}", "⛔ Invalid secret key:".bright_red(), e);
                println!();
                continue;
            }
        }
    }
}

/// Gets TOTP parameters (digits, period, algorithm) from user input
fn get_totp_parameters() -> Result<(usize, u64, Algorithm), AppError> {
    let digits_options = &["6 digits", "7 digits", "8 digits"];
    let digits_selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select digits")
        .default(0)
        .items(digits_options)
        .interact()
        .unwrap_or(0);

    let digits = match digits_selection {
        0 => 6,
        1 => 7,
        2 => 8,
        _ => 6,
    };

    let period_options = &["30 seconds", "60 seconds", "90 seconds"];
    let period_selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select refresh time")
        .default(0)
        .items(period_options)
        .interact()
        .unwrap_or(0);

    let period = match period_selection {
        0 => 30,
        1 => 60,
        2 => 90,
        _ => 30,
    };

    let algo_options = &["SHA1", "SHA256", "SHA512"];
    let algo_selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select algorithm")
        .default(0)
        .items(algo_options)
        .interact()
        .unwrap_or(0);

    let algorithm = match algo_selection {
        0 => Algorithm::SHA1,
        1 => Algorithm::SHA256,
        2 => Algorithm::SHA512,
        _ => Algorithm::SHA1,
    };

    Ok((digits, period, algorithm))
}

/// Selects an account from the list of available accounts
fn select_account(accounts: &[Account]) -> Result<&Account, AppError> {
    if accounts.len() == 1 {
        println!(
            "{} {}",
            "Using the only available account:".blue(),
            accounts[0].name()
        );
        return Ok(&accounts[0]);
    }

    let account_names: Vec<String> = accounts
        .iter()
        .map(|a| {
            if let Some(issuer) = a.issuer() {
                format!("{} ({})", a.name(), issuer)
            } else {
                a.name().to_string()
            }
        })
        .collect();

    let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select an account")
        .default(0)
        .items(&account_names)
        .interact()
        .unwrap_or(0);

    Ok(&accounts[selection])
}

/// Configures application settings
fn configure_settings(storage: &mut Storage, logger: &mut Logger) -> Result<(), AppError> {
    display_screen("Configure Settings");

    let config = Config::load()?;

    println!("{}", "Configure your Quackey settings".green().bold());
    println!(
        "{}",
        "You can change the path for your accounts storage file.".bright_black()
    );
    println!();

    let storage_dir = get_file_path("accounts storage file", &config.storage_dir)?;

    let mut config = Config { storage_dir };

    config.validate_paths()?;
    config.ensure_directories()?;
    config.save()?;

    let storage_path_changed = config.get_storage_file_path() != storage.file_path();

    if storage_path_changed {
        let old_path = storage.file_path().to_string();
        let new_path = config.get_storage_file_path();

        println!();
        println!("{}", "Changing storage file path:".bright_black());
        println!("{} {}", "From:".blue(), old_path);
        println!("{} {}", "To:".blue(), new_path);
        println!();

        if std::path::Path::new(&new_path).exists() {
            println!(
                "{}",
                "⚠️  The new storage file already exists.".yellow().bold()
            );
            println!("{}", "If it contains accounts, they will be loaded instead of copying from the old file.".bright_black());
            println!("{}", "If you want to keep your current accounts, please rename or move the existing file.".bright_black());
            println!();

            let proceed = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Do you want to proceed?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if !proceed {
                println!();
                println!("{}", "Operation cancelled.".bright_black());
                return wait_for_input();
            }
        }

        storage.update_file_path(&new_path)?;
        println!(
            "{}",
            "✅ Storage file path updated successfully!".green().bold()
        );
    }

    if config.get_log_file_path() != logger.file_path() {
        logger.update_file_path(&config.get_log_file_path())?;
    }

    logger.info("Application settings updated")?;

    wait_for_input()
}
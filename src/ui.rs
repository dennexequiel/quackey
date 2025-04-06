use crate::account::Account;
use crate::error::AppError;
use arboard::Clipboard;
use colored::*;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Cell, Table, format};
use std::io::{self, Write};
use totp_rs::Algorithm;

/// Application configuration constants
const SPINNER_TEMPLATE: &str = "{spinner:.green} {msg}";
const SPINNER_CHARS: &str = "⠁⠂⠄⡀⢀⠠⠐⠈ ";
const DUCK_ASCII: &str = r#"
   >(.)__ <(.)__
    (___/  (___/ 
"#;

/// Displays a generic screen with the duck ASCII, header and separators
pub fn display_screen(title: &str) {
    let width = get_terminal_width();

    clear_screen();
    println!("\n\n");
    println!("{}", centered_duck(width).bright_yellow());
    println!("{}", "-".repeat(width).yellow());
    println!("{}", center_text(title, width).bright_green().bold());
    println!("{}", "-".repeat(width).yellow());
    println!(
        "{}",
        "Note: For best experience, avoid resizing the terminal during use.".bright_black()
    );
    println!();
}

/// Displays the welcome screen
pub fn display_welcome_screen() {
    display_screen("Quackey: Generate TOTP directly from your terminal");
}

/// Displays the exit screen
pub fn display_exit_screen() {
    let width = get_terminal_width();

    clear_screen();
    println!("\n\n");
    println!("{}", centered_duck(width).bright_yellow());
    println!(
        "{}",
        center_text("Thanks for using Quackey, quack quack!", width)
            .bright_green()
            .bold()
    );
}

/// Gets the current terminal width
pub fn get_terminal_width() -> usize {
    match term_size::dimensions() {
        Some((w, _)) => w,
        None => 80, // Default width if terminal size can't be determined
    }
}

/// Centers text in the terminal
pub fn center_text(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(text.len()) / 2;
    format!("{:>width$}", text, width = text.len() + padding)
}

/// Returns the centered duck ASCII art
pub fn centered_duck(width: usize) -> String {
    let mut centered = String::new();
    for line in DUCK_ASCII.lines() {
        if !line.trim().is_empty() {
            centered.push_str(&center_text(line, width));
            centered.push('\n');
        }
    }
    centered
}

/// Displays the results of TOTP generation
pub fn display_totp_results(totp: &str, remaining: u64) -> Result<(), AppError> {
    println!("{}", "Here is your code, quack!".green().bold());

    let formatted_totp = format_totp(totp);
    println!(
        "{} {}",
        "🔑 Code:".blue(),
        formatted_totp.bright_white().bold()
    );
    println!("{} {} seconds", "⌛ Expires in:".blue(), remaining);
    println!();

    if Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Copy to clipboard")
        .default(true)
        .interact()
        .unwrap_or(false)
    {
        match copy_to_clipboard(totp) {
            Ok(_) => println!("{}", "📋 Copied to clipboard, quack!".green()),
            Err(_) => println!(
                "{}",
                "⛔ Failed to copy to clipboard, quack... *sniff*".red()
            ),
        }
    }

    Ok(())
}

/// Formats a TOTP code with spaces for better readability
pub fn format_totp(totp: &str) -> String {
    if totp.len() <= 3 {
        return totp.to_string();
    }

    let mid = totp.len() / 2;
    format!("{} {}", &totp[..mid], &totp[mid..])
}

/// Copies text to the system clipboard
pub fn copy_to_clipboard(text: &str) -> Result<(), AppError> {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(text).unwrap();
    Ok(())
}

/// Displays accounts in a formatted table
pub fn display_accounts_table(accounts: &[Account]) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // Add header row
    let headers = vec![
        Cell::new("#").style_spec("bFg"),
        Cell::new("Account Name").style_spec("bFg"),
        Cell::new("Issuer").style_spec("bFg"),
        Cell::new("Digits").style_spec("bFg"),
        Cell::new("Period").style_spec("bFg"),
        Cell::new("Algorithm").style_spec("bFg"),
    ];
    table.add_row(prettytable::Row::new(headers));

    // Add account rows
    for (i, account) in accounts.iter().enumerate() {
        let algo_name = match account.algorithm() {
            Algorithm::SHA1 => "SHA1",
            Algorithm::SHA256 => "SHA256",
            Algorithm::SHA512 => "SHA512",
        };

        let row = vec![
            Cell::new(&format!("{}.", i + 1)).style_spec("Fy"),
            Cell::new(&account.name()).style_spec("FW"),
            Cell::new(&account.issuer().unwrap_or(&"".to_string())).style_spec("FB"),
            Cell::new(&account.digits().to_string()).style_spec("FB"),
            Cell::new(&format!("{}s", account.period())).style_spec("FB"),
            Cell::new(&algo_name.to_string()).style_spec("FB"),
        ];
        table.add_row(prettytable::Row::new(row));
    }

    table.printstd();
}

/// Helper function to wait for user input
pub fn wait_for_input() -> Result<(), AppError> {
    println!("\n{}", "Press Enter to continue...".bright_black());
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(())
}

/// Clears the terminal screen
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Creates a new progress spinner with consistent styling
pub fn create_spinner(message: String) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars(SPINNER_CHARS)
            .template(SPINNER_TEMPLATE)
            .unwrap(),
    );
    spinner.set_message(message);
    spinner
} 
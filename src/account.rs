use serde::{Serialize, Deserialize};
use totp_rs::{TOTP, Algorithm as TotpAlgorithm, Secret};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::AppError;

// Create our own Algorithm enum that can be serialized/deserialized
// Use serde rename attributes to match the totp-rs variant names
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Algorithm {
    #[serde(rename = "SHA1")]
    Sha1,
    #[serde(rename = "SHA256")]
    Sha256,
    #[serde(rename = "SHA512")]
    Sha512,
}

// Conversion between our Algorithm and totp_rs::Algorithm
impl From<Algorithm> for TotpAlgorithm {
    fn from(algo: Algorithm) -> Self {
        match algo {
            Algorithm::Sha1 => TotpAlgorithm::SHA1,
            Algorithm::Sha256 => TotpAlgorithm::SHA256,
            Algorithm::Sha512 => TotpAlgorithm::SHA512,
        }
    }
}

// Default functions for serde
fn default_period() -> u64 { 30 }
fn default_digits() -> usize { 6 }
fn default_algorithm() -> Algorithm { Algorithm::Sha1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    name: String,
    secret: String,
    #[serde(default = "default_digits")]
    digits: usize,
    #[serde(default = "default_period")]
    period: u64,
    #[serde(default = "default_algorithm")]
    algorithm: Algorithm,
    issuer: Option<String>,
}

impl Account {
    pub fn new(
        name: String,
        secret: String,
        digits: usize,
        period: u64,
        algorithm: TotpAlgorithm,
        issuer: Option<String>
    ) -> Self {
        // Convert from totp_rs::Algorithm to our Algorithm
        let algo = match algorithm {
            TotpAlgorithm::SHA1 => Algorithm::Sha1,
            TotpAlgorithm::SHA256 => Algorithm::Sha256,
            TotpAlgorithm::SHA512 => Algorithm::Sha512,
        };

        Self {
            name,
            secret,
            digits,
            period,
            algorithm: algo,
            issuer,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn digits(&self) -> usize {
        self.digits
    }

    pub fn period(&self) -> u64 {
        self.period
    }

    pub fn algorithm(&self) -> TotpAlgorithm {
        self.algorithm.into()
    }

    pub fn issuer(&self) -> Option<&String> {
        self.issuer.as_ref()
    }

    /// Gets the account's secret key
    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn generate_totp(&self) -> Result<String, AppError> {
        // Create a TOTP according to the documentation
        let totp = TOTP::new(
            self.algorithm.into(),
            self.digits,
            1, // step_size
            self.period,
            Secret::Encoded(self.secret.clone()).to_bytes().unwrap(),
        ).map_err(|e| AppError::TotpError(format!("Failed to create TOTP: {}", e)))?;

        // Generate the current TOTP code
        let code = totp.generate_current()
            .map_err(|e| AppError::SystemTimeError(e))?;

        Ok(code)
    }

    pub fn time_remaining(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let current_period = now / self.period;
        let next_period_start = (current_period + 1) * self.period;

        next_period_start - now
    }
}


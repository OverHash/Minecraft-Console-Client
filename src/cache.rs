use std::{fs, str::FromStr};

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use toml_edit::Datetime;

const CACHE_PATH: &str = "cache.toml";

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Cache {
    /// The microsoft token
    microsoft_refresh_token: String,

    /// The minecraft token
    minecraft_token: CachedSessionToken,
}

impl Cache {
    pub fn get() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        // if the cache file does not exist, we return None
        // otherwise, if there was an error we bubble up
        // if success, we get the cache
        let cache = match fs::read_to_string(CACHE_PATH) {
            Ok(cache) => toml_edit::easy::from_str(&cache)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(Box::new(e)),
        };

        Ok(Some(cache))
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(CACHE_PATH, toml_edit::easy::to_string_pretty(self)?)?;

        Ok(())
    }

    /// Retrieves the inner minecraft token, wrapped in an option.
    /// Returns `None` if the token has expired, otherwise returns the token.
    pub fn get_minecraft_token(&self) -> Option<String> {
        self.minecraft_token.get_token()
    }

    /// Saves a new Minecraft token with expiry time to the cache
    pub fn save_minecraft_token(
        &mut self,
        token: String,
        expiry_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // write and save
        self.minecraft_token = CachedSessionToken::new(token, expiry_time)?;
        self.save()?;

        Ok(())
    }

    /// Retrieves the inner microsoft refresh token.
    pub fn get_microsoft_refresh_token(&self) -> &str {
        &self.microsoft_refresh_token
    }

    /// Saves a new Microsoft refresh token to the cache
    pub fn save_microsoft_refresh_token(
        &mut self,
        token: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // write and save
        self.microsoft_refresh_token = token;
        self.save()?;

        Ok(())
    }
}

impl std::default::Default for Cache {
    fn default() -> Self {
        Self {
            microsoft_refresh_token: "".to_string(),
            minecraft_token: CachedSessionToken {
                token: "".to_string(),
                expiry_time: toml_edit::Datetime::from_str("2011-11-18T12:00:00Z").unwrap(),
            },
        }
    }
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cache")
            .field(
                "microsoft_refresh_token",
                &"X".repeat(self.microsoft_refresh_token.len()),
            )
            .field("minecraft_token", &self.minecraft_token)
            .finish()
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct CachedSessionToken {
    /// The token itself
    pub token: String,
    ///  An ISO-8601 timestamp of when the token expires
    pub expiry_time: Datetime,
}

impl CachedSessionToken {
    pub fn new(
        token: String,
        expiry_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            token,
            expiry_time: toml_edit::Datetime::from_str(
                &expiry_time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )?,
        })
    }

    /// Retrieves the inner minecraft token, and if it valid
    pub fn get_token(&self) -> Option<String> {
        let token = &self.token;

        let expiry_time: DateTime<chrono::Utc> =
            chrono::DateTime::from_str(&self.expiry_time.to_string())
                .unwrap_or_else(|_| panic!("Failed to parse expiry time '{}'", self.expiry_time));

        // if expiry_time > current_time, then we have not expired
        // and should return the token
        if expiry_time > chrono::Utc::now() {
            Some(token.clone())
        } else {
            None
        }
    }
}

impl std::fmt::Debug for CachedSessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedSessionToken")
            .field("token", &"X".repeat(self.token.len()))
            .field("expiry_time", &self.expiry_time)
            .finish()
    }
}

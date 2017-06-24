pub mod log;
pub mod args;

use egg_mode;
use std::io;
use toml;
use chrono::prelude::*;


/*
 *  Configuration
 */

#[derive(Debug)]
pub struct Conf {
    pub span: i64,
    pub dryrun: bool,
    pub quiet: bool,
    pub file: String,
    pub auth: Auth,
}

impl Conf {
    pub fn new() -> Self {
        Conf {
            span: 31449600, // 52 Weeks
            dryrun: true,
            quiet: false,
            file: "eggspire.toml".to_string(),
            auth: Auth::new(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Auth {
    pub access_key: String,
    pub access_sec: String,
    pub con_key: String,
    pub con_sec: String,
}

impl Auth {
    pub fn new() -> Self {
        Auth {
            access_key: String::with_capacity(50),
            access_sec: String::with_capacity(45),
            con_key: String::with_capacity(25),
            con_sec: String::with_capacity(50),
        }
    }
    /// Parse twitter auth credentials from toml file
    pub fn from_file(path: &str) -> Result<Auth, Error> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::open(path)?;
        let mut buf = String::with_capacity(200);
        file.read_to_string(&mut buf)?;
        let auth: Self = toml::from_str(&buf)?;
        Ok(auth)
    }
}

/*
 *  Errors
 */

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParseError(toml::de::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::ParseError(err)
    }
}


/*
 *  Extend the egg-mode crate
 */

pub trait Eggspire {
    fn expired(&self, span: i64) -> bool;
    fn faved(&self) -> bool;
}

impl Eggspire for egg_mode::tweet::Tweet {
    /// Returns true if a tweet is older than span (in seconds) and not faved
    fn expired(&self, span: i64) -> bool {
        (Utc::now().timestamp() - self.created_at.timestamp()) > span
    }

    /// Returns true if the tweet is favorited by the authenticated user or the request fails
    fn faved(&self) -> bool {
        if let Some(faved) = self.favorited {
           faved
        } else {
           true
        } // Keep if request fails
    }
}

/*
 *  Macros
 */

/// Wraps an if condition around println!()
#[macro_export]
macro_rules! cprintln {
        ($cond:expr) => (
            if $conf { print!("\n") }
            );
        ($cond:expr, $fmt:expr) => (
            if $cond { println!($fmt)}
            );
        ($cond:expr, $fmt:expr, $($arg:tt)*) => (
            if $cond { println!($fmt, $($arg)*) }
            );
}

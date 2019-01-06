pub mod log;
pub mod args;

use egg_mode::{Token, KeyPair};
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
    pub likes: bool,
    pub file: String,
    pub token: egg_mode::Token,
}

impl Conf {
    pub fn new() -> Self {
        Conf {
            span: 31449600, // 52 Weeks
            dryrun: true,
            quiet: false,
            likes: false,
            file: "eggspire.toml".to_string(),
            token: Token::Access{ consumer: KeyPair::new("", ""),
                                  access: KeyPair::new("", ""), },
        }
    }
    /// Generate authentication token from auth credentials of configuration file
    pub fn get_token_from_file(&mut self) -> Result<(), Error> {
        let auth = Auth::from_file(&self.file)?;

        self.token = egg_mode::Token::Access {
            consumer: egg_mode::KeyPair::new(auth.access_key, auth.access_sec),
            access: egg_mode::KeyPair::new(auth.con_key, auth.con_sec),
        };

        Ok(())
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
    /// Parse twitter auth credentials from toml file
    pub fn from_file(path: &str) -> Result<Auth, Error> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::open(path)?;
        let mut buf = String::with_capacity(200);
        file.read_to_string(&mut buf)?;
        let auth : Self = toml::from_str(&buf)?;
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
    fn has_owner_id(&self, id : u64) -> bool;
}

impl Eggspire for egg_mode::tweet::Tweet {

    /// Returns true if a tweet is older than span (in seconds)
    fn expired(&self, span: i64) -> bool {
        (Utc::now().timestamp() - self.created_at.timestamp()) > span
    }

    /// Returns true if the tweet is faved by the authenticated user or the request fails
    fn faved(&self) -> bool {
        if let Some(faved) = self.favorited {
           faved
        } else {
           true
        }
    }

    /// Returns true if the owner id of the tweet matches the passed id
    fn has_owner_id(&self, id : u64) -> bool {
        if let Some(owner) = &self.user {
            owner.id == id
        } else {
            true
        }
    }
}

/*
 *  Macros
 */
/// Wraps an if condition around println!()
#[macro_export]
macro_rules! cprintln {
//        ($cond:expr) => (
//            if $conf { print!("\n") }
//            );
        ($cond:expr, $fmt:expr) => (
            if $cond { println!($fmt)}
            );
        ($cond:expr, $fmt:expr, $($arg:tt)*) => (
            if $cond { println!($fmt, $($arg)*) }
            );
}

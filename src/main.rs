extern crate clap;          // Argparser
#[macro_use]
extern crate log;           // Log
extern crate env_logger;    // Log
#[macro_use]
extern crate serde_derive;  // Parse config file
extern crate serde;         // Parse Config file
extern crate toml;          // Parse config file
extern crate egg_mode;      // Twitter API
extern crate chrono;        // Date/Time
extern crate futures;       // Async IO
extern crate tokio_core;    // Async IO

use egg_mode::{KeyPair, Token, verify_tokens, tweet};
use chrono::prelude::*;
use tokio_core::reactor::Core;

#[macro_use]
mod util;

use util::{Eggspire, Auth, args};

fn main() {

    // Init logger and set default log level (Error)
    util::log::init(::log::LogLevelFilter::Error);
    // Get configuration from arg parser
    let mut conf = args::get_conf();

    debug!("{:#?}", conf);

    // Get auth credentials from file
    match Auth::from_file(&conf.file) {
        Ok(a) => conf.auth = a,
        Err(e) => {
            error!("Couldn't import auth credentials.");
            debug!("{:?}", e);
            std::process::exit(1);
        }
    }

    trace!("{:#?}", conf.auth);

    // Authenticate with Twitter
    let access_token = KeyPair::new(conf.auth.access_key, conf.auth.access_sec);
    let con_token = KeyPair::new(conf.auth.con_key, conf.auth.con_sec);
    let token = Token::Access {
        consumer: con_token,
        access: access_token,
    };

    let mut core= Core::new().unwrap();
    let handle = core.handle();


    let user;
    match core.run(verify_tokens(&token, &handle)) {
        Err(e) => {
            error!("The authentication failed.");
            debug!("{:?}", e);
            std::process::exit(1);
        }
        Ok(u) => user = u,
    }
    info!("Authenticated as User: {} (Id: {})", user.name, user.id);

    // Get timeline of authenticated user
    let mut tl = tweet::user_timeline(user.id, true, true, &token, &handle).with_page_size(100);

    // Get IDs of expired tweets
    let mut to_delete = Vec::<u64>::new();
    loop {
        match core.run(tl.older(None)) {
            Ok(tweets) => {
                trace!("tl count: {:?}, max_id: {:?}, min_id: {:?}",
                       tl.count,
                       tl.max_id,
                       tl.min_id);
                for status in &tweets {
                    if status.expired(conf.span) && !status.faved() {
                        to_delete.push(status.id);
                    }
                }
                if tl.min_id == None {
                    break;
                } // reached last tweet
            }
            Err(e) => {
                use egg_mode::error::Error::*;
                match e {
                    RateLimit(utc) => {
                        let sleep_s = Utc::now().timestamp() - utc as i64;
                        if sleep_s > 0 {
                            cprintln!(!conf.quiet,
                                      "The Twitter API rate limit was reached.\
                                      Sleeping for {} seconds.",
                                      sleep_s);
                            std::thread::sleep(std::time::Duration::from_secs(sleep_s as u64));
                        }
                    }
                    e => {
                        error!("Sorry, an unhandled error occurred.");
                        debug!("{:?}", e);
                        std::process::exit(1)
                    }
                }

            }
        }
    }
    cprintln!(!conf.quiet, "Found {} expired tweets.", to_delete.len());

    // Delete expired tweets
    if !conf.dryrun && to_delete.len() > 0 {
        let mut ctr = 0;
        for tweet in to_delete {
            if let Ok(deleted) = core.run(egg_mode::tweet::delete(tweet, &token, &handle)) {
                info!("Deleted: {}", deleted.text);
                ctr += 1;
            } else {
                error!("Deletion of tweet with id {} failed.", tweet);
            }
        }
        cprintln!(!conf.quiet, "Deleted {} expired tweets.", ctr);
    }
}

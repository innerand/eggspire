#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] mod util;

use chrono::prelude::*;
use egg_mode::{KeyPair, Token, verify_tokens, tweet};
use crate::util::{Eggspire, Auth, args};
use tokio::runtime::current_thread::{block_on_all};
use futures::future::Future;


fn main() {
    // Init logger and set default log level (Error)
    //--------------------------------------------------------------------------
    util::log::init(::log::LevelFilter::Error);

    // Get configuration from arg parser
    //--------------------------------------------------------------------------
    let mut conf = args::get_conf();
    debug!("{:#?}", conf);

    // Get auth credentials from configuration file
    //--------------------------------------------------------------------------
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
    //--------------------------------------------------------------------------
    let access_token = KeyPair::new(conf.auth.access_key, conf.auth.access_sec);
    let con_token = KeyPair::new(conf.auth.con_key, conf.auth.con_sec);
    let token = Token::Access {
        consumer: con_token,
        access: access_token,
    };

    let user;
    match block_on_all(verify_tokens(&token)) {
        Err(e) => {
            error!("The authentication failed.");
            debug!("{:?}", e);
            std::process::exit(1);
        }
        Ok(u) => user = u,
    }
    info!("Authenticated as User: {} (Id: {})", user.name, user.id);

    // Walk through timeline
    //--------------------------------------------------------------------------
    let mut tl : Option<tweet::Timeline> = None;    // User Timeline
    let mut current_id :Option<u64> = None;         // Track processed IDs
    let mut expired_ctr = 0;

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    loop {
        // Get new timeline struct
        if tl.is_none() {
            let mut new_tl = tweet::user_timeline(user.id, true, true, &token).with_page_size(100);

            if let Some(id) = current_id {
                new_tl.min_id = Some(id);
            }

            debug!("init tl count: {:?}, max_id: {:?}, min_id: {:?}",
                   new_tl.count, new_tl.max_id, new_tl.min_id);

            tl = Some(new_tl);
        }

        // Query timeline slice from Twitter
        match block_on_all(tl.unwrap().older(None)) {
            Ok((new_tl, tweets)) => {
                debug!("new_tl count: {:?}, max_id: {:?}, min_id: {:?}",
                       new_tl.count, new_tl.max_id, new_tl.min_id);

                for tweet in &tweets {
                    if tweet.expired(conf.span) && !tweet.faved(){
                        expired_ctr += 1;
                        if !conf.dryrun{
                            let id = tweet.id;
                            rt.spawn(tweet::delete(tweet.id, &token)
                                     .map(|response| {
                                          debug!("Deleted tweet {:}", response.id);
                                          ()
                                     })
                                     .map_err(move |err| {
                                             error!("Failed to delete tweet({:}): {:?}", id, err);
                                             ()
                                     }));
                        }
                    }
                }

                if new_tl.min_id.is_none() {
                    break;
                } else {
                    current_id = new_tl.min_id;
                }

                tl = Some(new_tl);
            }

            Err(e) => {
                tl = None;

                use egg_mode::error::Error::*;
                match e {
                    RateLimit(utc) => {
                        let sleep_s = Utc::now().timestamp() - utc as i64;
                        cprintln!(!conf.quiet,
                                  "Rate limit has been reached.\
                                   Sleeping for {} seconds.", sleep_s);
                        std::thread::sleep(std::time::Duration::from_secs(sleep_s as u64));
                        // TODO: Pause runtime
                    },
                    _ => unhandeled_error(Box::new(e)),
                }
            }
        }
    }

    cprintln!(!conf.quiet, "Found {} expired tweets.", expired_ctr);
    rt.shutdown_on_idle().wait().unwrap();
}

/// Print the error message and exit
fn unhandeled_error(error : Box<dyn std::error::Error>) -> ! {
    error!("Sorry, an unhandled error occurred.");
    debug!("{:?}", error);
    std::process::exit(1)
}

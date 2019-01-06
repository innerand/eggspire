#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] mod util;

use chrono::prelude::*;
use egg_mode::{verify_tokens, tweet};
use crate::util::{Eggspire, args};
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

    // Generate auth token
    //--------------------------------------------------------------------------
    match conf.get_token_from_file() {
        Ok(_) => (),
        Err(e) => {
            error!("Couldn't import auth credentials.");
            debug!("{:?}", e);
            std::process::exit(1);
        }
    }

    // Test auth token
    //--------------------------------------------------------------------------
    let user;
    match block_on_all(verify_tokens(&conf.token)) {
        Err(e) => {
            error!("The authentication failed.");
            debug!("{:?}", e);
            std::process::exit(1);
        }
        Ok(u) => user = u,
    }
    info!("Authenticated as User: {} (Id: {})", user.name, user.id);

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    {
    // Walk through user timeline
    //--------------------------------------------------------------------------
    let mut tl : Option<tweet::Timeline> = None;    // User Timeline
    let mut current_id :Option<u64> = None;         // Track processed IDs
    let mut expired_ctr = 0;

    loop {
        // Get new timeline struct
        if tl.is_none() {
            let mut new_tl = tweet::user_timeline(user.id, true, true, &conf.token).with_page_size(100);

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
                            rt.spawn(tweet::delete(tweet.id, &conf.token)
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
    }

    // Walk through user likes
    //--------------------------------------------------------------------------
    let mut tl : Option<tweet::Timeline> = None;    // User Timeline
    let mut current_id :Option<u64> = None;         // Track processed IDs
    let mut expired_ctr = 0;
    let span = conf.span;


    loop {
        // Skip likes if not enabled
        if !conf.likes { break; }

        // Get new timeline struct
        if tl.is_none() {
            let mut new_tl = tweet::liked_by(user.id, &conf.token).with_page_size(100);

            if let Some(id) = current_id {
                new_tl.min_id = Some(id);
                debug!("Set some id to {:}", id);
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
                for tweet in tweets.iter()
                                .filter(|tweet| tweet.expired(span))
                                .filter(|tweet| !tweet.has_owner_id(user.id)) {
                    expired_ctr += 1;
                    if !conf.dryrun {
                        let id = tweet.id;
                        rt.spawn(tweet::unlike(tweet.id, &conf.token)
                                 .map(|response| {
                                     debug!("Unliked tweet {:}", response.id);
                                     ()
                                 })
                                 .map_err(move |err| {
                                     error!("Failed to unlike tweet({:}): {:?}", id, err);
                                     ()
                                 }));
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

    cprintln!(!conf.quiet, "Found {} tweets to unlike.", expired_ctr);

    rt.shutdown_on_idle().wait().unwrap();
}

/* WIP
fn process_tl(tl : tweet::Timeline<'static>, token : &'static egg_mode::Token ) -> () {
    tokio::spawn(tl.older(None).map(move |(new_tl, tweets)| {
        if new_tl.min_id == None {
        } else {
            // New Span for next timeline
            tl_older(new_tl, token);
        }

        let span = 1;
        let user_id = 1;
        let dryrun = true;
        let span = 1;

        for tweet in tweets.iter()
                        .filter(|tweet| tweet.expired(span))
                        .filter(|tweet| !tweet.has_owner_id(user_id)) {
            //expired_ctr += 1;
            if !dryrun {
                let id = tweet.id;
                tokio::spawn(tweet::unlike(tweet.id, &token)
                             .map(|response| {
                                  debug!("Unliked tweet {:}", response.id);
                                  ()
                             })
                             .map_err(move |err| {
                                     error!("Failed to unlike tweet({:}): {:?}", id, err);
                                     ()
                             }));
            }
        }

    }).map_err(|err| {
        ()
    }));
}
*/

/// Print the error message and exit
fn unhandeled_error(error : Box<dyn std::error::Error>) -> ! {
    error!("Sorry, an unhandled error occurred.");
    debug!("{:?}", error);
    std::process::exit(1)
}

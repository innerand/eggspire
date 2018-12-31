#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] mod util;

use egg_mode::{KeyPair, Token, verify_tokens, tweet};
use crate::util::{Eggspire, Auth, args};
use tokio::runtime::current_thread::block_on_all;

fn main() {
    // Init logger and set default log level (Error)
    //--------------------------------------------------------------------------
    util::log::init(::log::LevelFilter::Debug);
    // Get configuration from arg parser
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
    let mut to_delete = Vec::<u64>::new();          // IDs  of tweets to remove

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

                for status in &tweets {
                    if status.expired(conf.span) && !status.faved() {
                        to_delete.push(status.id);
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
                // tl = None;
                match e {
                    // TODO: Error handling
                    _ => unhandeled_error(Box::new(e))
                }
            }
        }
    }

    cprintln!(!conf.quiet, "Found {} expired tweets.", to_delete.len());

    // Delete expired tweets
    //--------------------------------------------------------------------------

    if !conf.dryrun && to_delete.len() > 0 {
        let mut ctr = 0;
        for tweet in to_delete {
            if let Ok(deleted) = block_on_all(egg_mode::tweet::delete(tweet, &token)) {
                info!("Deleted: {}", deleted.text);
                ctr += 1;
            } else {
                error!("Deletion of tweet with id {} failed.", tweet);
            }
        }
        cprintln!(!conf.quiet, "Deleted {} expired tweets.", ctr);
    }

}

/// Print the error message and exit
fn unhandeled_error(error : Box<dyn std::error::Error>) -> ! {
    error!("Sorry, an unhandled error occurred.");
    debug!("{:?}", error);
    std::process::exit(1)
}

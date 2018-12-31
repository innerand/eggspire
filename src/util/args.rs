use clap::{App, Arg};
use crate::util::Conf;

const SECONDS_IN_WEEK: i64 = 604800;

pub fn get_conf() -> Conf {

    // Define Arguments
    let matches = App::new("eggspire")
        .about("\nDeletes Tweets that are expired and not faved by the authenticated user.\
                \nA Tweet has expired if it is older than a given timespan.\
                \nCAUTION: Will delete Tweets without confirmation!")
        .version("1.0.2")
        //.author("Innerand <innerand@nxa.at>")
        .arg(Arg::with_name("file")
             .long("auth-file")
             .short("a")
             .help("Path to a toml file with Twitter credentials")
             .value_name("FILE")
             .takes_value(true)
             .default_value("eggspire.toml")
        )
        .arg(Arg::with_name("span")
            .help("Timespan in weeks")
            .takes_value(true)
            .value_name("WEEKS")
            .default_value("52")
            .validator(is_nat_i64)
        )
        .arg(Arg::with_name("dryrun")
             .long("dry-run")
             .short("d")
             .help("Checks only, does not delete any Tweets")
             .takes_value(false)
        )
        .arg(Arg::with_name("quiet")
             .long("quiet")
             .short("q")
             .help("Be quiet")
             .takes_value(false)
        )
        .get_matches();

    // Get configruation from arguments
    let mut conf = Conf::new();
    conf.span = matches.value_of("span")
        .unwrap()
        .parse::<i64>()
        .unwrap() * SECONDS_IN_WEEK;
    conf.file = matches.value_of("file").unwrap().to_string();
    conf.dryrun = matches.is_present("dryrun");
    conf.quiet = matches.is_present("quiet");

    conf
}

/// Checks if string can be parsed to i64 and is greater than 0
fn is_nat_i64(s: String) -> Result<(), String> {
    match s.parse::<i64>() {
        Ok(num) if num > 0 && num < 52000 => return Ok(()),
        _ => Err(String::from("Has to be a natural number (1,2,..)")),
    }
}

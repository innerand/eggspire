# Eggspire 

Eggspire is a small tool that deletes expired tweets from your Twitter
account. 


## Usage
```bash
./eggspire --help
eggspire 1.1.0

Deletes tweets that are expired and not liked by the authenticated user and opionaly removes expired likes.
A tweet or like has expired if it is older than a given timespan.
CAUTION: Will delete tweets and remove likes without confirmation!

USAGE:
    eggspire [FLAGS] [OPTIONS] [WEEKS]

FLAGS:
    -d, --dry-run          Checks only, does not delete any tweets / likes
    -h, --help             Prints help information
    -l, --include-likes    Remove expired likes from tweets of other users
    -q, --quiet            Be quiet
    -V, --version          Prints version information

OPTIONS:
    -a, --auth-file <FILE>    Path to a toml file with Twitter credentials [default: eggspire.toml]

ARGS:
    <WEEKS>    Timespan in weeks [default: 52]
```


## Copyright
Eggspire is released under the [MIT License](/LICENSE.md).



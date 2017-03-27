# Eggspire 

Eggspire is a small application that deletes expired Tweets from your Twitter
account. (The main aim here was to gain some rust experience.) 


## Usage
```bash
./eggspire --help
eggspire 1.0.0

Deletes Tweets that are expired and not faved by the authenticated user.
A Tweet has expired if it is older than a given timespan.

CAUTION: Will delete Tweets without confirmation!

USAGE:
    eggspire [FLAGS] [OPTIONS] [<WEEKS>]

FLAGS:
    -d, --dry-run    Checks only, does not delete any Tweets
    -h, --help       Prints help information
    -q, --quiet      Be quiet
    -V, --version    Prints version information

OPTIONS:
    -a, --auth-file <FILE>    Path to a toml file with Twitter credentials
                              [default: eggspire.toml]

ARGS:
    <WEEKS>    Timespan in weeks [default: 52]

```


## Copyright
Eggspire is released under the [MIT License](/LICENSE.md).



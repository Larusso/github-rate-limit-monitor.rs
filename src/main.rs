#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate docopt;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate failure;
extern crate indicatif;

mod grlm;

const USAGE: &'static str = "
grlm - github rate limit monitor

Usage:
  grlm [(-l <user> -p <password> | -t <token>)] [-f <frequency>]
  grlm --version
  grlm -h | --help

Options:
  -l <user>, --login <user>                the github username
  -p <password>, --password <password>     the user password
  -t <token>, --access-token <token>       an github accesstoken
  -f <frequency>, --frequency <frequency>  refresh freqency [default: 10]
  -V, --version                            print version
  -h, --help                               show this help message and exit
";

fn main() {
    let options = grlm::cli::get_options(USAGE);
    if let Some(o) = options {
        grlm::monitor(o);
    }
}

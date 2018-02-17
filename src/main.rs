#[macro_use]
//Docop
extern crate serde_derive;
extern crate serde_json;
extern crate docopt;

use docopt::Docopt;
use std::{thread, time};

//Hyper
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate failure;
use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use hyper::header::UserAgent;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde_json::Value;
use failure::Error;


const USAGE: &'static str = "
grlm - github rate limit monitor

Usage:
  grlm (-l <user> -p <password> | -t <token>) [-f <frequency>]
  grlm --version
  grlm -h | --help

Options:
-l <user>, --login <user>                the github username
-p <password>, --password <password>     the user password
-t <token>, --access-token <token>        an github accesstoken
-f <frequency>, --frequency <frequency>  refresh freqency [default: 10]
-V, --version                            print version
-h, --help                               show this help message and exit
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_login: Vec<String>,
    flag_password: Vec<String>,
    flag_access_token: Vec<String>,
    flag_frequency: u64,
    flag_version: bool
}

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Serialize, Deserialize, Debug)]
struct RateLimitResult {
    resources: GithubRateLimit,
    rate: RateLimit
}

#[derive(Serialize, Deserialize, Debug)]
struct GithubRateLimit {
    core: RateLimit,
    search: RateLimit,
    graphql: RateLimit
}

#[derive(Serialize, Deserialize, Debug)]
struct RateLimit {
    limit: u32,
    remaining: u32,
    reset: u32
}

fn main() {
    let version = format!("{}.{}.{}{}",
                     env!("CARGO_PKG_VERSION_MAJOR"),
                     env!("CARGO_PKG_VERSION_MINOR"),
                     env!("CARGO_PKG_VERSION_PATCH"),
                     option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    if args.flag_version {
        println!("{:?}",version);
    } else {
        monitor(args)
    }
}

fn fetch_rate_limit() -> Result<RateLimitResult> {
    let mut core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
                 .connector(HttpsConnector::new(4, &handle)?)
                 .build(&handle);

    let uri = "https://api.github.com/rate_limit".parse()?;

    let mut req = hyper::Request::new(hyper::Method::Get, uri);
    req.headers_mut().set(UserAgent::new("curl/7.54.0"));


    let work = client.request(req).and_then(|res| {
        res.body().concat2().and_then(move |body| {
            let v: RateLimitResult = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            })?;

            return Ok(v)
        })
    });
    let result = core.run(work)?;
    Ok(result)
}

fn monitor(ref args : Args) {
    let f = args.flag_frequency;
    loop {
        println!("minitor github");
        let r = fetch_rate_limit();
        println!("{:?}", r);
        let ten_millis = time::Duration::from_secs(f);
        thread::sleep(ten_millis)
    }
}
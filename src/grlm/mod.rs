mod github;

pub mod cli;

use self::github::{AuthType, RateLimitResult, fetch_rate_limit};

use indicatif::ProgressBar;
use indicatif::ProgressDrawTarget;
use indicatif::ProgressStyle;

use parking_lot::{RwLock};

use std::thread;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct MonitorState {
    bar: ProgressBar,
    rate_limit: Option<RateLimitResult>,
    poll_frequency: u64,
    auth: AuthType,
}

pub struct Monitor {
    state: Arc<RwLock<MonitorState>>,
}

impl Monitor {
    fn new(args : cli::Options) -> Monitor {
        let f = args.frequency;
        let auth = args.auth;
        let initial_length = match auth {
            AuthType::Anonymos => 60,
            _ => 5000,
        };
        let bar = ProgressBar::new(initial_length);
        bar.set_draw_target(ProgressDrawTarget::stderr_nohz());
        bar.set_style(ProgressStyle::default_bar()
            .template(&format!("{{prefix:.bold}} {{pos}} {{wide_bar:.{}}} of {{len}} {{msg.{}}} ", "yellow", "yellow"))
            .progress_chars(" \u{15E7}\u{FF65}"));

        bar.set_prefix("Requests");
        Monitor {
            state: Arc::new(RwLock::new(MonitorState {
                bar: bar,
                rate_limit: None,
                poll_frequency: f,
                auth: auth,
            })),
        }
    }

    pub fn start_ticker(&self) {
        let tick = Duration::from_secs(self.state.read().poll_frequency);
        let mut instant = Instant::now();
        let mut first_run = true;
        loop {
            if first_run || instant.elapsed() >= tick
            {
                first_run = false;
                let mut state = self.state.write();
                match fetch_rate_limit(&state.auth) {
                    Ok(r) => state.rate_limit = Some(r),
                    Err(e) => println!("Error {}", e),
                }
                instant = Instant::now();
            }
            if let Some(ref r) = self.state.read().rate_limit {
                let ref bar = self.state.read().bar;
                bar.set_length(r.rate.limit);
                bar.set_message(&format!("{}",r.rate.resets_in()));
                bar.set_position(r.rate.limit - r.rate.remaining);
                bar.set_style(ProgressStyle::default_bar()
                   .template(&format!("{{prefix:.bold}} {{pos:.{}}} {{wide_bar:.{}}} of {{len}} resets in {{msg:.{}}} ", self.rate_color(), "yellow", self.message_color()))
                   .progress_chars(self.progress_chars()));
            }
            thread::sleep(Duration::from_millis(1000/30));
        }
    }

    pub fn start(args : cli::Options) {
        let m = Monitor::new(args);
        m.start_ticker();
    }

    fn progress_chars(&self) -> &'static str {
        if let Some(ref r) = self.state.read().rate_limit {
            match r.rate.remaining {
                x if x == 0 => "#####",
                _ => " \u{15E7}\u{FF65}",
            }
        }
        else {
            " \u{15E7}\u{FF65}"
        }
    }

    fn rate_color(&self) -> &'static str {
        if let Some(ref r) = self.state.read().rate_limit {
            match (r.rate.remaining as f64) / (r.rate.limit as f64) {
                x if x <= 0.08 => "red",
                x if x <= 0.5 => "yellow",
                _ => "green"
            }
        }
        else {
            "white"
        }
    }

    fn message_color(&self) -> &'static str {
        if let Some(ref r) = self.state.read().rate_limit {
            match r.rate.resets_in() {
                x if x < 120 => "green",
                _ => "white"
            }
        }
        else {
            "white"
        }
    }
}
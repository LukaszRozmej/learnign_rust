extern crate timer;
extern crate chrono;

use reqwest::blocking::Client;
use reqwest::header::{ETAG, HeaderValue, IF_NONE_MATCH};
use crate::BlocklistStore;
use timer::Timer;
use timer::Guard;
use std::sync::{Arc, Mutex, MutexGuard};
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct BlocklistDownloader {
    timer : Timer,
    timer_guards: (Guard, Guard),
}

struct Downloader<T : BlocklistStore> {
    e_tag: String,
    store : Arc<Mutex<T>>,
    client: Client
}

impl BlocklistDownloader {
    pub fn new<T : BlocklistStore + Sync + Send + 'static>(store : Arc<Mutex<T>>) -> Self
    {
        let config = Config {interval: 600, url: String::from("https://raw.githubusercontent.com/stamparm/ipsum/master/ipsum.txt")};
        let client = Client::builder().gzip(true).brotli(true).deflate(true).build().unwrap();
        let downloader = Downloader { e_tag: String::new(), store, client };
        let downloader = Arc::new(Mutex::new(downloader));
        let timer = Timer::new();

        // non-blocking first refresh
        let guard = {
            let downloader_instant = downloader.clone();
            let url = config.url.clone();
            let guard_instant = timer.schedule_with_delay(chrono::Duration::microseconds(1), move || {
                refresh(downloader_instant.lock().unwrap(), &url);
            });

            let downloader = downloader.clone();
            let url = config.url.clone();
            let guard_repeat = timer.schedule_repeating(chrono::Duration::seconds(config.interval), move || {
                refresh(downloader.lock().unwrap(), &url);
            });

            (guard_instant, guard_repeat)
        };

        // blocking first refresh
        // refresh(downloader.lock().unwrap(), &config.url);

        Self {
            timer,
            timer_guards: guard
        }
    }
}

fn refresh<T : BlocklistStore>(mut downloader: MutexGuard<Downloader<T>>, url: &str) {
    let response = downloader.client
        .get(url)
        .header(IF_NONE_MATCH, &downloader.e_tag)
        .send();

    match response {
        Ok(response) => {
            let status = response.status();
            let e_tag = response.headers()
                .get(ETAG)
                .unwrap_or(&HeaderValue::from_str("").unwrap())
                .to_string();
            let text = response.text().unwrap_or(String::from(""));
            if status.is_success() {
                if e_tag != "" {
                    downloader.e_tag = e_tag;
                }
                parse(text, &mut downloader.store);
            }
            else {
                log::info!("Server response wasn't successful.")
            }
        },
        Err(error) => log::error!("Request for new IPs failed {}", error)
    }
}

fn parse<T : BlocklistStore>(ips : String, store: &mut Arc<Mutex<T>>) {
    let x = ips.split("\n")
        .filter(|s| !s.starts_with("#"))
        .map(|s| s.split_whitespace().next().unwrap_or(""))
        .filter(|s| *s != "")
        .map(|s| Ipv4Addr::from_str(s))
        .filter(|a| a.is_ok())
        .map(|a| a.unwrap_or(Ipv4Addr::UNSPECIFIED))
        .map(|a| u32::from(a));

    store.lock().unwrap().set_addresses(x);
}

pub trait HeaderValueExt {
    fn to_string(&self) -> String;
}

impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> String {
        self.to_str().unwrap_or_default().to_string()
    }
}

struct Config {
    url: String,
    interval: i64,
}
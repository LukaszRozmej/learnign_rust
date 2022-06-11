use reqwest::{Client, Response, StatusCode};
use reqwest::header::{ETAG, HeaderValue, IF_NONE_MATCH};
use crate::BlocklistStore;
use std::sync::Arc;
use std::net::Ipv4Addr;
use std::str::FromStr;
use futures::AsyncBufReadExt;
use futures::stream::TryStreamExt;
use tokio::time;
use tokio::time::Duration;
use tokio::sync::{Mutex, MutexGuard};


struct Downloader<T : BlocklistStore> {
    e_tag: String,
    store : Arc<T>,
    client: Client
}

pub fn start<T : BlocklistStore + Sync + Send + 'static>(store : Arc<T>)
{
    let config = Config { interval: 60000, url: String::from("https://raw.githubusercontent.com/stamparm/ipsum/master/ipsum.txt") };
    let client = Client::builder().gzip(true).brotli(true).deflate(true).build().unwrap();
    let downloader = Downloader { e_tag: String::new(), store, client };
    let downloader = Arc::new(Mutex::new(downloader));

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(config.interval));
        loop {
            refresh(downloader.clone().lock().await, &config.url).await;
            interval.tick().await;
        }
    });
}

fn convert_err(_: reqwest::Error) -> std::io::Error { todo!() }

async fn refresh<T : BlocklistStore>(mut downloader: MutexGuard<'_, Downloader<T>>, url: &str) {
    let response = downloader.client
        .get(url)
        .header(IF_NONE_MATCH, &downloader.e_tag)
        .send()
        .await;

    match response {
        Ok(response) => {
            let status = response.status();
            let e_tag = response.headers()
                .get(ETAG)
                .unwrap_or(&HeaderValue::from_str("").unwrap())
                .to_string();

            if status.is_success() {
                if e_tag != "" {
                    downloader.e_tag = e_tag;
                }

                parse(response, &mut downloader.store).await;
            }
            else if status != StatusCode::NOT_MODIFIED {
                log::info!("Server response wasn't successful.")
            }
        },
        Err(error) => log::error!("Request for new IPs failed {}", error)
    }
}

async fn parse<T: BlocklistStore>(response: Response, store: &mut Arc<T>) {
    let mut chunks = response
        .bytes_stream()
        .map_err(convert_err)
        .into_async_read();

    let mut buffer = String::new();
    let mut values = Vec::new();
    let mut length = chunks.read_line(&mut buffer).await.unwrap_or(0);
    while length > 0  {
        if !buffer.starts_with("#") {
            if let Some(first) = buffer.split_whitespace().next() {
                if let Ok(address) = Ipv4Addr::from_str(first) {
                    values.push(address);
                }
            }
        }

        buffer.clear();
        length = chunks.read_line(&mut buffer).await.unwrap_or(0);
    }

    store.set_addresses(values.iter().map(|a| *a));
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
    interval: u64,
}
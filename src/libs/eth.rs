use std::{collections::BTreeMap, sync::LazyLock};

use ahash::HashSet;
use parking_lot::RwLock;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;

use super::constants::{ETH_INTERNAL, ETH_TIMEOUT, ETH_URL};

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Deserialize)]
pub struct Block {
    pub hash: String,
    pub height: u32,
    pub time: u64,
}

type Set = HashSet<u32>;
type Map = BTreeMap<u32, Block>;

pub static BLOCKS: RwLock<Map> = RwLock::new(Map::new());
pub static BAN: LazyLock<RwLock<Set>> = LazyLock::new(|| RwLock::new(Set::default()));

pub fn fetch() -> Option<Block> {
    let blocks = BLOCKS.read();
    let mut ban = BAN.write();

    for (height, block) in blocks.iter().rev() {
        if !ban.contains(height) {
            ban.insert(*height);
            return Some(block.clone());
        }
    }

    tracing::warn!(target: "eth-request", "Blocks are exhausted!");
    None
}

pub async fn fetcher() {
    #[derive(Debug, Deserialize)]
    struct EthRespPageBlocks<'a> {
        hash: &'a str,
        number: &'a str,
        timestamp: &'a str,
    }
    #[derive(Deserialize)]
    struct EthRespPageProps<'a> {
        #[serde(borrow)]
        latestBlocks: Vec<EthRespPageBlocks<'a>>,
    }
    #[derive(Deserialize)]
    struct EthRespProps<'a> {
        #[serde(borrow)]
        pageProps: EthRespPageProps<'a>,
    }
    #[derive(Deserialize)]
    struct EthResp<'a> {
        #[serde(borrow)]
        props: EthRespProps<'a>,
    }

    let selector = Selector::parse("#__NEXT_DATA__").unwrap();

    loop {
        tracing::info!(target: "eth-fetcher", "Fetching ETH blocks");

        'a: {
            let data: reqwest::Result<Html> = try {
                let client = Client::builder().connect_timeout(ETH_TIMEOUT).build()?;
                let res = client.get(ETH_URL).send().await?.text().await?;
                Html::parse_document(&res)
            };
            let html = match data {
                Ok(html) => html,
                Err(e) => {
                    tracing::warn!(target: "eth-fetcher", "Failed to fetch ETH blocks: {e:?}");
                    break 'a;
                }
            };
            let Some(element) = html.select(&selector).next() else {
                tracing::warn!(target: "eth-fetcher", "Failed to find __NEXT_DATA__ element");
                break 'a;
            };
            let Some(text) = element.text().next() else {
                tracing::warn!(target: "eth-fetcher", "Failed to find __NEXT_DATA__ text");
                break 'a;
            };
            let EthResp {
                props:
                    EthRespProps {
                        pageProps: EthRespPageProps { latestBlocks },
                    },
            } = match serde_json::from_str(text) {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::warn!(target: "eth-fetcher", "Failed to parse __NEXT_DATA__ json: {e:?}");
                    break 'a;
                }
            };
            let mut guard = BLOCKS.write();
            for raw_block in latestBlocks {
                let block: Option<Block> = try {
                    let hash = raw_block.hash.get(2..)?.to_owned();
                    let height = raw_block.number.parse().ok()?;
                    let time = raw_block.timestamp.parse().ok()?;
                    Block { hash, height, time }
                };
                if let Some(block) = block {
                    guard.insert(block.height, block);
                }
            }
            tracing::info!(target: "eth-fetcher", "{} blocks in total, latest: {}", guard.len(), guard.last_key_value().unwrap().0);
        }

        tokio::time::sleep(ETH_INTERNAL).await;
    }
}

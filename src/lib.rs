use std::time::{SystemTime, UNIX_EPOCH};

use http_req::request;
use schedule_flows::schedule_cron_job;
use serde_derive::{Deserialize, Serialize};

use slack_flows::send_message_to_channel;

#[no_mangle]
pub fn run() {
    let keyword = std::env::var("KEYWORD").unwrap();

    schedule_cron_job(String::from("50 * * * *"), keyword, callback);
}

fn callback(keyword: Vec<u8>) {
    let kw = String::from_utf8(keyword).unwrap();

    let now = SystemTime::now();
    let dura = now.duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600;

    let kws = kw.split("||");

    let mut lists = Vec::new();
    for kw in kws {
        let url = format!("https://hn.algolia.com/api/v1/search_by_date?tags=story&query={kw}&numericFilters=created_at_i>{dura}");

        let mut writer = Vec::new();
        let resp = request::get(url, &mut writer).unwrap();

        if resp.status_code().is_success() {
            let search: Search = serde_json::from_slice(&writer).unwrap();

            let hits = search.hits;
            let list = hits
                .iter()
                .map(|hit| {
                    let title = &hit.title;
                    let url = &hit.url.clone().unwrap_or_default();
                    let author = &hit.author;

                    format!("- *{title}*\n<{url}|source> by {author}\n")
                })
                .collect::<Vec<_>>();
            lists.push(list);
        }
    }

    let mut lists = lists.into_iter().flatten().collect::<Vec<_>>();
    lists.sort();
    lists.dedup();
    let list = lists.into_iter().collect::<String>();

    let msg = format!(":sparkles: {kw} :sparkles:\n{list}");
    send_message_to_channel("ham-5b68442", "general", msg);
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Search {
    pub hits: Vec<Hit>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub title: String,
    pub url: Option<String>,
    pub author: String,
    #[serde(rename = "created_at_i")]
    pub created_at_i: i64,
}

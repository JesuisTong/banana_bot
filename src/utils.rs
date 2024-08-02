use chrono::Local;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, CONTENT_TYPE, ORIGIN, PRAGMA,
    REFERER, REFERRER_POLICY, USER_AGENT,
};

pub fn now() -> String {
    Local::now().format("%F %T").to_string()
}

pub fn get_current_timestamp() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis() as i64
}

pub fn format_println(name: &str, msg: &str) {
    info!("[{}] [{}]: {}", now(), name, msg);
}
pub fn format_error(name: &str, msg: &str) {
    error!("[{}] [{}]: {}", now(), name, msg);
}

pub fn init_headers(h: &mut HeaderMap) -> &mut HeaderMap {
    h.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, */*"),
    );
    h.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6"),
    );
    h.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    h.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    h.insert(REFERER, HeaderValue::from_static("https://banana.carv.io/"));
    h.insert(ORIGIN, HeaderValue::from_static("https://banana.carv.io"));
    h.insert("priority", HeaderValue::from_static("u=1, i"));
    h.insert("sec-ch-ua", HeaderValue::from_static("\"\""));
    h.insert("sec-ch-ua-mobile", HeaderValue::from_static("?1"));
    h.insert("sec-ch-ua-platform", HeaderValue::from_static("\"\""));
    h.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    h.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    h.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    h.insert("x-app-id", HeaderValue::from_static("carv"));
    h.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    h.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1 Edg/126.0.0.0"));

    h
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub link: Option<String>,
    pub access_token: Option<String>,
    pub cookie_token: Option<String>,
    pub invite_code: Option<String>,
}

impl Clone for User {
    fn clone(&self) -> Self {
        User {
            link: self.link.clone(),
            access_token: self.access_token.clone(),
            cookie_token: self.cookie_token.clone(),
            invite_code: self.invite_code.clone(),
        }
    }
}

pub fn read_config_json(file_path: &str) -> HashMap<String, User> {
    let file = fs::File::open(file_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let hashmap: HashMap<String, User> =
        serde_json::from_reader(reader).expect("Unable to parse JSON");
    hashmap
}

pub fn write_config_json(file_path: &str, data: &HashMap<String, User>) {
    let json_data = serde_json::to_string_pretty(data).expect("Unable to serialize data");
    let mut file = fs::File::create(file_path).expect("Unable to create file");
    file.write_all(json_data.as_bytes())
        .expect("Unable to write data to file");
}

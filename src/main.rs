use colog;
use log::info;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

mod utils;

use utils::User;

#[derive(Serialize, Deserialize, Debug)]
struct TapData {
    number_gem: f32,
    number_ec: i32,
    level: i32,
    base_rate: f32,
    min_ec: i32,
    number_tap: i64,
}

#[derive(Debug)]
enum BananaErr {
    LoginErr,
    GetUserInfoErr,
}

impl Display for BananaErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BananaErr {}

struct Banana {
    name: String,
    access_token: String,
    cookie_token: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct LotteryInfo {
    countdown_end: bool,
    countdown_interval: i32,
    last_countdown_start_time: i64,
}

#[derive(Deserialize, Serialize, Clone)]
struct BananaUserInfo {
    username: String,
    lottery_info: LotteryInfo,
    max_click_count: i32,
    today_click_count: i32,
}

impl Banana {
    fn new(name: String, access_token: String, cookie_token: String) -> Self {
        Self {
            name,
            access_token,
            cookie_token,
        }
    }

    async fn get_user_info(&self) -> Result<BananaUserInfo, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        utils::init_headers(&mut headers);
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookie_token).unwrap());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", &self.access_token)).unwrap(),
        );

        let response = client
            .get("https://interface.carv.io/banana/get_user_info")
            .headers(headers)
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            let bui: serde_json::Value =
                serde_json::from_str(&response.text().await?).map_err(|err| {
                    utils::format_error(&self.name, &format!("err: {:?}", err));
                    Box::new(BananaErr::GetUserInfoErr)
                })?;
            if bui["code"] == 0 {
                let bui: BananaUserInfo = serde_json::from_value(bui["data"].clone()).unwrap();
                return Ok(bui);
            }
        }
        utils::format_error(&self.name, "get_user_info failed");
        Err(Box::new(BananaErr::GetUserInfoErr))
    }

    async fn do_click(
        &self,
        max_click_count: i32,
        today_click_count: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        utils::init_headers(&mut headers);
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookie_token).unwrap());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", &self.access_token)).unwrap(),
        );

        let mut rest_count = max_click_count - today_click_count;

        loop {
            if rest_count <= 0 {
                break;
            }

            let mut rand_num = rest_count;
            let mut rng = rand::thread_rng();
            if rand_num > 10 {
                rand_num = rng.gen_range(10..rand_num);
            }

            let response = client
                .post("https://interface.carv.io/banana/do_click")
                .headers(headers.clone())
                .body(
                    json!({
                        "clickCount": rand_num,
                    })
                    .to_string(),
                )
                .send()
                .await?;

            if response.status() == StatusCode::OK {
                let s: serde_json::Value = serde_json::from_str(&response.text().await?)?;
                if s["code"] == 0 {
                    rest_count -= rand_num;
                }
                utils::format_println(&self.name, &format!("click: {:?}", s));
                sleep(Duration::from_millis(rng.gen_range(500..3000))).await;
            }
        }

        utils::format_println(&self.name, "click done!");

        Ok(())
    }

    async fn claim(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        utils::init_headers(&mut headers);
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookie_token).unwrap());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", &self.access_token)).unwrap(),
        );

        let response = client
            .post("https://interface.carv.io/banana/claim_lottery")
            .headers(headers)
            .body(
                json!({
                    "claimLotteryType": 1
                })
                .to_string(),
            )
            .send()
            .await?;

        utils::format_println(&self.name, &format!("claim: {:?}", response.status()));

        Ok(())
    }

    // async fn post_task() -> Result {}

    // async fn get_task_list() -> Result {}
}

// https://banana.carv.io/#tgWebAppData=query_id%3DAAEx6mk6AgAAADHqaTqBh_xK%26user%3D%257B%2522id%2522%253A5274987057%252C%2522first_name%2522%253A%2522tong%2522%252C%2522last_name%2522%253A%2522zag%2522%252C%2522username%2522%253A%2522TongZag%2522%252C%2522language_code%2522%253A%2522zh-hans%2522%252C%2522allows_write_to_pm%2522%253Atrue%257D%26auth_date%3D1722502397%26hash%3Db6ae4585e9a451766f95b42ab8db76c2a2c37f7c3dc3d59082d522804c659fee&tgWebAppVersion=7.6&tgWebAppPlatform=web&tgWebAppThemeParams=%7B%22bg_color%22%3A%22%23212121%22%2C%22button_color%22%3A%22%238774e1%22%2C%22button_text_color%22%3A%22%23ffffff%22%2C%22hint_color%22%3A%22%23aaaaaa%22%2C%22link_color%22%3A%22%238774e1%22%2C%22secondary_bg_color%22%3A%22%23181818%22%2C%22text_color%22%3A%22%23ffffff%22%2C%22header_bg_color%22%3A%22%23212121%22%2C%22accent_text_color%22%3A%22%238774e1%22%2C%22section_bg_color%22%3A%22%23212121%22%2C%22section_header_text_color%22%3A%22%238774e1%22%2C%22subtitle_text_color%22%3A%22%23aaaaaa%22%2C%22destructive_text_color%22%3A%22%23ff595a%22%7D
// https://banana.carv.io/?tgWebAppStartParam=referral%3DHHQJ6T4#tgWebAppData=user%3D%257B%2522id%2522%253A5274987057%252C%2522first_name%2522%253A%2522tong%2522%252C%2522last_name%2522%253A%2522zag%2522%252C%2522username%2522%253A%2522TongZag%2522%252C%2522language_code%2522%253A%2522zh-hans%2522%252C%2522allows_write_to_pm%2522%253Atrue%257D%26chat_instance%3D-368903118116780412%26start_param%3Dreferral%253DHHQJ6T4%26auth_date%3D1722501792%26hash%3Dfe267034a5b2e82b6dff1290b96c67bccaba4a3d53fe917ba906f645f67b9787&tgWebAppVersion=7.6&tgWebAppPlatform=web&tgWebAppThemeParams=%7B%22bg_color%22%3A%22%23212121%22%2C%22button_color%22%3A%22%238774e1%22%2C%22button_text_color%22%3A%22%23ffffff%22%2C%22hint_color%22%3A%22%23aaaaaa%22%2C%22link_color%22%3A%22%238774e1%22%2C%22secondary_bg_color%22%3A%22%23181818%22%2C%22text_color%22%3A%22%23ffffff%22%2C%22header_bg_color%22%3A%22%23212121%22%2C%22accent_text_color%22%3A%22%238774e1%22%2C%22section_bg_color%22%3A%22%23212121%22%2C%22section_header_text_color%22%3A%22%238774e1%22%2C%22subtitle_text_color%22%3A%22%23aaaaaa%22%2C%22destructive_text_color%22%3A%22%23ff595a%22%7D
async fn login(
    tg_url: &str,
    invite_code: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // let
    let url = Url::parse(tg_url)?;
    let f = url.fragment();
    if let Some(f) = f {
        let v = f.split('&').nth(0).unwrap();
        let v = v.split('=').nth(1).unwrap();
        let s = urlencoding::decode(v)?;
        let body = json!({
            "tgInfo": s.to_string(),
            "InviteCode": invite_code,
        });

        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        utils::init_headers(&mut headers);

        let response = client
            .post("https://interface.carv.io/banana/login")
            .headers(headers.clone())
            .body(body.to_string())
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            let game_user_token = response
                .headers()
                .get_all("set-cookie")
                .iter()
                .filter_map(|v| {
                    let vv = v.to_str().expect("set cookie is not a string");
                    match regex::Regex::new(r"banana-game:user:token")
                        .unwrap()
                        .is_match(vv)
                    {
                        true => Some(vv),
                        false => None,
                    }
                })
                .next()
                .unwrap();
            let ck = cookie::Cookie::parse(game_user_token).unwrap();
            let (name, value) = ck.name_value();
            let name_value = name.to_owned() + value;

            let val: serde_json::Value = serde_json::from_str(&response.text().await?).unwrap();
            let token = val["data"]["token"].as_str().unwrap();

            return Ok((token.to_string(), name_value));
        }
    }

    Err(Box::new(BananaErr::LoginErr))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    colog::init();

    // read user token from file
    let file_path = path::PathBuf::from(std::env::current_dir().unwrap()).join("user.json");
    info!("file_path: {:?}", file_path);
    let users = utils::read_config_json(file_path.to_str().unwrap());
    let mut copy_users: HashMap<String, User> = HashMap::new();

    for (name, mut user) in users {
        if user.access_token.is_none() || user.cookie_token.is_none() {
            let default_invite_code = "".to_string();
            let invite_code = user.invite_code.as_ref().unwrap_or(&default_invite_code);
            let (access_token, cookie_token) = login(&user.link.as_ref().unwrap(), &invite_code)
                .await
                .unwrap();
            user.access_token = Some(access_token);
            user.cookie_token = Some(cookie_token);
        }

        // update json
        copy_users.insert(name.clone().to_string(), user.clone());
        utils::write_config_json(file_path.to_str().unwrap(), &copy_users);

        info!("name: {}, start", &name);

        let user = Banana::new(
            name.clone(),
            user.access_token.unwrap(),
            user.cookie_token.unwrap(),
        );

        let userinfo = user.get_user_info().await.unwrap();

        user.do_click(userinfo.max_click_count, userinfo.today_click_count)
            .await
            .unwrap();

        let can_claim_time = userinfo.lottery_info.last_countdown_start_time
            + (userinfo.lottery_info.countdown_interval as i64 * 60000);
        let rest_time = can_claim_time - utils::get_current_timestamp();

        utils::format_println(&name, &format!("rest_time: {}", rest_time));

        let arc_user = Arc::new(user);
        tokio::spawn(async move {
            if rest_time <= 0 {
                let _ = arc_user.claim().await.map_err(|err| {
                    utils::format_error(&name, &format!("err: {:?}", err));
                });
            } else {
                sleep(Duration::from_millis(rest_time as u64 + 1000u64)).await;
                let _ = arc_user.claim().await.map_err(|err| {
                    utils::format_error(&name, &format!("err: {:?}", err));
                });
            }
            loop {
                sleep(Duration::from_secs(60 * 60 * 8)).await;
                let _ = arc_user.claim().await.map_err(|err| {
                    utils::format_error(&name, &format!("err: {:?}", err));
                });
            }
        });
    }

    // TODO: use another way to keep the program running
    // at most 7 days
    sleep(Duration::from_secs(60 * 60 * 24 * 7)).await;

    Ok(())
}

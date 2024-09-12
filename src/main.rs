use colog;
use colored::*;
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
    QuestListErr,
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
    remain_lottery_count: i32,
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

    fn request(&self) -> (reqwest::Client, HeaderMap) {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        utils::init_headers(&mut headers);
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookie_token).unwrap());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", &self.access_token)).unwrap(),
        );

        (client, headers)
    }

    async fn get_user_info(&self) -> Result<BananaUserInfo, Box<dyn std::error::Error>> {
        let (client, headers) = self.request();

        let response = client
            .get("https://interface.carv.io/banana/get_user_info")
            .headers(headers)
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            let bui: serde_json::Value = response.json().await.map_err(|err| {
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
        let (client, headers) = self.request();

        let mut rest_count = max_click_count - today_click_count;

        loop {
            if rest_count <= 0 {
                self.claim_ads_income(0).await?;
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
        utils::format_println(&self.name, "claim start!");

        let (client, headers) = self.request();

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

        utils::format_println(&self.name, &format!("claim done!: {:?}", response.status()));

        Ok(())
    }

    async fn do_lottery(&self) -> Result<(), Box<dyn std::error::Error>> {
        let userinfo = self.get_user_info().await?;

        if userinfo.lottery_info.remain_lottery_count <= 0 {
            return Ok(());
        }

        let (client, headers) = self.request();
        let mut cnt = userinfo.lottery_info.remain_lottery_count;

        while cnt > 0 {
            let response = client
                .post("https://interface.carv.io/banana/do_lottery")
                .headers(headers.clone())
                .body("{}")
                .send()
                .await?;

            let status = response.status();
            utils::format_println(&self.name, &format!("do_lottery: {:?}", status));

            if status != StatusCode::OK {
                break;
            }

            let result = response.json::<serde_json::Value>().await?;
            if result["code"].as_i64().unwrap() != 0 {
                break;
            }

            utils::format_println(
                &self.name,
                &format!(
                    "id: {}\nname: {:?}\nrarity: {:?}",
                    result["data"]["banana_id"].as_i64().unwrap(),
                    result["data"]["name"].as_str().unwrap(),
                    result["data"]["ripeness"].as_str().unwrap()
                ),
            );

            sleep(Duration::from_millis(500)).await;
            self.do_share(result["data"]["banana_id"].as_i64().unwrap())
                .await?;
            sleep(Duration::from_millis(1000)).await;
            self.claim_ads_income(2).await?;
            cnt -= 1;
        }

        utils::format_println(&self.name, "harvest done!");

        Ok(())
    }

    async fn do_share(&self, banana_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let (client, headers) = self.request();

        let response = client
            .post("https://interface.carv.io/banana/do_share")
            .headers(headers)
            .body(
                json!({
                    "banana_id": banana_id
                })
                .to_string(),
            )
            .send()
            .await?;

        utils::format_println(
            &self.name,
            &format!("do_share done!: {:?}", response.status()),
        );

        Ok(())
    }

    async fn complete_quest(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (client, headers) = self.request();
        let quest_list_resp = client
            .get("https://interface.carv.io/banana/get_quest_list")
            .headers(headers.clone())
            .send()
            .await?;

        if quest_list_resp.status() != StatusCode::OK {
            utils::format_error(&self.name, "get_quest_list failed");
            return Err(Box::new(BananaErr::QuestListErr));
        }

        let quest_list: serde_json::Value =
            serde_json::from_str(&quest_list_resp.text().await?).unwrap();
        let quest_list: Vec<serde_json::Value> =
            serde_json::from_value(quest_list["data"]["quest_list"].clone()).unwrap();

        for quest in quest_list.iter().filter(|x| {
            let is_achieved = x["is_achieved"].as_bool().unwrap();
            let is_claimed = x["is_claimed"].as_bool().unwrap();
            !is_achieved && !is_claimed
        }) {
            match quest["quest_type"].as_str().unwrap() {
                "carv_ios_app"
                | "carv_android_app"
                | "retweet_tweet"
                | "like_tweet"
                | "follow_on_twitter"
                | "visit_page"
                | "telegram_join_group" => {
                    let quest_id = quest["quest_id"].as_i64().unwrap();
                    let body = json!({
                        "quest_id": quest_id
                    });
                    let response = client
                        .post("https://interface.carv.io/banana/achieve_quest")
                        .headers(headers.clone())
                        .body(body.to_string())
                        .send()
                        .await?;

                    utils::format_println(
                        &self.name,
                        &format!("achieve quest {}: {:?}", quest_id, response.status()),
                    );

                    sleep(Duration::from_secs(1)).await;
                    let response = client
                        .post("https://interface.carv.io/banana/claim_quest")
                        .headers(headers.clone())
                        .body(body.to_string())
                        .send()
                        .await?;

                    utils::format_println(
                        &self.name,
                        &format!("claim quest {}: {:?}", quest_id, response.status()),
                    );
                }
                _ => {}
            };
            sleep(Duration::from_secs(2)).await;
        }

        utils::format_println(&self.name, "complete_quest done!");

        Ok(())
    }

    async fn do_speedup(&self) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        let (client, headers) = self.request();

        let response = client
            .post("https://interface.carv.io/banana/do_speedup")
            .headers(headers)
            .body("{}")
            .send()
            .await?;

        utils::format_println(&self.name, &format!("do_speedup: {:?}", response.status()));

        if response.status() == StatusCode::OK {
            let data = &response.json::<serde_json::Value>().await?;
            let code = data["code"].as_i64().unwrap();
            if code == 0i64 {
                let can_claim_time = data["data"]["lottery_info"]["last_countdown_start_time"]
                    .as_i64()
                    .unwrap()
                    + (data["data"]["lottery_info"]["countdown_interval"]
                        .as_i64()
                        .unwrap()
                        * 60000);
                let rest_time = can_claim_time - utils::get_current_timestamp();

                self.claim_ads_income(1).await?;

                return Ok(Some(rest_time));
            }
        }

        Ok(None)
    }

    async fn loop_claim_quest_lottery(&self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(true) = self.get_quest_is_claimed().await? {
            self.claim_quest_lottery().await?;
            sleep(Duration::from_secs(1)).await;
        }
        utils::format_println(&self.name, "loop_claim_quest_lottery done!");
        Ok(())
    }

    async fn get_quest_is_claimed(&self) -> Result<Option<bool>, Box<dyn std::error::Error>> {
        let (client, headers) = self.request();

        let response = client
            .get("https://interface.carv.io/banana/get_quest_list")
            .headers(headers)
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            let data = response.json::<serde_json::Value>().await?;
            let code = data["code"].as_i64().expect("code is not a number");
            if code == 0i64 {
                let is_claimed = data["data"]["is_claimed"].as_bool().unwrap();

                return Ok(Some(is_claimed));
            }
        }

        Ok(None)
    }

    async fn claim_quest_lottery(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (client, headers) = self.request();

        client
            .post("https://interface.carv.io/banana/claim_quest_lottery")
            .headers(headers)
            .body("{}")
            .send()
            .await?;

        Ok(())
    }

    async fn claim_ads_income(&self, income_type: u8) -> Result<(), Box<dyn std::error::Error>> {
        let (client, headers) = self.request();

        let response = client
            .post("https://interface.carv.io/banana/claim_ads_income")
            .headers(headers)
            .body(
                json!({
                    "type": &income_type
                })
                .to_string(),
            )
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::OK {
            let response = response.json::<serde_json::Value>().await?;
            if response["code"].as_i64().unwrap() == 0 {
                utils::format_println(
                    &self.name,
                    &format!(
                        "claim_ads_income_{}: {:?}",
                        income_type,
                        response["data"]["income"].as_f64().unwrap()
                    ),
                );
            } else {
                utils::format_error(
                    &self.name,
                    &format!(
                        "claim_ads_income_{} failed: {:?}",
                        income_type,
                        response.to_string()
                    ),
                );
            }
        } else {
            utils::format_error(
                &self.name,
                &format!("claim_ads_income_{} failed", income_type),
            );
        }

        Ok(())
    }
}

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
    println!(
        "Welcome to Banana Bot 🍌\nFree your hands now!\n\nOfficial website: {}",
        "https://t.me/OfficialBananaBot/banana?startapp=referral=HHQJ6T4".yellow()
    );
    info!("file_path: {:?}", file_path);
    let users = utils::read_config_json(file_path.to_str().unwrap());
    let mut copy_users: HashMap<String, User> = users.clone();

    for (name, mut user) in users {
        if user.access_token.is_none() || user.cookie_token.is_none() {
            let default_invite_code = "".to_string();
            let invite_code = user.invite_code.as_ref().unwrap_or(&default_invite_code);
            let (access_token, cookie_token) = login(&user.link.as_ref().unwrap(), &invite_code)
                .await
                .unwrap();
            user.access_token = Some(access_token);
            user.cookie_token = Some(cookie_token);
            // overwrite user config
            copy_users.insert(name.clone().to_string(), user.clone());
            utils::write_config_json(file_path.to_str().unwrap(), &copy_users);
        }

        info!("name: {}, start", &name);

        let user = Banana::new(
            name.clone(),
            user.access_token.unwrap(),
            user.cookie_token.unwrap(),
        );

        let userinfo = user
            .get_user_info()
            .await
            .expect(&format!("{} get_user_info failed", &name));

        user.do_click(userinfo.max_click_count, userinfo.today_click_count)
            .await
            .unwrap();

        user.complete_quest().await.unwrap();
        user.loop_claim_quest_lottery().await.unwrap();

        let arc_user = Arc::new(user);

        tokio::spawn(async move {
            let can_claim_time = userinfo.lottery_info.last_countdown_start_time
                + (userinfo.lottery_info.countdown_interval as i64 * 60000);
            let rest_time = can_claim_time - utils::get_current_timestamp();

            utils::format_println(
                &name,
                &format!("next claim is after: {}secs", (rest_time.max(0) / 1000) | 0),
            );

            if rest_time > 0 {
                sleep(Duration::from_millis(rest_time as u64 + 1000u64)).await;
            }
            let _ = arc_user.claim().await.map_err(|err| {
                utils::format_error(&name, &format!("err: {:?}", err));
            });

            loop {
                let do_speedup_res = arc_user.do_speedup().await.unwrap();

                let rest_time = match do_speedup_res {
                    Some(rest_time) => rest_time / 1000 + 10,
                    None => 60 * 60 * 8 + 10,
                } as u64;

                utils::format_println(
                    &name,
                    &format!("next claim is after: {}secs", rest_time.max(0)),
                );
                sleep(Duration::from_secs(rest_time)).await;

                arc_user
                    .claim()
                    .await
                    .map_err(|err| {
                        utils::format_error(&name, &format!("claim err: {:?}", err));
                    })
                    .ok();
                arc_user
                    .do_lottery()
                    .await
                    .map_err(|err| {
                        utils::format_error(&name, &format!("do_lottery err: {:?}", err));
                    })
                    .ok();
            }
        });

        sleep(Duration::from_secs(1)).await;
    }

    // at most 7 days
    sleep(Duration::from_secs(60 * 60 * 24 * 7)).await;

    Ok(())
}

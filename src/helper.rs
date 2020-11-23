use slack;
use slack::RtmClient;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct PostParams {
    pub subject: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct PostResponse {
    pub message: String,
    pub ok: bool,
}

pub fn get_bot_id(cli: &RtmClient, bot_name: &String) -> String {
    cli
    .start_response()
    .users
    .as_ref()
    .and_then(|users| {
        users.iter().find(|user| match &user.name {
            None => false,
            Some(n) => n == bot_name,
        })
    })
    .and_then(|user| user.id.as_ref())
    .expect("the bot not found")
    .to_string()
}

pub fn send_msg(cli: &RtmClient, channel_id: &String, msg: &str) -> () {
    let _ = cli
        .sender()
        .send_message(channel_id, msg);
}

pub fn post(endpoint: &String, data: &PostParams) -> Result<PostResponse, String> {
    let cli = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    let res = cli
        .post(endpoint)
        .headers(headers)
        .json(data)
        .send();

    let res_raw =  match res {
        Ok(mut res) => res.json(),
        Err(e) => return Err(format!("{:?}", e)),
    };

    match res_raw {
        Ok(res) => Ok(res),
        Err(e) => return Err(format!("{:?}", e))
    }
}

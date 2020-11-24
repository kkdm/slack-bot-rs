use env_logger;
use log::{error, warn, info, debug};
use std::env;
use std::process;
use slack;
use slack::{RtmClient, Event, EventHandler};
use structopt::StructOpt;
use regex::Regex;

mod helper;

struct Handler;

#[derive(StructOpt, Debug)]
#[structopt(name = "slack-bot-rs")]
pub struct Opt {
    /// Bot token (env SLACK_TOKEN is also available)
    #[structopt(short = "t", long = "token", default_value = "")]
    token: String,

    /// Bot name
    #[structopt(required = true, short = "n", long = "bot-name")]
    bot_name: String,

    /// API server endpoint
    #[structopt(required = true, short = "s", long = "api-server")]
    api_server: String,
}

impl EventHandler for Handler {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        if let Event::Message(e) = event {
            if let slack::Message::Standard(m) = *e {
                let opt = Opt::from_args();
                let bot_id = helper::get_bot_id(cli, &opt.bot_name);
                let channel_id = m.channel.unwrap();
                let user_id = m.user.unwrap();
                let txt = m.text.unwrap();

                if !txt.contains(&bot_id) {
                    return
                }

                let re = 
                    Regex::new(
                        r"^<@[a-z0-9A-Z]+>\s+deploy\s+(v[0-9]\.[0-9]\.[0-9])\s*$")
                    .unwrap();

                if !re.is_match(&txt.as_str()) {
                    helper::send_msg(
                        cli,
                        &channel_id,
                        &format!("<@{}> ```usage: deploy VERSION```", user_id).as_str()
                    );
                    return
                }              

                if let Some(t) = re.captures(&txt.as_str()).unwrap().get(1) {
                    let params = helper::PostParams{
                        subject: "deploy".to_string(),
                        message: t.as_str().to_string(),
                    };
                    match helper::post(
                        &format!("http://{}/publish", &opt.api_server), &params)
                    {
                        Ok(res) if res.ok => info!("ok: {}", res.message),
                        Ok(res) => error!("failed: {}", res.message),
                        Err(e) => error!("error: {}", e),
                    };
                }
            }
        }
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        debug!("on_close");
    }

    fn on_connect(&mut self, _cli: &RtmClient) {
        debug!("on_connect");
    }
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    let opt = Opt::from_args();
    
    let token = 
        match env::var("SLACK_TOKEN") { 
            Err(_) if &opt.token == "" => {
                error!("error: slack token not specified");
                process::exit(1);
            },
            Err(_) => opt.token,
            Ok(v) => v,
        };

    let mut handler = Handler;
    let r = RtmClient::login_and_run(&token, &mut handler);

    match r {
        Ok(_) => {},
        Err(err) => panic!("Error: {}", err)
    }
}

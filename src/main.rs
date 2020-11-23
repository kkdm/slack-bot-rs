use slack;
use slack::{RtmClient, Event, EventHandler, Message};
use slack::api::MessageStandard;
use structopt::StructOpt;
use regex::Regex;

mod helper;

struct Handler;

#[derive(StructOpt, Debug)]
#[structopt(name = "slack-bot-rs")]
pub struct Opt {
    /// Bot token
    #[structopt(required = true, short, long)]
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
                    println!("{}", &txt);
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
                    match helper::post(&opt.api_server, &params) {
                        Ok(res) if res.ok => println!("ok: {}", res.message),
                        Ok(res) => println!("failed: {}", res.message),
                        Err(e) => println!("error: {}", e),
                    };
                }
            }
        }
    }

    fn on_close(&mut self, _cli: &RtmClient) {
        println!("on_close");
    }

    fn on_connect(&mut self, _cli: &RtmClient) {
        println!("on_connect");
    }
}

fn main() {
    let opt = Opt::from_args();
    let mut handler = Handler;
    let r = RtmClient::login_and_run(&opt.token, &mut handler);

    match r {
        Ok(_) => {},
        Err(err) => panic!("Error: {}", err)
    }
}

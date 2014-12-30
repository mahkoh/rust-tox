#![feature(phase,globs)]

extern crate regex;
#[phase(plugin)]
extern crate regex_macros;
extern crate tox;

use tox::core::*;

static BOOTSTRAP_IP: &'static str = "192.254.75.98";
static BOOTSTRAP_PORT: u16 = 33445;
static BOOTSTRAP_KEY: &'static str =
                    "951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F";
static GROUPCHAT_ADDR: &'static str =
        "EFA99A172718C2CCC642AF02BBA5369CB49311EF163D915ED64EA815335FC2748B1A458717E1";
static BOT_NAME: &'static str = "mahkohBot";

fn main() {
    let tox = Tox::new(ToxOptions::new()).unwrap();
    tox.set_name(BOT_NAME.to_string()).unwrap();

    let bootstrap_key = BOOTSTRAP_KEY.parse().unwrap();
    tox.bootstrap_from_address(BOOTSTRAP_IP.to_string(), BOOTSTRAP_PORT, 
                               box bootstrap_key).unwrap();

    let groupchat_addr = GROUPCHAT_ADDR.parse().unwrap();
    let groupbot_id = tox.add_friend(box groupchat_addr, "Hello".to_string()).ok().unwrap();

    let pattern = regex!(r"^(\[.+?\]: )?%(\w+)");
    for ev in tox.events().iter() {
        match ev {
            StatusMessage(id, _) if id == groupbot_id => {
                if tox.count_chatlist() < 1 {
                    tox.send_message(groupbot_id, "invite".to_string()).unwrap();
                    println!("connected to groupbot");
                }
            },
            GroupInvite(id, _, ref addr) if id == groupbot_id => {
                tox.join_groupchat(id, addr.clone()).unwrap();
                println!("invited to group");
            },
            GroupMessage(group, _, msg) => {
                println!("{}", msg);
                if let Some(cap) = pattern.captures(msg.as_slice()) {
                    let msg = match cap.at(2) {
                        Some("xot") => Some("https://github.com/mahkoh/Xot"),
                        _ => None,
                    };

                    if let Some(s) = msg {
                        let _ = tox.group_message_send(group, s.to_string());
                        println!("{}", "#### sent");
                    }
                }
            },
            _ => { }
        }
    }
}

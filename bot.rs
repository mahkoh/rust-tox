#![feature(globs)]
#![feature(phase)]
extern crate regex;
#[phase(plugin)]
extern crate regex_macros;
extern crate tox;
extern crate debug;

use tox::core::*;

static BOOTSTRAP_IP: &'static str = "192.254.75.98";
static BOOTSTRAP_PORT: u16 = 33445;
static BOOTSTRAP_KEY: &'static str =
                    "951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F";
static GROUPCHAT_ADDR: &'static str =
        "56A1ADE4B65B86BCD51CC73E2CD4E542179F47959FE3E0E21B4B0ACDADE51855D34D34D37CB5";
static BOT_NAME: &'static str = "mahkohBot";

fn main() {
    let tox = Tox::new(false).unwrap();
    tox.set_name(BOT_NAME.to_string()).unwrap();
    
    let bootstrap_key = from_str(BOOTSTRAP_KEY).unwrap();
    tox.bootstrap_from_address(BOOTSTRAP_IP.to_string(), false, BOOTSTRAP_PORT, 
                               box bootstrap_key).unwrap();

    let groupchat_addr = from_str(GROUPCHAT_ADDR).unwrap();
    let groupbot_id = tox.add_friend(box groupchat_addr, "Hello".to_string()).ok().unwrap();

    let pattern = regex!(r"^(\[.+?\]: )?%(\w+)");
    loop {
        for ev in tox.events() {
            match ev {
                StatusMessage(id, _) if id == groupbot_id => {
                    tox.send_message(groupbot_id, "invite".to_string()).unwrap();
                    println!("connected to groupbot");
                },
                GroupInvite(id, ref addr) if id == groupbot_id => {
                    tox.join_groupchat(id, addr.clone()).unwrap();
                    println!("invited to group");
                },
                GroupMessage(group, _, msg) => {
                    println!("{}", msg);
                    match pattern.captures(msg.as_slice()) {
                        Some(c) => {
                            let msg = match c.at(2) {
                                "xot"    => Some("https://github.com/mahkoh/Xot"),
                                "europe" => Some("http://i.imgur.com/mQbQbkf.jpg"),
                                "usa"    => Some("http://i.imgur.com/OEZTpCr.jpg"),
                                "germany" => Some("http://a.pomf.se/skymwy.jpg"),
                                _ => None,
                            };
                            match msg {
                                Some(s) => {
                                    let _ = tox.group_message_send(group, s.to_string());
                                    println!("{}", "#### sent");
                                },
                                None => { },
                            }
                        },
                        None => { }
                    }
                },
                _ => { }
            }
        }
        std::io::timer::sleep(50);
    }
}

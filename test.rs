extern crate tox;
extern crate debug;

use tox::core::{Tox, NameChange, StatusMessage, FriendMessage};

fn main() {
    let mut tox = Tox::new(false).unwrap();
    let id =
        from_str("951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F").unwrap();
    tox.bootstrap_from_address("192.254.75.98".to_string(), false, 33445, box id).unwrap();
    let groupbot =
        from_str("56A1ADE4B65B86BCD51CC73E2CD4E542179F47959FE3E0E21B4B0ACDADE51855D34D34D37CB5").unwrap();
    tox.add_friend(box groupbot, "Hello".to_string()).ok().unwrap();
    tox.set_name("mahkoh".to_string()).ok().unwrap();
    loop {
        for ev in tox.events() {
            match ev {
                NameChange(id, name) => println!("NameChange({}, \"{}\")", id, name),
                StatusMessage(id, msg) => println!("StatusMessage({}, \"{}\")", id, msg),
                FriendMessage(id, msg) => println!("FriendMessage({}, \"{}\")", id, msg),
                _ => println!("{:?}", ev),
            }
        }
        std::io::timer::sleep(50);
    }
}

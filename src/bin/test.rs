#![feature(globs)]
extern crate tox;
extern crate debug;

use tox::core::*;

fn main() {
    let tox = Tox::new(false).unwrap();
    let id =
        from_str("951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F").unwrap();
    tox.bootstrap_from_address("192.254.75.98".to_string(), false, 33445, box id).unwrap();
    let groupbot =
        from_str("56A1ADE4B65B86BCD51CC73E2CD4E542179F47959FE3E0E21B4B0ACDADE51855D34D34D37CB5").unwrap();
    tox.add_friend(box groupbot, "Hello".to_string()).ok().unwrap();
    tox.set_name("mahkoh".to_string()).ok().unwrap();
    loop {
        for ev in tox.events() {
            match ev.clone() {
                FriendRequest(..)       => println!("FriendRequest(..)       "),
                FriendMessage(..)       => println!("FriendMessage(..)       "),
                FriendAction(..)        => println!("FriendAction(..)        "),
                NameChange(..)          => println!("NameChange(..)          "),
                StatusMessage(id, _)       => {
                    println!("StatusMessage(..)       ");
                    let _ = tox.send_message(id, "invite".to_string());
                },
                UserStatus(..)          => println!("UserStatus(..)          "),
                TypingChange(..)        => println!("TypingChange(..)        "),
                ReadReceipt(..)         => println!("ReadReceipt(..)         "),
                ConnectionStatus(..)    => println!("ConnectionStatus(..)    "),
                GroupInvite(id, group)  => {
                    println!("GroupInvite(..)         ");
                    let _ = tox.join_groupchat(id, group);
                },
                GroupMessage(_, _, msg) => println!("GroupMessage(_, _, {})", msg),
                GroupNamelistChange(..) => println!("GroupNamelistChange(..) "),
                FileSendRequest(..)     => println!("FileSendRequest(..)     "),
                FileControl(..)         => println!("FileControl(..)         "),
                FileData(..)            => println!("FileData(..)            "),
            }
            println!("{:?}", ev);
        }
        std::io::timer::sleep(std::time::Duration::milliseconds(50));
    }
}

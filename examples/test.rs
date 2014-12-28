#![feature(globs)]
extern crate tox;

use tox::core::*;

fn main() {
    let tox = Tox::new(ToxOptions::new()).unwrap();

    let ids = [
        ("192.254.75.98"   , 33445 , "951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F"),
        ("37.187.46.132"   , 33445 , "A9D98212B3F972BD11DA52BEB0658C326FCCC1BFD49F347F9C2D3D8B61E1B927"),
        ("144.76.60.215"   , 33445 , "04119E835DF3E78BACF0F84235B300546AF8B936F035185E2A8E9E0A67C8924F"),
        ("23.226.230.47"   , 33445 , "A09162D68618E742FFBCA1C2C70385E6679604B2D80EA6E84AD0996A1AC8A074"),
        ("54.199.139.199"  , 33445 , "7F9C31FE850E97CEFD4C4591DF93FC757C7C12549DDD55F8EEAECC34FE76C029"),
        ("192.210.149.121" , 33445 , "F404ABAA1C99A9D37D61AB54898F56793E1DEF8BD46B1038B9D822E8460FAB67"),
        ("37.59.102.176"   , 33445 , "B98A2CEAA6C6A2FADC2C3632D284318B60FE5375CCB41EFA081AB67F500C1B0B"),
        ("178.21.112.187"  , 33445 , "4B2C19E924972CB9B57732FB172F8A8604DE13EEDA2A6234E348983344B23057"),
        ("107.161.17.51"   , 33445 , "7BE3951B97CA4B9ECDDA768E8C52BA19E9E2690AB584787BF4C90E04DBB75111"),
        ("31.7.57.236"     , 443   , "2A4B50D1D525DA2E669592A20C327B5FAD6C7E5962DC69296F9FEC77C4436E4E"),
        ("63.165.243.15"   , 443   , "8CD087E31C67568103E8C2A28653337E90E6B8EDA0D765D57C6B5172B4F1F04C"),
    ];

    for &(ip, port, id) in ids.iter() {
        let id = id.parse().unwrap();
        tox.bootstrap_from_address(ip.to_string(), port, box id).unwrap();
    }

    let groupbot = "56A1ADE4B65B86BCD51CC73E2CD4E542179F47959FE3E0E21B4B0ACDADE51855D34D34D37CB5".parse().unwrap();
    tox.set_name("test".to_string()).ok().unwrap();
    tox.add_friend(box groupbot, "Hello".to_string()).ok().unwrap();
    loop {
        for ev in tox.events() {
            match ev {
                FriendRequest(..)       => println!("FriendRequest(..)       "),
                FriendMessage(..)       => println!("FriendMessage(..)       "),
                FriendAction(..)        => println!("FriendAction(..)        "),
                NameChange(..)          => println!("NameChange(..)          "),
                StatusMessage(id, _)       => {
                    println!("StatusMessage(..)       ");
                    let _ = tox.send_message(id, "invite".to_string());
                },
                UserStatusVar(..)       => println!("UserStatusVar(..)       "),
                TypingChange(..)        => println!("TypingChange(..)        "),
                ReadReceipt(..)         => println!("ReadReceipt(..)         "),
                ConnectionStatusVar(..) => println!("ConnectionStatusVar(..) "),
                GroupInvite(id, _, group)  => {
                    println!("GroupInvite(..)         ");
                    let _ = tox.join_groupchat(id, group);
                },
                GroupMessage(_, _, msg) => println!("GroupMessage(_, _, {})", msg),
                GroupNamelistChange(..) => println!("GroupNamelistChange(..) "),
                FileSendRequest(..)     => println!("FileSendRequest(..)     "),
                FileControl(..)         => println!("FileControl(..)         "),
                FileData(..)            => println!("FileData(..)            "),
                AvatarInfo(..)          => println!("AvatarInfo(..)          "),
                AvatarData(..)          => println!("AvatarData(..)          "),
            }
        }
        std::io::timer::sleep(std::time::Duration::milliseconds(50));
    }
}

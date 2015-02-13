#![feature(libc)]

extern crate tox;
extern crate libc;
extern crate comm;

use tox::core::*;
use tox::av::Event::*;

use std::collections::{HashMap};

use comm::select::{Select};

fn main() {
    let (core_ctrl, core_events) = ToxControl::new(ToxOptions::new()).unwrap();
    let (av_ctrl, av_events) = core_ctrl.av(2).unwrap();

    let mut audiomap = HashMap::new();

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
        core_ctrl.bootstrap_from_address(ip.to_string(), port, Box::new(id)).unwrap();
    }

    let groupbot = "56A1ADE4B65B86BCD51CC73E2CD4E542179F47959FE3E0E21B4B0ACDADE51855D34D34D37CB5".parse().unwrap();
    core_ctrl.set_name("test".to_string()).ok().unwrap();
    core_ctrl.add_friend(Box::new(groupbot), "Hello".to_string()).ok().unwrap();

    let sel = Select::new();
    sel.add(&core_events);
    sel.add(&av_events);

    loop {
        sel.wait(&mut []);

        while let Ok(ev) = core_events.recv_async() {
            match ev {
                FriendRequest(..)       => println!("FriendRequest(..)       "),
                FriendMessage(..)       => println!("FriendMessage(..)       "),
                FriendAction(..)        => println!("FriendAction(..)        "),
                NameChange(..)          => println!("NameChange(..)          "),
                StatusMessage(id, _)       => {
                    println!("StatusMessage(..)       ");
                    let _ = core_ctrl.send_message(id, "invite".to_string());
                },
                UserStatusVar(..)       => println!("UserStatusVar(..)       "),
                TypingChange(..)        => println!("TypingChange(..)        "),
                ReadReceipt(..)         => println!("ReadReceipt(..)         "),
                ConnectionStatusVar(..) => println!("ConnectionStatusVar(..) "),
                GroupInvite(id, ty, data)  => {
                    println!("GroupInvite(_, {:?}, _)         ", ty);
                    match ty {
                        GroupchatType::Text => core_ctrl.join_groupchat(id, data).unwrap(),
                        GroupchatType::Av => av_ctrl.join_av_groupchat(id, data).unwrap(),
                    };
                },
                GroupMessage(_, _, msg) => println!("GroupMessage(_, _, {:?})", msg),
                GroupNamelistChange(gnum, pnum, change) => {
                    println!("GroupNamelistChange(..) ");
                    if change == ChatChange::PeerDel {
                        audiomap.remove(&(gnum, pnum));
                    }
                },
                FileSendRequest(..)     => println!("FileSendRequest(..)     "),
                FileControl(..)         => println!("FileControl(..)         "),
                FileData(..)            => println!("FileData(..)            "),
                AvatarInfo(..)          => println!("AvatarInfo(..)          "),
                AvatarData(..)          => println!("AvatarData(..)          "),
            }
        }

        while let Ok(ev) = av_events.recv_async() {
            match ev {
                Invite(..)             => println!("Av::Invite(..)"),
                Ringing(..)            => println!("Av::Ringing(..)"),
                Start(..)              => println!("Av::Start(..)"),
                Cancel(..)             => println!("Av::Cancel(..)"),
                Reject(..)             => println!("Av::Reject(..)"),
                End(..)                => println!("Av::End(..)"),
                RequestTimeout(..)     => println!("Av::RequestTimeout(..)"),
                PeerTimeout(..)        => println!("Av::PeerTimeout(..)"),
                PeerCsChange(..)       => println!("Av::PeerCsChange(..)"),
                SelfCsChange(..)       => println!("Av::SelfCsChange(..)"),
                GroupAudio(gnum, pnum, bit)  => {
                    //println!("Av::GroupAudio(..) SR: {} CH: {} N: {}", bit.sample_rate,
                    //         bit.channels, bit.samples);
                    if audiomap.get(&(gnum, pnum)).is_none() {
                        audiomap.insert((gnum, pnum), alsa::open().unwrap());
                    }
                    let alsa = *audiomap.get(&(gnum, pnum)).unwrap();
                    alsa::write(alsa, &bit).unwrap();
                }
            }
        }
    }
}

mod alsa {
    use libc::{c_int, c_uint, c_void, c_ulong, c_long, c_char, puts, EPIPE};
    use tox::av::{AudioBit};

    pub const RATE: u32 = 48000;
    pub const CHANNELS: u8 = 1;

    #[repr(C)]
    pub struct snd_pcm_t;

    pub const SND_PCM_STREAM_PLAYBACK: c_uint = 0;
    pub const SND_PCM_FORMAT_S16_LE: c_uint = 2;
    pub const SND_PCM_ACCESS_RW_INTERLEAVED: c_int = 3;

    #[link(name = "asound")]
    extern {
        fn snd_pcm_open(pcm: *mut *mut snd_pcm_t, name: *const u8, stream: c_uint,
                        mode: c_int) -> c_int;
        fn snd_pcm_set_params(pcm: *mut snd_pcm_t, format: c_uint, access: c_int,
                              channels: c_uint, rate: c_uint, soft_resample: c_int,
                              latency: c_uint) -> c_int;
        fn snd_pcm_writei(pcm: *mut snd_pcm_t, buffer: *const c_void,
                          size: c_ulong) -> c_long;
        fn snd_strerror(e: c_int) -> *const c_char;
        fn snd_pcm_prepare(pcm: *mut snd_pcm_t) -> c_int;
    }

    pub fn open() -> Option<*mut snd_pcm_t> {
        unsafe {
            let mut ptr = 0 as *mut snd_pcm_t;
            let e = snd_pcm_open(&mut ptr, b"default\0".as_ptr(), SND_PCM_STREAM_PLAYBACK,
                                 0);
            if e != 0 {
                puts(snd_strerror(e));
                return None;
            }
            let e = snd_pcm_set_params(ptr, SND_PCM_FORMAT_S16_LE,
                                  SND_PCM_ACCESS_RW_INTERLEAVED, CHANNELS as c_uint,
                                  RATE as c_uint, 1, 500000);
            if e != 0 {
                puts(snd_strerror(e));
                return None;
            }
            Some(ptr)
        }
    }

    pub fn write(p: *mut snd_pcm_t, bit: &AudioBit) -> Option<()> {
        unsafe {
            if bit.sample_rate != RATE || bit.channels != CHANNELS {
                println!("sample rate {} channels {}", bit.sample_rate, bit.channels);
                return None;
            }
            let e = snd_pcm_writei(p, bit.pcm.as_ptr() as *const c_void,
                                   bit.samples as c_ulong);
            if e < 0 {
                if e as c_int == -EPIPE {
                    snd_pcm_prepare(p);
                    Some(())
                } else {
                    puts(snd_strerror(e as c_int));
                    None
                }
            } else {
                Some(())
            }
        }
    }
}

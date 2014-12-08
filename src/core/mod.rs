/*!
    The core rust-tox module.

    # Example (a simple echo bot)

    ~~~{.rust}
    #![feature(globs)]
    extern crate tox;

    use tox::core::*;

    fn main() {
        let tox = Tox::new(ToxOptions::new()).unwrap();
        let bootkey = box from_str("951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F").unwrap();
        tox.bootstrap_from_address("192.254.75.98".to_string(), 33445, bootkey).unwrap();

        println!("Bot key: {}", tox.get_address());

        loop {
            for ev in tox.events() {
                match ev {
                    FriendRequest(fnum, _) => {
                        tox.add_friend_norequest(fnum).unwrap();
                    },
                    FriendMessage(fnum, msg) => {
                        tox.send_message(fnum, msg).unwrap();
                    },
                    _ => (),
                }
            }

            std::io::timer::sleep(std::time::Duration::milliseconds(50));
        }
    }
    ~~~
*/
use std::{fmt, mem};
use std::str::{FromStr};
use rust_core::slice::{MutableIntSlice};
pub use self::Event::*;

mod backend;
mod ll;

pub const MAX_NAME_LENGTH:          uint = 128u;
pub const MAX_MESSAGE_LENGTH:       uint = 1368u;
pub const MAX_STATUSMESSAGE_LENGTH: uint = 1007u;
pub const ID_CLIENT_SIZE:           uint = 32u;
pub const ADDRESS_SIZE:             uint = ID_CLIENT_SIZE + 6u;
pub const AVATAR_MAX_DATA_LENGTH:   uint = 16384u;
pub const HASH_LENGTH:              uint = 32u;

#[deriving(Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum AvatarFormat {
    None = ll::TOX_AVATAR_FORMAT_NONE as u8,
    PNG = ll::TOX_AVATAR_FORMAT_PNG as u8,
}

/// Tox events enum
#[deriving(Clone)]
pub enum Event {
    /// The first value is the client id, the second is the friend request message
    FriendRequest(Box<ClientId>, String),
    /// `(fnum, msg)` where `fnum` is the friend number and `msg` is the received message
    FriendMessage(i32, String),
    /// `(fnum, msg)` where `fnum` is the friend number and `msg` is the action message
    FriendAction(i32, String),
    /// `(fnum, name)` where `fnum` is the friend number and `name` is the new friend name
    NameChange(i32, String),
    /// `(fnum, status)` where `fnum` is the friend number and `status` is the status message
    StatusMessage(i32, String),
    /// `(fnum, usrst)` where `fnum` is the friend number and `usrst` is the friend status
    UserStatusVar(i32, UserStatus),
    /// `(fnum, is_typing)`. `true` value of is_typing means that friend is typing. `fnum` is the friend number
    TypingChange(i32, bool),
    // ?
    ReadReceipt(i32, u32),
    /// `(fnum, ConnectionStatus)`. `fnum` is the friend number
    ConnectionStatusVar(i32, ConnectionStatus),
    /// `(fnum, data)` where `data` is special data what needs
    /// to be passed to Tox::join_group method, `fnum` is the friend number
    GroupInvite(i32, Vec<u8>),
    /// `(gnum, pnum, msg)` where `gnum` is the group number, `pnum` is the peer number and `msg` is the message
    GroupMessage(i32, i32, String),
    /// `(gnum, pnum, ChatChange)`
    GroupNamelistChange(i32, i32, ChatChange),
    /// `(fnum, fid, fisize, finame)`
    FileSendRequest(i32, u8, u64, Vec<u8>),
    /// `(fnum, TranserType, fid, ControlType, data)`
    FileControl(i32, TransferType, u8, ControlType, Vec<u8>),
    /// `(fnum, fid, data)`
    FileData(i32, u8, Vec<u8>),
    /// `(fnum, AvatarFormat, Hash)`
    AvatarInfo(i32, AvatarFormat, Hash),
    /// `(fnum, AvatarFormat, Hash, data)`
    AvatarData(i32, AvatarFormat, Hash, Vec<u8>),
}

/// A Tox address consist of `ClientId`, nospam and checksum
#[repr(C)]
pub struct Address {
    id: ClientId,
    nospam: [u8, ..4],
    #[allow(dead_code)]
    checksum: [u8, ..2],
}

impl Clone for Address {
    fn clone(&self) -> Address {
        Address {
            id: self.id.clone(),
            nospam: self.nospam,
            checksum: self.checksum,
        }
    }
}

impl Address {
    fn checksum(&self) -> [u8, ..2] {
        let mut check = [0u8, 0u8];
        for (i, &x) in self.id.raw.iter().enumerate() {
            check[i % 2] ^= x;
        }
        for i in range(0u, 4) {
            check[(ID_CLIENT_SIZE + i) % 2] ^= self.nospam[i];
        }
        check
    }
}

impl fmt::Show for Address {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(self.id.fmt(fmt));
        try!(write!(fmt, "{:02X}", self.nospam[0]));
        try!(write!(fmt, "{:02X}", self.nospam[1]));
        try!(write!(fmt, "{:02X}", self.nospam[2]));
        try!(write!(fmt, "{:02X}", self.nospam[3]));
        let check = self.checksum();
        try!(write!(fmt, "{:02X}", check[0]));
        try!(write!(fmt, "{:02X}", check[1]));
        Ok(())
    }
}

impl FromStr for Address {
    fn from_str(s: &str) -> Option<Address> {
        if s.len() != 2 * ADDRESS_SIZE {
            return None;
        }

        let mut id     = [0u8, ..32];
        let mut nospam = [0u8, ..4];
        let mut check  = [0u8, ..2];

        if parse_hex(s.slice(0, 2*ID_CLIENT_SIZE), id.as_mut_slice()).is_err() {
            return None;
        }
        if parse_hex(s.slice(2*ID_CLIENT_SIZE, 2*ID_CLIENT_SIZE+8),
                             nospam.as_mut_slice()).is_err() {
            return None;
        }
        if parse_hex(s.slice(2*ID_CLIENT_SIZE+8, 2*ADDRESS_SIZE),
                             check.as_mut_slice()).is_err() {
            return None;
        }

        let addr = Address { id: ClientId { raw: id }, nospam: nospam, checksum: check };
        if addr.checksum().as_slice() != check.as_slice() {
            return None;
        }
        Some(addr)
    }
}

fn parse_hex(s: &str, buf: &mut [u8]) -> Result<(),()> {
    if s.len() != 2*buf.len() {
        return Err(());
    }
    for i in range(0u, buf.len()) {
        for j in range(0u, 2) {
            buf[i] = (buf[i] << 4) + match s.as_bytes()[2*i + j] as char {
                c @ '0' ... '9' => (c as u8) - ('0' as u8),
                c @ 'a' ... 'f' => (c as u8) - ('a' as u8) + 10,
                c @ 'A' ... 'F' => (c as u8) - ('A' as u8) + 10,
                _              => return Err(()),
            }
        }
    }
    return Ok(());
}

/// `ClientId` is the main part of tox `Address`. Other two are nospam and checksum.
#[repr(C)]
pub struct ClientId {
    pub raw: [u8, ..ID_CLIENT_SIZE],
}

impl Clone for ClientId {
    fn clone(&self) -> ClientId {
        ClientId { raw: self.raw }
    }
}

impl fmt::Show for ClientId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for &n in self.raw.iter() {
            try!(write!(fmt, "{:02X}", n));
        }
        Ok(())
    }
}

impl FromStr for ClientId {
    fn from_str(s: &str) -> Option<ClientId> {
        if s.len() != 2 * ID_CLIENT_SIZE {
            return None;
        }

        let mut id = [0u8, ..ID_CLIENT_SIZE];

        if parse_hex(s, id.as_mut_slice()).is_err() {
            return None;
        }
        Some(ClientId { raw: id })
    }
}

/// Locally-calculated cryptographic hash of the avatar data
#[deriving(Clone, PartialEq, Eq)]
pub struct Hash {
    pub hash: [u8, ..HASH_LENGTH]
}

/*impl Clone for Hash {
    fn clone(&self) -> Hash {
        Hash { hash: self.hash }
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Hash) -> bool {
        self.hash.as_slice() == other.hash.as_slice()
    }
}

impl Eq for Hash { }
*/
impl Hash {
    pub fn new(data: &[u8]) -> Result<Hash, ()> {
        let mut hash: Hash = unsafe { mem::uninitialized() };
        let res = unsafe {
            ll::tox_hash(hash.hash.as_mut_ptr(), data.as_ptr(), data.len() as u32)
        };
        match res {
            0 => Ok(hash),
            _ => Err(()),
        }
    }
}

#[deriving(Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Online,
    Offline,
}

#[repr(u32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum UserStatus {
    None = ll::TOX_USERSTATUS_NONE,
    Away = ll::TOX_USERSTATUS_AWAY,
    Busy = ll::TOX_USERSTATUS_BUSY,
}

#[repr(u32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum ChatChange {
    PeerAdd = ll::TOX_CHAT_CHANGE_PEER_ADD,
    PeerDel = ll::TOX_CHAT_CHANGE_PEER_DEL,
    PeerName = ll::TOX_CHAT_CHANGE_PEER_NAME,
}

#[repr(u32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum ControlType {
    Accept = ll::TOX_FILECONTROL_ACCEPT,
    Pause = ll::TOX_FILECONTROL_PAUSE,
    Kill = ll::TOX_FILECONTROL_KILL,
    Finished = ll::TOX_FILECONTROL_FINISHED,
    ResumeBroken = ll::TOX_FILECONTROL_RESUME_BROKEN,
}

/// Faerr - Friend Add Error
#[repr(i32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum Faerr {
    Toolong = ll::TOX_FAERR_TOOLONG,
    Nomessage = ll::TOX_FAERR_NOMESSAGE,
    Ownkey = ll::TOX_FAERR_OWNKEY,
    Alreadysent = ll::TOX_FAERR_ALREADYSENT,
    Unknown = ll::TOX_FAERR_UNKNOWN,
    Badchecksum = ll::TOX_FAERR_BADCHECKSUM,
    Setnewnospam = ll::TOX_FAERR_SETNEWNOSPAM,
    Nomem = ll::TOX_FAERR_NOMEM,
}

#[deriving(Clone, PartialEq, Eq)]
pub enum TransferType {
    Receiving,
    Sending,
}


/// ToxOptions provides options that tox will be initalized with.
///
/// Usage:
/// ```
///     let txo = ToxOptions::new().ipv6().proxy("[proxy address]", port);
///     let tox = Tox::new(txo);
/// ```
pub struct ToxOptions {
    txo: ll::Tox_Options
}

impl ToxOptions {
    /// Create a default ToxOptions struct
    pub fn new() -> ToxOptions {
        ToxOptions {
            txo: ll::Tox_Options {
                ipv6enabled: 0,
                udp_disabled: 0,
                proxy_enabled: 0,
                proxy_address: [0, ..256u],
                proxy_port: 0,
            }
        }
    }

    /// Enable IPv6
    pub fn ipv6(mut self) -> ToxOptions {
        self.txo.ipv6enabled = 1;
        self
    }

    /// Disable UDP
    pub fn no_udp(mut self) -> ToxOptions {
        self.txo.udp_disabled = 1;
        self
    }

    /// Use a proxy
    pub fn proxy(mut self, addr: &str, port: u16) -> ToxOptions {
        if addr.len() >= 256 {
            panic!("proxy address is too long");
        }

        self.txo.proxy_address.as_mut_slice()
                              .as_unsigned_mut()
                              .clone_from_slice(addr.as_bytes());
        self.txo.proxy_enabled = 1;
        self.txo.proxy_port = port;
        self
    }
}

/// The main tox struct. All the control goes here.
pub struct Tox {
    pub events: Receiver<Event>,
    control: SyncSender<backend::Control>,
}

macro_rules! forward {
    ($slf:expr, $name:expr, ($($pp:ident),+), ->) => {
        {
            let (snd, rcv) = channel();
            $slf.control.send($name($($pp),*, snd));
            rcv.recv()
        }
    };
    ($slf:expr, $name:expr, ->) => {
        {
            let (snd, rcv) = channel();
            $slf.control.send($name(snd));
            rcv.recv()
        }
    };
    ($slf:expr, $name:expr, ($($pp:ident),+)) => {
        {
            $slf.control.send($name($($pp),*));
        }
    };
    ($slf:expr, $name:expr) => {
            $slf.control.send($name);
    };

}

impl Drop for Tox {
    fn drop(&mut self) {
        forward!(self, backend::Control::Kill)
    }
}

impl Tox {
    /// Get self address
    pub fn get_address(&self) -> Address {
        forward!(self, backend::Control::GetAddress, ->)
    }
    /// Add a friend and send friend request
    pub fn add_friend(&self, address: Box<Address>, msg: String) -> Result<i32, Faerr> {
        forward!(self, backend::Control::AddFriend, (address, msg), ->)
    }
    /// Add a friend without sending friend request. Beware, friend will appear online only if he added you too
    pub fn add_friend_norequest(&self, client_id: Box<ClientId>) -> Result<i32, ()> {
        forward!(self, backend::Control::AddFriendNorequest, (client_id), ->)
    }
    /// Get friend number associated with given ClientId
    pub fn get_friend_number(&self, client_id: Box<ClientId>) -> Result<i32, ()> {
        forward!(self, backend::Control::GetFriendNumber, (client_id), ->)
    }
    /// Get ClientId of the friend with given friend number
    pub fn get_client_id(&self, friendnumber: i32) -> Result<Box<ClientId>, ()> {
        forward!(self, backend::Control::GetClientId, (friendnumber), ->)
    }
    /// Remove the friend with given friend number
    pub fn del_friend(&self, friendnumber: i32) -> Result<(),()> {
        forward!(self, backend::Control::DelFriend, (friendnumber), ->)
    }
    /// Get the connection status of the friend
    pub fn get_friend_connection_status(
            &self,
            friendnumber: i32) -> Result<ConnectionStatus, ()> {
        forward!(self, backend::Control::GetFriendConnectionStatus, (friendnumber), ->)
    }
    /// Returns `true` if friend with given friend number exists. Otherwise, returns `false`
    pub fn friend_exists(&self, friendnumber: i32) -> bool {
        forward!(self, backend::Control::FriendExists, (friendnumber), ->)
    }
    /// Send a message to the friend
    pub fn send_message(&self, friendnumber: i32,
                        msg: String) -> Result<u32, ()> {
        forward!(self, backend::Control::SendMessage, (friendnumber, msg), ->)
    }
    /// Send an action message to the friend
    pub fn send_action(&self, friendnumber: i32, action: String) -> Result<u32, ()> {
        forward!(self, backend::Control::SendAction, (friendnumber, action), ->)
    }
    /// Set self nickname
    pub fn set_name(&self, name: String) -> Result<(),()> {
        forward!(self, backend::Control::SetName, (name), ->)
    }
    /// Returns the self nickname
    pub fn get_self_name(&self) -> Result<String, ()> {
        forward!(self, backend::Control::GetSelfName, ->)
    }
    /// Get the nickname of the friend
    pub fn get_name(&self, friendnumber: i32) -> Result<String, ()> {
        forward!(self, backend::Control::GetName, (friendnumber), ->)
    }
    /// Set self status message
    pub fn set_status_message(&self, status: String) -> Result<(),()> {
        forward!(self, backend::Control::SetStatusMessage, (status), ->)
    }
    /// Set self status (`None`, `Away` or `Busy`)
    pub fn set_user_status(&self, userstatus: UserStatus) -> Result<(), ()> {
        forward!(self, backend::Control::SetUserStatus, (userstatus), ->)
    }
    /// Get the status message of the friend
    pub fn get_status_message(&self, friendnumber: i32) -> Result<String, ()> {
        forward!(self, backend::Control::GetStatusMessage, (friendnumber), ->)
    }
    /// Get self status message
    pub fn get_self_status_message(&self) -> Result<String, ()> {
        forward!(self, backend::Control::GetSelfStatusMessage, ->)
    }
    /// Get status of the friend
    pub fn get_user_status(&self, friendnumber: i32) -> Result<UserStatus, ()> {
        forward!(self, backend::Control::GetUserStatus, (friendnumber), ->)
    }
    /// Get self status
    pub fn get_self_user_status(&self) -> Result<UserStatus, ()> {
        forward!(self, backend::Control::GetSelfUserStatus, ->)
    }
    /// Return timestamp of last time the friend was seen online, or 0 if never seen
    pub fn get_last_online(&self, friendnumber: i32) -> Result<u64, ()> {
        forward!(self, backend::Control::GetLastOnline, (friendnumber), ->)
    }
    /// Set typing status for the given friend.You are responsible for turning it on or off
    pub fn set_user_is_typing(&self, friendnumber: i32,
                              is_typing: bool) -> Result<(), ()> {
        forward!(self, backend::Control::SetUserIsTyping, (friendnumber, is_typing), ->)
    }
    /// Get typing status of the given friend
    pub fn get_is_typing(&self, friendnumber: i32) -> bool {
        forward!(self, backend::Control::GetIsTyping, (friendnumber), ->)
    }

    pub fn set_sends_receipts(&self, friendnumber: i32, yesno: bool) {
        forward!(self, backend::Control::SetSendsReceipts, (friendnumber, yesno))
    }
    /// Returns the number of friends
    pub fn count_friendlist(&self) -> u32 {
        forward!(self, backend::Control::CountFriendlist, ->)
    }
    /// Returns the number of chats
    pub fn count_chatlist(&self) -> u32 {
        forward!(self, backend::Control::CountChatlist, ->)
    }
    /// Get the number of online friends
    pub fn get_num_online_friends(&self) -> u32 {
        forward!(self, backend::Control::GetNumOnlineFriends, ->)
    }
    /// Get the Vec of valid friend IDs
    pub fn get_friendlist(&self) -> Vec<i32> {
        forward!(self, backend::Control::GetFriendlist, ->)
    }
    /// Get self nospam
    pub fn get_nospam(&self) -> [u8, ..4] {
        forward!(self, backend::Control::GetNospam, ->)
    }
    /// Set self nospam
    pub fn set_nospam(&self, nospam: [u8, ..4]) {
        forward!(self, backend::Control::SetNospam, (nospam))
    }
    /// Create a new groupchat, returns groupchat number
    pub fn add_groupchat(&self) -> Result<i32, ()> {
        forward!(self, backend::Control::AddGroupchat, ->)
    }
    /// Leave the groupchat
    pub fn del_groupchat(&self, groupnumber: i32) -> Result<(),()> {
        forward!(self, backend::Control::DelGroupchat, (groupnumber), ->)
    }
    /// Returns the name of peer with given peer number in the groupchat
    pub fn group_peername(&self, groupnumber: i32,
                          peernumber: i32) -> Result<String, ()> {
        forward!(self, backend::Control::GroupPeername, (groupnumber, peernumber), ->)
    }
    /// Invite the friend to the groupchat
    pub fn invite_friend(&self, friendnumber: i32, groupnumber: i32) -> Result<(), ()> {
        forward!(self, backend::Control::InviteFriend, (friendnumber, groupnumber), ->)
    }
    /// Join a groupchat using `data` obtained by `GroupInvite` event
    pub fn join_groupchat(&self, friendnumber: i32, data: Vec<u8>) -> Result<i32, ()> {
        forward!(self, backend::Control::JoinGroupchat, (friendnumber, data), ->)
    }
    /// Send a message to the groupchat
    pub fn group_message_send(&self, groupnumber: i32,
                              message: String) -> Result<(), ()> {
        forward!(self, backend::Control::GroupMessageSend, (groupnumber, message), ->)
    }
    /// Send an action message to the groupchat
    pub fn group_action_send(&self, groupnumber: i32, action: String) -> Result<(), ()> {
        forward!(self, backend::Control::GroupActionSend, (groupnumber, action), ->)
    }
    /// Returns number of peers in the groupchat
    pub fn group_number_peers(&self, groupnumber: i32) -> Result<i32, ()> {
        forward!(self, backend::Control::GroupNumberPeers, (groupnumber), ->)
    }
    /// Returns list of all peer names in the groupchat
    pub fn group_get_names(&self,
                           groupnumber: i32) -> Result<Vec<Option<String>>, ()> {
        forward!(self, backend::Control::GroupGetNames, (groupnumber), ->)
    }
    /// Returns the Vec of all valid group IDs
    pub fn get_chatlist(&self) -> Vec<i32> {
        forward!(self, backend::Control::GetChatlist, ->)
    }

    pub fn set_avatar(&self, format: AvatarFormat, data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::Control::SetAvatar, (format, data), ->)
    }

    pub fn unset_avatar(&self) {
        forward!(self, backend::Control::UnsetAvatar)
    }

    pub fn get_self_avatar(&self) -> Result<(AvatarFormat, Vec<u8>, Hash), ()> {
        forward!(self, backend::Control::GetSelfAvatar, ->)
    }

    pub fn request_avatar_info(&self, friendnumber: i32) -> Result<(), ()> {
        forward!(self, backend::Control::RequestAvatarInfo, (friendnumber), ->)
    }

    pub fn send_avatar_info(&self, friendnumber: i32) -> Result<(), ()> {
        forward!(self, backend::Control::SendAvatarInfo, (friendnumber), ->)
    }

    pub fn request_avatar_data(&self, friendnumber: i32) -> Result<(), ()> {
        forward!(self, backend::Control::RequestAvatarData, (friendnumber), ->)
    }

    pub fn new_file_sender(&self, friendnumber: i32, filesize: u64,
                           filename: Path) -> Result<i32, ()> {
        forward!(self, backend::Control::NewFileSender, (friendnumber, filesize, filename), ->)
    }

    pub fn file_send_control(&self, friendnumber: i32, send_receive: TransferType,
                             filenumber: u8, message_id: u8,
                             data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::Control::FileSendControl, (friendnumber, send_receive, filenumber,
                                            message_id, data), ->)
    }

    pub fn file_send_data(&self, friendnumber: i32, filenumber: u8,
                          data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::Control::FileSendData, (friendnumber, filenumber, data), ->)
    }

    pub fn file_data_size(&self, friendnumber: i32) -> Result<i32, ()> {
        forward!(self, backend::Control::FileDataSize, (friendnumber), ->)
    }

    pub fn file_data_remaining(&self, friendnumber: i32, filenumber: u8,
                               send_receive: TransferType) -> Result<u64, ()> {
        forward!(self, backend::Control::FileDataRemaining, (friendnumber, filenumber, send_receive), ->)
    }
    /// Bootstrap from the given (address, port, ClientId)
    pub fn bootstrap_from_address(&self, address: String, port: u16,
                                  public_key: Box<ClientId>) -> Result<(), ()> {
        forward!(self, backend::Control::BootstrapFromAddress, (address, port, public_key), ->)
    }
    /// Returns `true` if connected to DHT. Otherwise, returns `false`
    pub fn is_connected(&self) -> bool {
        forward!(self, backend::Control::Isconnected, ->)
    }
    /// Create a new tox instance
    pub fn new(mut opts: ToxOptions) -> Option<Tox> {
        let (ctrl, events) = match backend::Backend::new(&mut opts.txo) {
            Some(x) => x,
            None => return None,
        };
        Some(Tox {
            events: events,
            control: ctrl,
        })
    }
    /// Returns a tox data that should be saved in the tox file
    pub fn save(&self) -> Vec<u8> {
        forward!(self, backend::Control::Save, ->)
    }
    /// Load instance data from Vec
    pub fn load(&self, data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::Control::Load, (data), ->)
    }
    /// Return an events iterator
    pub fn events<'a>(&'a self) -> EventIter<'a> {
        EventIter { events: &self.events }
    }
}

/// Tox events iterator
pub struct EventIter<'a> {
    events: &'a Receiver<Event>,
}

impl<'a> Iterator<Event> for EventIter<'a> {
    fn next(&mut self) -> Option<Event> {
        match self.events.try_recv() {
            Ok(t) => Some(t),
            _ => None,
        }
    }
}

use std::{fmt, mem};
use std::from_str::{FromStr};
use rust_core::slice::{MutableIntSlice};

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
    AvatarNone = ll::TOX_AVATAR_FORMAT_NONE as u8,
    AvatarPNG = ll::TOX_AVATAR_FORMAT_PNG as u8,
}

#[deriving(Clone)]
pub enum Event {
    FriendRequest(Box<ClientId>, String),
    FriendMessage(i32, String),
    FriendAction(i32, String),
    NameChange(i32, String),
    StatusMessage(i32, String),
    UserStatusVar(i32, UserStatus),
    TypingChange(i32, bool),
    ReadReceipt(i32, u32),
    ConnectionStatusVar(i32, ConnectionStatus),
    GroupInvite(i32, Vec<u8>),
    GroupMessage(i32, i32, String),
    GroupNamelistChange(i32, i32, ChatChange),
    FileSendRequest(i32, u8, u64, Vec<u8>),
    FileControl(i32, TransferType, u8, ControlType, Vec<u8>),
    FileData(i32, u8, Vec<u8>),
    AvatarInfo(i32, AvatarFormat, Hash),
    AvatarData(i32, AvatarFormat, Hash, Vec<u8>),
}

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

pub fn parse_hex(s: &str, buf: &mut [u8]) -> Result<(),()> {
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

pub struct Hash {
    pub hash: [u8, ..HASH_LENGTH]
}

impl Clone for Hash {
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

#[deriving(Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Online,
    Offline,
}

#[repr(u32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum UserStatus {
    UserStatusNone = ll::TOX_USERSTATUS_NONE,
    UserStatusAway = ll::TOX_USERSTATUS_AWAY,
    UserStatusBusy = ll::TOX_USERSTATUS_BUSY,
}

#[repr(u32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum ChatChange {
    ChatChangePeerAdd = ll::TOX_CHAT_CHANGE_PEER_ADD,
    ChatChangePeerDel = ll::TOX_CHAT_CHANGE_PEER_DEL,
    ChatChangePeerName = ll::TOX_CHAT_CHANGE_PEER_NAME,
}

#[repr(u32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum ControlType {
    ControlAccept = ll::TOX_FILECONTROL_ACCEPT,
    ControlPause = ll::TOX_FILECONTROL_PAUSE,
    ControlKill = ll::TOX_FILECONTROL_KILL,
    ControlFinished = ll::TOX_FILECONTROL_FINISHED,
    ControlResumeBroken = ll::TOX_FILECONTROL_RESUME_BROKEN,
}

#[repr(i32)]
#[deriving(Clone, PartialEq, Eq)]
pub enum Faerr {
    FaerrToolong = ll::TOX_FAERR_TOOLONG,
    FaerrNomessage = ll::TOX_FAERR_NOMESSAGE,
    FaerrOwnkey = ll::TOX_FAERR_OWNKEY,
    FaerrAlreadysent = ll::TOX_FAERR_ALREADYSENT,
    FaerrUnknown = ll::TOX_FAERR_UNKNOWN,
    FaerrBadchecksum = ll::TOX_FAERR_BADCHECKSUM,
    FaerrSetnewnospam = ll::TOX_FAERR_SETNEWNOSPAM,
    FaerrNomem = ll::TOX_FAERR_NOMEM,
}

#[deriving(Clone, PartialEq, Eq)]
pub enum TransferType {
    Receiving,
    Sending,
}


/// A `Tox_Option` struct wrapper
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
        forward!(self, backend::Kill)
    }
}

impl Tox {
    pub fn get_address(&self) -> Address {
        forward!(self, backend::GetAddress, ->)
    }

    pub fn add_friend(&self, address: Box<Address>, msg: String) -> Result<i32, Faerr> {
        forward!(self, backend::AddFriend, (address, msg), ->)
    }

    pub fn add_friend_norequest(&self, client_id: Box<ClientId>) -> Result<i32, ()> {
        forward!(self, backend::AddFriendNorequest, (client_id), ->)
    }

    pub fn get_friend_number(&self, client_id: Box<ClientId>) -> Result<i32, ()> {
        forward!(self, backend::GetFriendNumber, (client_id), ->)
    }

    pub fn get_client_id(&self, friendnumber: i32) -> Result<Box<ClientId>, ()> {
        forward!(self, backend::GetClientId, (friendnumber), ->)
    }

    pub fn del_friend(&self, friendnumber: i32) -> Result<(),()> {
        forward!(self, backend::DelFriend, (friendnumber), ->)
    }

    pub fn get_friend_connection_status(
            &self,
            friendnumber: i32) -> Result<ConnectionStatus, ()> {
        forward!(self, backend::GetFriendConnectionStatus, (friendnumber), ->)
    }

    pub fn friend_exists(&self, friendnumber: i32) -> bool {
        forward!(self, backend::FriendExists, (friendnumber), ->)
    }

    pub fn send_message(&self, friendnumber: i32,
                        msg: String) -> Result<u32, ()> {
        forward!(self, backend::SendMessage, (friendnumber, msg), ->)
    }

    pub fn send_action(&self, friendnumber: i32, action: String) -> Result<u32, ()> {
        forward!(self, backend::SendAction, (friendnumber, action), ->)
    }

    pub fn set_name(&self, name: String) -> Result<(),()> {
        forward!(self, backend::SetName, (name), ->)
    }

    pub fn get_self_name(&self) -> Result<String, ()> {
        forward!(self, backend::GetSelfName, ->)
    }

    pub fn get_name(&self, friendnumber: i32) -> Result<String, ()> {
        forward!(self, backend::GetName, (friendnumber), ->)
    }

    pub fn set_status_message(&self, status: String) -> Result<(),()> {
        forward!(self, backend::SetStatusMessage, (status), ->)
    }

    pub fn set_user_status(&self, userstatus: UserStatus) -> Result<(), ()> {
        forward!(self, backend::SetUserStatus, (userstatus), ->)
    }

    pub fn get_status_message(&self, friendnumber: i32) -> Result<String, ()> {
        forward!(self, backend::GetStatusMessage, (friendnumber), ->)
    }

    pub fn get_self_status_message(&self) -> Result<String, ()> {
        forward!(self, backend::GetSelfStatusMessage, ->)
    }

    pub fn get_user_status(&self, friendnumber: i32) -> Result<UserStatus, ()> {
        forward!(self, backend::GetUserStatus, (friendnumber), ->)
    }

    pub fn get_self_user_status(&self) -> Result<UserStatus, ()> {
        forward!(self, backend::GetSelfUserStatus, ->)
    }

    pub fn get_last_online(&self, friendnumber: i32) -> Result<u64, ()> {
        forward!(self, backend::GetLastOnline, (friendnumber), ->)
    }

    pub fn set_user_is_typing(&self, friendnumber: i32,
                              is_typing: bool) -> Result<(), ()> {
        forward!(self, backend::SetUserIsTyping, (friendnumber, is_typing), ->)
    }

    pub fn get_is_typing(&self, friendnumber: i32) -> bool {
        forward!(self, backend::GetIsTyping, (friendnumber), ->)
    }

    pub fn set_sends_receipts(&self, friendnumber: i32, yesno: bool) {
        forward!(self, backend::SetSendsReceipts, (friendnumber, yesno))
    }

    pub fn count_friendlist(&self) -> u32 {
        forward!(self, backend::CountFriendlist, ->)
    }

    pub fn count_chatlist(&self) -> u32 {
        forward!(self, backend::CountChatlist, ->)
    }

    pub fn get_num_online_friends(&self) -> u32 {
        forward!(self, backend::GetNumOnlineFriends, ->)
    }

    pub fn get_friendlist(&self) -> Vec<i32> {
        forward!(self, backend::GetFriendlist, ->)
    }

    pub fn get_nospam(&self) -> [u8, ..4] {
        forward!(self, backend::GetNospam, ->)
    }

    pub fn set_nospam(&self, nospam: [u8, ..4]) {
        forward!(self, backend::SetNospam, (nospam))
    }

    pub fn add_groupchat(&self) -> Result<i32, ()> {
        forward!(self, backend::AddGroupchat, ->)
    }

    pub fn del_groupchat(&self, groupnumber: i32) -> Result<(),()> {
        forward!(self, backend::DelGroupchat, (groupnumber), ->)
    }

    pub fn group_peername(&self, groupnumber: i32,
                          peernumber: i32) -> Result<String, ()> {
        forward!(self, backend::GroupPeername, (groupnumber, peernumber), ->)
    }

    pub fn invite_friend(&self, friendnumber: i32, groupnumber: i32) -> Result<(), ()> {
        forward!(self, backend::InviteFriend, (friendnumber, groupnumber), ->)
    }

    pub fn join_groupchat(&self, friendnumber: i32, data: Vec<u8>) -> Result<i32, ()> {
        forward!(self, backend::JoinGroupchat, (friendnumber, data), ->)
    }

    pub fn group_message_send(&self, groupnumber: i32,
                              message: String) -> Result<(), ()> {
        forward!(self, backend::GroupMessageSend, (groupnumber, message), ->)
    }

    pub fn group_action_send(&self, groupnumber: i32, action: String) -> Result<(), ()> {
        forward!(self, backend::GroupActionSend, (groupnumber, action), ->)
    }

    pub fn group_number_peers(&self, groupnumber: i32) -> Result<i32, ()> {
        forward!(self, backend::GroupNumberPeers, (groupnumber), ->)
    }

    pub fn group_get_names(&self,
                           groupnumber: i32) -> Result<Vec<Option<String>>, ()> {
        forward!(self, backend::GroupGetNames, (groupnumber), ->)
    }

    pub fn get_chatlist(&self) -> Vec<i32> {
        forward!(self, backend::GetChatlist, ->)
    }

    pub fn set_avatar(&self, format: AvatarFormat, data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::SetAvatar, (format, data), ->)
    }

    pub fn unset_avatar(&self) {
        forward!(self, backend::UnsetAvatar)
    }

    pub fn get_self_avatar(&self) -> Result<(AvatarFormat, Vec<u8>, Hash), ()> {
        forward!(self, backend::GetSelfAvatar, ->)
    }

    pub fn request_avatar_info(&self, friendnumber: i32) -> Result<(), ()> {
        forward!(self, backend::RequestAvatarInfo, (friendnumber), ->)
    }

    pub fn send_avatar_info(&self, friendnumber: i32) -> Result<(), ()> {
        forward!(self, backend::SendAvatarInfo, (friendnumber), ->)
    }

    pub fn request_avatar_data(&self, friendnumber: i32) -> Result<(), ()> {
        forward!(self, backend::RequestAvatarData, (friendnumber), ->)
    }

    pub fn new_file_sender(&self, friendnumber: i32, filesize: u64,
                           filename: Path) -> Result<i32, ()> {
        forward!(self, backend::NewFileSender, (friendnumber, filesize, filename), ->)
    }

    pub fn file_send_control(&self, friendnumber: i32, send_receive: TransferType,
                             filenumber: u8, message_id: u8,
                             data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::FileSendControl, (friendnumber, send_receive, filenumber,
                                            message_id, data), ->)
    }

    pub fn file_send_data(&self, friendnumber: i32, filenumber: u8,
                          data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::FileSendData, (friendnumber, filenumber, data), ->)
    }

    pub fn file_data_size(&self, friendnumber: i32) -> Result<i32, ()> {
        forward!(self, backend::FileDataSize, (friendnumber), ->)
    }

    pub fn file_data_remaining(&self, friendnumber: i32, filenumber: u8,
                               send_receive: TransferType) -> Result<u64, ()> {
        forward!(self, backend::FileDataRemaining, (friendnumber, filenumber, send_receive), ->)
    }

    pub fn bootstrap_from_address(&self, address: String, port: u16,
                                  public_key: Box<ClientId>) -> Result<(), ()> {
        forward!(self, backend::BootstrapFromAddress, (address, port, public_key), ->)
    }

    pub fn is_connected(&self) -> bool {
        forward!(self, backend::Isconnected, ->)
    }

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

    pub fn save(&self) -> Vec<u8> {
        forward!(self, backend::Save, ->)
    }

    pub fn load(&self, data: Vec<u8>) -> Result<(), ()> {
        forward!(self, backend::Load, (data), ->)
    }

    pub fn events<'a>(&'a self) -> EventIter<'a> {
        EventIter { events: &self.events }
    }
}

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

pub fn hash(data: &[u8]) -> Result<Hash, ()> {
    let mut hash: Hash = unsafe { mem::uninitialized() };
    let res = unsafe {
        ll::tox_hash(hash.hash.as_mut_ptr(), data.as_ptr(), data.len() as u32)
    };
    match res {
        0 => Ok(hash),
        _ => Err(()),
    }
}

use std::{fmt};
use std::from_str::{FromStr};
use self::ll::*;

mod backend;
mod ll;

pub static MAX_NAME_LENGTH: uint = 128u;
pub static MAX_MESSAGE_LENGTH: uint = 1368u;
pub static MAX_STATUSMESSAGE_LENGTH: uint = 1007u;
pub static ID_CLIENT_SIZE: uint = 32u;
pub static ADDRESS_SIZE: uint = ID_CLIENT_SIZE + 6u;

pub enum Event {
    FriendRequest(Box<ClientId>, String),
    FriendMessage(i32, String),
    FriendAction(i32, String),
    NameChange(i32, String),
    StatusMessage(i32, String),
    UserStatus(i32, UserStatus),
    TypingChange(i32, bool),
    ReadReceipt(i32, u32),
    ConnectionStatus(i32, ConnectionStatus),
    GroupInvite(i32, Box<ClientId>),
    GroupMessage(i32, i32, String),
    GroupNamelistChange(i32, i32, ChatChange),
    FileSendRequest(i32, u8, u64, Vec<u8>),
    FileControl(i32, TransferType, u8, ControlType, Vec<u8>),
    FileData(i32, u8, Vec<u8>),
}

pub struct Address {
    id: ClientId,
    nospam: [u8, ..4],
    #[allow(dead_code)]
    checksum: [u8, ..2],
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
        let _ = self.id.fmt(fmt);
        try!(write!(fmt, "{:x}", self.nospam[0]));
        try!(write!(fmt, "{:x}", self.nospam[1]));
        try!(write!(fmt, "{:x}", self.nospam[2]));
        try!(write!(fmt, "{:x}", self.nospam[3]));
        let check = self.checksum();
        try!(write!(fmt, "{:x}", check[0]));
        try!(write!(fmt, "{:x}", check[1]));
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
            buf[i] = (buf[i] << 4) + match s[2*i + j] as char {
                c @ '0' .. '9' => (c as u8) - ('0' as u8),
                c @ 'a' .. 'f' => (c as u8) - ('a' as u8) + 10,
                c @ 'A' .. 'F' => (c as u8) - ('A' as u8) + 10,
                _              => return Err(()),
            }
        }
    }
    return Ok(());
}

pub struct ClientId {
    pub raw: [u8, ..ID_CLIENT_SIZE],
}

impl fmt::Show for ClientId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for &n in self.raw.iter() {
            try!(write!(fmt, "{:x}", n));
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

pub enum ConnectionStatus {
    Online,
    Offline,
}

#[repr(C)]
pub enum UserStatus {
    UserStatusNone = TOX_USERSTATUS_NONE,
    UserStatusAway = TOX_USERSTATUS_AWAY,
    UserStatusBusy = TOX_USERSTATUS_BUSY,
}

#[repr(C)]
pub enum ChatChange {
    ChatChangePeerAdd = TOX_CHAT_CHANGE_PEER_ADD,
    ChatChangePeerDel = TOX_CHAT_CHANGE_PEER_DEL,
    ChatChangePeerName = TOX_CHAT_CHANGE_PEER_NAME,
}

#[repr(C)]
pub enum ControlType {
    ControlAccept = TOX_FILECONTROL_ACCEPT,
    ControlPause = TOX_FILECONTROL_PAUSE,
    ControlKill = TOX_FILECONTROL_KILL,
    ControlFinished = TOX_FILECONTROL_FINISHED,
    ControlResumeBroken = TOX_FILECONTROL_RESUME_BROKEN,
}

#[repr(C)]
pub enum Faerr {
    FaerrToolong = TOX_FAERR_TOOLONG,
    FaerrNomessage = TOX_FAERR_NOMESSAGE,
    FaerrOwnkey = TOX_FAERR_OWNKEY,
    FaerrAlreadysent = TOX_FAERR_ALREADYSENT,
    FaerrUnknown = TOX_FAERR_UNKNOWN,
    FaerrBadchecksum = TOX_FAERR_BADCHECKSUM,
    FaerrSetnewnospam = TOX_FAERR_SETNEWNOSPAM,
    FaerrNomem = TOX_FAERR_NOMEM,
}

pub enum TransferType {
    Receiving,
    Sending,
}

pub struct Tox {
    events: Receiver<Event>,
    control: SyncSender<backend::Control>,
}

macro_rules! forward {
    ($name:expr, ($($pp:ident),+), ->) => {
        {
            let (snd, rcv) = channel();
            self.control.send($name($($pp),*, snd));
            rcv.recv()
        }
    };
    ($name:expr, ->) => {
        {
            let (snd, rcv) = channel();
            self.control.send($name(snd));
            rcv.recv()
        }
    };
    ($name:expr, ($($pp:ident),+)) => {
        {
            self.control.send($name($($pp),*));
        }
    };
    ($name:expr) => {
            self.control.send($name);
    };

}

impl Drop for Tox {
    fn drop(&mut self) {
        forward!(backend::Kill)
    }
}

impl Tox {
    pub fn get_address(&self) -> Address {
        forward!(backend::GetAddress, ->)
    }

    pub fn add_friend(&self, address: Box<Address>,
                      msg: String) -> Result<i32, Faerr> {
        forward!(backend::AddFriend, (address, msg), ->)
    }

    pub fn add_friend_norequest(&self, client_id: Box<ClientId>) -> Result<i32, ()> {
        forward!(backend::AddFriendNorequest, (client_id), ->)
    }

    pub fn get_friend_number(&self, client_id: Box<ClientId>) -> Result<i32, ()> {
        forward!(backend::GetFriendNumber, (client_id), ->)
    }

    pub fn get_client_id(&self, friendnumber: i32) -> Result<Box<ClientId>, ()> {
        forward!(backend::GetClientId, (friendnumber), ->)
    }

    pub fn del_friend(&self, friendnumber: i32) -> Result<(),()> {
        forward!(backend::DelFriend, (friendnumber), ->)
    }

    pub fn get_friend_connection_status(
            &self,
            friendnumber: i32) -> Result<ConnectionStatus, ()> {
        forward!(backend::GetFriendConnectionStatus, (friendnumber), ->)
    }

    pub fn friend_exists(&self, friendnumber: i32) -> bool {
        forward!(backend::FriendExists, (friendnumber), ->)
    }

    pub fn send_message(&self, friendnumber: i32,
                        msg: String) -> Result<u32, ()> {
        forward!(backend::SendMessage, (friendnumber, msg), ->)
    }

    pub fn send_message_withid(&self, friendnumber: i32, theid: u32,
                               msg: String) -> Result<u32, ()> {
        forward!(backend::SendMessageWithid, (friendnumber, theid, msg), ->)
    }

    pub fn send_action(&self, friendnumber: i32, action: String) -> Result<u32, ()> {
        forward!(backend::SendAction, (friendnumber, action), ->)
    }

    pub fn send_action_withid(&self, friendnumber: i32, theid: u32,
                              action: String) -> Result<u32, ()> {
        forward!(backend::SendActionWithid, (friendnumber, theid, action), ->)
    }

    pub fn set_name(&self, name: String) -> Result<(),()> {
        forward!(backend::SetName, (name), ->)
    }

    pub fn get_self_name(&self) -> Result<String, ()> {
        forward!(backend::GetSelfName, ->)
    }

    pub fn get_name(&self, friendnumber: i32) -> Result<String, ()> {
        forward!(backend::GetName, (friendnumber), ->)
    }

    pub fn set_status_message(&self, status: String) -> Result<(),()> {
        forward!(backend::SetStatusMessage, (status), ->)
    }

    pub fn set_user_status(&self, userstatus: UserStatus) -> Result<(), ()> {
        forward!(backend::SetUserStatus, (userstatus), ->)
    }

    pub fn get_status_message(&self, friendnumber: i32) -> Result<String, ()> {
        forward!(backend::GetStatusMessage, (friendnumber), ->)
    }

    pub fn get_self_status_message(&self) -> Result<String, ()> {
        forward!(backend::GetSelfStatusMessage, ->)
    }

    pub fn get_user_status(&self, friendnumber: i32) -> Result<UserStatus, ()> {
        forward!(backend::GetUserStatus, (friendnumber), ->)
    }

    pub fn get_self_user_status(&self) -> Result<UserStatus, ()> {
        forward!(backend::GetSelfUserStatus, ->)
    }

    pub fn get_last_online(&self, friendnumber: i32) -> Result<u64, ()> {
        forward!(backend::GetLastOnline, (friendnumber), ->)
    }

    pub fn set_user_is_typing(&self, friendnumber: i32,
                              is_typing: bool) -> Result<(), ()> {
        forward!(backend::SetUserIsTyping, (friendnumber, is_typing), ->)
    }

    pub fn get_is_typing(&self, friendnumber: i32) -> bool {
        forward!(backend::GetIsTyping, (friendnumber), ->)
    }

    pub fn set_sends_receipts(&self, friendnumber: i32, yesno: bool) {
        forward!(backend::SetSendsReceipts, (friendnumber, yesno))
    }

    pub fn count_friendlist(&self) -> u32 {
        forward!(backend::CountFriendlist, ->)
    }

    pub fn get_num_online_friends(&self) -> u32 {
        forward!(backend::GetNumOnlineFriends, ->)
    }

    pub fn get_friendlist(&self) -> Vec<i32> {
        forward!(backend::GetFriendlist, ->)
    }

    pub fn get_nospam(&self) -> [u8, ..4] {
        forward!(backend::GetNospam, ->)
    }

    pub fn set_nospam(&self, nospam: [u8, ..4]) {
        forward!(backend::SetNospam, (nospam))
    }

    pub fn add_groupchat(&self) -> Result<i32, ()> {
        forward!(backend::AddGroupchat, ->)
    }

    pub fn del_groupchat(&self, groupnumber: i32) -> Result<(),()> {
        forward!(backend::DelGroupchat, (groupnumber), ->)
    }

    pub fn group_peername(&self, groupnumber: i32,
                          peernumber: i32) -> Result<String, ()> {
        forward!(backend::GroupPeername, (groupnumber, peernumber), ->)
    }

    pub fn invite_friend(&self, friendnumber: i32,
                         groupnumber: i32) -> Result<(), ()> {
        forward!(backend::InviteFriend, (friendnumber, groupnumber), ->)
    }

    pub fn join_groupchat(&self, friendnumber: i32,
                          fgpk: Box<ClientId>) -> Result<i32, ()> {
        forward!(backend::JoinGroupchat, (friendnumber, fgpk), ->)
    }

    pub fn group_message_send(&self, groupnumber: i32,
                              message: String) -> Result<(), ()> {
        forward!(backend::GroupMessageSend, (groupnumber, message), ->)
    }

    pub fn group_action_send(&self, groupnumber: i32,
                             action: String) -> Result<(), ()> {
        forward!(backend::GroupActionSend, (groupnumber, action), ->)
    }

    pub fn group_number_peers(&self, groupnumber: i32) -> Result<i32, ()> {
        forward!(backend::GroupNumberPeers, (groupnumber), ->)
    }

    pub fn group_get_names(&self,
                           groupnumber: i32) -> Result<Vec<Option<String>>, ()> {
        forward!(backend::GroupGetNames, (groupnumber), ->)
    }

    pub fn get_chatlist(&self) -> Vec<i32> {
        forward!(backend::GetChatlist, ->)
    }

    pub fn new_file_sender(&self, friendnumber: i32, filesize: u64,
                           filename: Path) -> Result<i32, ()> {
        forward!(backend::NewFileSender, (friendnumber, filesize, filename), ->)
    }

    pub fn file_send_control(&self, friendnumber: i32, send_receive: TransferType,
                             filenumber: u8, message_id: u8,
                             data: Vec<u8>) -> Result<(), ()> {
        forward!(backend::FileSendControl, (friendnumber, send_receive, filenumber,
                                            message_id, data), ->)
    }

    pub fn file_send_data(&self, friendnumber: i32, filenumber: u8,
                          data: Vec<u8>) -> Result<(), ()> {
        forward!(backend::FileSendData, (friendnumber, filenumber, data), ->)
    }

    pub fn file_data_size(&self, friendnumber: i32) -> Result<i32, ()> {
        forward!(backend::FileDataSize, (friendnumber), ->)
    }

    pub fn file_data_remaining(&self, friendnumber: i32, filenumber: u8,
                               send_receive: TransferType) -> Result<u64, ()> {
        forward!(backend::FileDataRemaining, (friendnumber, filenumber, send_receive), ->)
    }

    pub fn bootstrap_from_address(&self, address: String, ipv6: bool, port: u16,
                                  public_key: Box<ClientId>) -> Result<(), ()> {
        forward!(backend::BootstrapFromAddress, (address, ipv6, port, public_key), ->)
    }

    pub fn is_connected(&self) -> bool {
        forward!(backend::Isconnected, ->)
    }

    pub fn new(ipv6enabled: bool) -> Option<Tox> {
        let (ctrl, events) = match backend::Backend::new(ipv6enabled) {
            Some(x) => x,
            None => return None,
        };
        Some(Tox {
            events: events,
            control: ctrl,
        })
    }

    pub fn save(&self) -> Vec<u8> {
        forward!(backend::Save, ->)
    }

    pub fn load(&self, data: Vec<u8>) -> Result<(), ()> {
        forward!(backend::Load, (data), ->)
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

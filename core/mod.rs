use self::ll::*;

mod ll;
mod backend;

pub static MAX_NAME_LENGTH: uint = 128u;
pub static MAX_MESSAGE_LENGTH: uint = 1368u;
pub static MAX_STATUSMESSAGE_LENGTH: uint = 1007u;
pub static ID_CLIENT_SIZE: uint = 32u;
pub static ADDRESS_SIZE: uint = ID_CLIENT_SIZE + 6u;

pub struct Address {
    id: ClientId,
    no_spam: [u8, ..4],
    checksum: [u8, ..2],
}

pub struct ClientId {
    raw: [u8, ..ID_CLIENT_SIZE],
}

enum ConnectionStatus {
    Online,
    Offline,
}

#[repr(C)]
enum UserStatus {
    UserStatusNone = TOX_USERSTATUS_NONE,
    UserStatusAway = TOX_USERSTATUS_AWAY,
    UserStatusBusy = TOX_USERSTATUS_BUSY,
}

#[repr(C)]
enum ChatChange {
    ChatChangePeerAdd = TOX_CHAT_CHANGE_PEER_ADD,
    ChatChangePeerDel = TOX_CHAT_CHANGE_PEER_DEL,
    ChatChangePeerName = TOX_CHAT_CHANGE_PEER_NAME,
}

#[repr(C)]
enum ControlType {
    ControlAccept = TOX_FILECONTROL_ACCEPT,
    ControlPause = TOX_FILECONTROL_PAUSE,
    ControlKill = TOX_FILECONTROL_KILL,
    ControlFinished = TOX_FILECONTROL_FINISHED,
    ControlResumeBroken = TOX_FILECONTROL_RESUME_BROKEN,
}

enum TransferType {
    Receiving,
    Sending,
}

/*
struct Tox {
    backend: BackendCtrl, 
}

impl Tox {
    pub fn get_address(&mut self) -> Box<Address> {
        self.backend.get_address();
    }

    pub fn add_friend(&mut self, address: Box<Address>, msg: String) -> i32 {
    }
    pub fn add_friend_norequest(&mut self, client_id: Box<ClientId>) -> i32 { }
    pub fn get_friend_number(&mut self, client_id: Box<ClientId>) -> i32 { }
    pub fn get_client_id(&mut self, friendnumber: i32) -> Result<Box<ClientId>, ()> { }
    pub fn del_friend(&mut self, friendnumber: i32) -> Result<(),()> { }
    pub fn get_friend_connection_status(&mut self, friendnumber: i32) -> Result<ConnectionStatus, ()> { }
    pub fn friend_exists(&mut self, friendnumber: i32) -> bool { }
    pub fn send_message(&mut self, friendnumber: i32, msg: String) -> u32 { }
    pub fn send_message_withid(&mut self, friendnumber: i32, theid: u32, message: String) -> u32 { }
    pub fn send_action(&mut self, friendnumber: i32, action: String) -> u32 { }
    pub fn send_action_withid(&mut self, friendnumber: i32, theid: u32, action: String) -> u32 { }
    pub fn set_name(&mut self, name: String) -> Result<(),()> { }
    pub fn get_self_name(&mut self) -> Result<String, ()> { }
    pub fn get_name(&mut self, friendnumber: i32) -> Result<String, ()> { }
    pub fn set_status_message(&mut self, status: String) -> Result<(),()> { }
    pub fn set_user_status(&mut self, userstatus: UserStatus) -> Result<(), ()> { }
    pub fn get_status_message(&mut self, friendnumber: i32) -> Result<String, ()> { }
    pub fn get_self_status_message(&mut self) -> Result<String, ()> { }
    pub fn get_user_status(&mut self, friendnumber: i32) -> Result<UserStatus, ()> { }
    pub fn get_self_user_status(&mut self) -> Result<UserStatus, ()> { }
    pub fn get_last_online(&mut self, friendnumber: i32) -> Result<u64, ()> { }
    pub fn set_user_is_typing(&mut self, friendnumber: i32, is_typing: bool) -> Result<(), ()> { }
    pub fn get_is_typing(&mut self, friendnumber: i32) -> bool { }
    pub fn set_sends_receipts(&mut self, friendnumber: i32, yesno: bool) { }
    pub fn count_friendlist(&mut self) -> u32 { }
    pub fn get_num_online_friends(&mut self) -> u32 { }
    pub fn get_friendlist(&mut self) -> Vec<u32> { }
    pub fn get_nospam(&mut self) -> [u8, ..4] { }
    pub fn set_nospam(&mut self, nospam: [u8, ..4]) { }
    pub fn add_groupchat(&mut self) -> Result<i32, ()> { }
    pub fn del_groupchat(&mut self, groupnumber: i32) -> Result<(),()> { }
    pub fn group_peername(&mut self, groupnumber: i32, peernumber: i32) -> Result<String, ()> { }
    pub fn invite_friend(&mut self, friendnumber: i32, groupnumber: i32) -> Result<(), ()> { }
    pub fn join_groupchat(&mut self, friendnumber: i32, friend_group_public_key: Box<ClientId>) -> Result<i32, ()> { }
    pub fn group_message_send(&mut self, groupnumber: i32, message: String) -> Result<(), ()> { }
    pub fn group_action_send(&mut self, groupnumber: i32, action: String) -> Result<(), ()> { }
    pub fn group_number_peers(&mut self, groupnumber: i32) -> Result<i32, ()> { }
    pub fn group_get_names(&mut self, groupnumber: i32) -> Result<Vec<String>, ()> { }
    pub fn get_chatlist(&mut self) -> Vec<i32> { }
    pub fn new_file_sender(&mut self, friendnumber: i32, filesize: u64, filename: Path) -> Result<i32, ()> { }
    pub fn file_send_control(&mut self, friendnumber: i32, send_receive: u8, filenumber: u8, message_id: u8, data: Vec<u8>) -> Result<(), ()> { }
    pub fn file_send_data(&mut self, friendnumber: i32, filenumber: u8, data: Vec<u8>) -> Result<(), ()> { }
    pub fn file_data_size(&mut self, friendnumber: i32) -> Result<i32, ()> { }
    pub fn file_data_remaining(&mut self, friendnumber: i32, filenumber: u8, send_receive: u8) -> Result<u64, ()> { }
    pub fn bootstrap_from_address(&mut self, address: String, ipv6enabled: u8, port: u16, public_key: Box<ClientId>) -> Result<(), ()> { }
    pub fn is_connected(&mut self) -> bool { }
    pub fn new(ipv6enabled: bool) -> Tox { }
    pub fn save(&mut self) -> Vec<u8> { }
    pub fn load(&mut self, data: Vec<u8>) -> Result<(), ()> { }
}
*/

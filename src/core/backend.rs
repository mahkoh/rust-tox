use std;
use std::{ptr, task};
use std::io::{timer};
use std::num::{Int};
use std::raw::{Slice};
use std::mem::{transmute};
use std::c_vec::{CVec};
use core::ll::*;
use core::{Address, ClientId, Event, ConnectionStatus,
           UserStatus, Faerr, TransferType, AvatarFormat,
           MAX_NAME_LENGTH, AVATAR_MAX_DATA_LENGTH, Hash};
use core::Event::*;
use core::ConnectionStatus::*;
use core::UserStatus::*;
use core::ChatChange::*;
use core::ControlType::*;
use core::Faerr::*;
use core::TransferType::*;
use core::AvatarFormat::*;

use libc::{c_void, c_uint};

pub enum Control {
    GetAddress(Sender<Address>),
    AddFriend(Box<Address>, String, Sender<Result<i32, Faerr>>),
    AddFriendNorequest(Box<ClientId>, Sender<Result<i32, ()>>),
    GetFriendNumber(Box<ClientId>, Sender<Result<i32, ()>>),
    GetClientId(i32, Sender<Result<Box<ClientId>, ()>>),
    DelFriend(i32, Sender<Result<(),()>>),
    GetFriendConnectionStatus(i32, Sender<Result<ConnectionStatus, ()>>),
    FriendExists(i32, Sender<bool>),
    SendMessage(i32, String, Sender<Result<u32, ()>>),
    SendAction(i32, String, Sender<Result<u32, ()>>),
    SetName(String, Sender<Result<(),()>>),
    GetSelfName(Sender<Result<String, ()>>),
    GetName(i32, Sender<Result<String, ()>>),
    SetStatusMessage(String, Sender<Result<(), ()>>),
    SetUserStatus(UserStatus, Sender<Result<(), ()>>),
    GetStatusMessage(i32, Sender<Result<String, ()>>),
    GetSelfStatusMessage(Sender<Result<String, ()>>),
    GetUserStatus(i32, Sender<Result<UserStatus, ()>>),
    GetSelfUserStatus(Sender<Result<UserStatus, ()>>),
    GetLastOnline(i32, Sender<Result<u64, ()>>),
    SetUserIsTyping(i32, bool, Sender<Result<(), ()>>),
    GetIsTyping(i32, Sender<bool>),
    SetSendsReceipts(i32, bool),
    CountFriendlist(Sender<u32>),
    GetNumOnlineFriends(Sender<u32>),
    GetFriendlist(Sender<Vec<i32>>),
    GetNospam(Sender<[u8, ..4]>),
    SetNospam([u8, ..4]),
    AddGroupchat(Sender<Result<i32, ()>>),
    DelGroupchat(i32, Sender<Result<(), ()>>),
    GroupPeername(i32, i32, Sender<Result<String, ()>>),
    InviteFriend(i32, i32, Sender<Result<(), ()>>),
    JoinGroupchat(i32, Vec<u8>, Sender<Result<i32, ()>>),
    GroupMessageSend(i32, String, Sender<Result<(), ()>>),
    GroupActionSend(i32, String, Sender<Result<(), ()>>),
    GroupNumberPeers(i32, Sender<Result<i32, ()>>),
    GroupGetNames(i32, Sender<Result<Vec<Option<String>>, ()>>),
    CountChatlist(Sender<u32>),
    GetChatlist(Sender<Vec<i32>>),
    SetAvatar(AvatarFormat, Vec<u8>, Sender<Result<(), ()>>),
    UnsetAvatar,
    GetSelfAvatar(Sender<Result<(AvatarFormat, Vec<u8>, Hash), ()>>),
    RequestAvatarInfo(i32, Sender<Result<(), ()>>),
    RequestAvatarData(i32, Sender<Result<(), ()>>),
    SendAvatarInfo(i32, Sender<Result<(), ()>>),
    NewFileSender(i32, u64, Path, Sender<Result<i32, ()>>),
    FileSendControl(i32, TransferType, u8, u8, Vec<u8>, Sender<Result<(), ()>>),
    FileSendData(i32, u8, Vec<u8>, Sender<Result<(), ()>>),
    FileDataSize(i32, Sender<Result<i32, ()>>),
    FileDataRemaining(i32, u8, TransferType, Sender<Result<u64, ()>>),
    BootstrapFromAddress(String, u16, Box<ClientId>, Sender<Result<(), ()>>),
    Isconnected(Sender<bool>),
    Kill,
    Save(Sender<Vec<u8>>),
    Load(Vec<u8>, Sender<Result<(), ()>>),
}

pub struct Backend {
    raw: *mut Tox,
    internal: Box<Internal>,
    control: Receiver<Control>,
}

impl Drop for Backend {
    fn drop(&mut self) {
        unsafe { tox_kill(self.raw); }
    }
}

impl Backend {
    fn get_address(&mut self) -> Address {
        let mut adr: Address = unsafe { std::mem::uninitialized() };
        unsafe { tox_get_address(&*self.raw, &mut adr as *mut _ as *mut _); }
        adr
    }

    fn add_friend(&mut self, address: Box<Address>, msg: String) -> Result<i32, Faerr> {
        let res = unsafe {
            tox_add_friend(self.raw, &*address as *const _ as *const _,
                           msg.as_bytes().as_ptr(), msg.len() as u16)
        };
        match res {
            TOX_FAERR_TOOLONG => Err(FaerrToolong),
            TOX_FAERR_NOMESSAGE => Err(FaerrNomessage),
            TOX_FAERR_OWNKEY => Err(FaerrOwnkey),
            TOX_FAERR_ALREADYSENT => Err(FaerrAlreadysent),
            TOX_FAERR_UNKNOWN => Err(FaerrUnknown),
            TOX_FAERR_BADCHECKSUM => Err(FaerrBadchecksum),
            TOX_FAERR_SETNEWNOSPAM => Err(FaerrSetnewnospam),
            TOX_FAERR_NOMEM => Err(FaerrNomem),
            n if n >= 0 => Ok(n),
            _ => Err(FaerrUnknown),
        }
    }

    fn add_friend_norequest(&mut self, client_id: Box<ClientId>) -> Result<i32, ()> {
        match unsafe { tox_add_friend_norequest(self.raw, client_id.raw.as_ptr()) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn get_friend_number(&mut self, client_id: Box<ClientId>) -> Result<i32, ()> {
        let res = unsafe {
            tox_get_friend_number(&*self.raw, client_id.raw.as_ptr())
        };
        match res {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn get_client_id(&mut self, friendnumber: i32) -> Result<Box<ClientId>, ()> {
        let mut client: ClientId = unsafe { std::mem::uninitialized() };
        let res = unsafe {
            tox_get_client_id(&*self.raw, friendnumber, client.raw.as_mut_ptr())
        };
        match res {
            -1 => Err(()),
            _ => Ok(box client),
        }
    }

    fn del_friend(&mut self, friendnumber: i32) -> Result<(),()> {
        match unsafe { tox_del_friend(self.raw, friendnumber) } {
            -1 => Err(()),
            _ => Ok(()),
        }
    }

    fn get_friend_connection_status(
            &mut self,
            friendnumber: i32) -> Result<ConnectionStatus, ()> {
        match unsafe { tox_get_friend_connection_status(&*self.raw, friendnumber) } {
            1 => Ok(Online),
            0 => Ok(Offline),
            _ => Err(()),
        }
    }

    fn friend_exists(&mut self, friendnumber: i32) -> bool {
        match unsafe { tox_friend_exists(&*self.raw, friendnumber) } {
            1 => true,
            _ => false,
        }
    }

    fn send_message(&mut self, friendnumber: i32, mut msg: String) -> Result<u32, ()> {
        let res = unsafe {
            tox_send_message(self.raw, friendnumber,
                             msg.as_mut_vec().as_ptr(), msg.len() as u32)
        };
        match res {
            0 => Err(()),
            n => Ok(n),
        }
    }

    fn send_action(&mut self, friendnumber: i32, mut action: String) -> Result<u32, ()> {
        let res = unsafe {
            tox_send_action(self.raw, friendnumber,
                            action.as_mut_vec().as_ptr(), action.len() as u32)
        };
        match res {
            0 => Err(()),
            n => Ok(n),
        }
    }

    fn set_name(&mut self, mut name: String) -> Result<(),()> {
        let res = unsafe {
            tox_set_name(self.raw, name.as_mut_vec().as_ptr(), name.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn get_self_name(&mut self) -> Result<String, ()> {
        let mut name = Vec::with_capacity(MAX_NAME_LENGTH);
        let res = unsafe {
            let len = tox_get_self_name(&*self.raw, name.as_mut_ptr());
            name.set_len(len as uint);
            len
        };
        match res {
            0 => Err(()),
            _ => match String::from_utf8(name) {
                Ok(name) => Ok(name),
                _ => Err(()),
            },
        }
    }

    fn get_name(&mut self, friendnumber: i32) -> Result<String, ()> {
        let mut name = Vec::with_capacity(MAX_NAME_LENGTH);
        let res = unsafe {
            let len = tox_get_name(&*self.raw, friendnumber, name.as_mut_ptr());
            // len might be -1 but it doesn't matter if we don't return name.
            name.set_len(len as uint);
            len
        };
        match res {
            -1 => Err(()),
            _ => match String::from_utf8(name) {
                Ok(name) => Ok(name),
                _ => Err(()),
            },
        }
    }

    fn set_status_message(&mut self, mut status: String) -> Result<(),()> {
        let res = unsafe {
            tox_set_status_message(self.raw, status.as_mut_vec().as_ptr(),
                                   status.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn set_user_status(&mut self, userstatus: UserStatus) -> Result<(), ()> {
        match unsafe { tox_set_user_status(self.raw, userstatus as u8) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn get_status_message(&mut self, friendnumber: i32) -> Result<String, ()> {
        let size = unsafe { tox_get_status_message_size(&*self.raw, friendnumber) };
        let size = match size {
            -1 => return Err(()),
            _ => size,
        };
        let mut status = Vec::with_capacity(size as uint);
        let size = unsafe {
            let len = tox_get_status_message(&*self.raw, friendnumber, status.as_mut_ptr(),
                                             size as u32);
            status.set_len(len as uint);
            len
        };
        match size {
            -1 => return Err(()),
            _ => match String::from_utf8(status) {
                Ok(status) => Ok(status),
                _ => return Err(()),
            },
        }
    }

    fn get_self_status_message(&mut self) -> Result<String, ()> {
        let size = unsafe { tox_get_self_status_message_size(&*self.raw) };
        let size = match size {
            -1 => return Err(()),
            _ => size as u32,
        };
        let mut status = Vec::with_capacity(size as uint);
        let size = unsafe {
            let len = tox_get_self_status_message(&*self.raw, status.as_mut_ptr(), size);
            status.set_len(len as uint);
            len
        };
        match size {
            -1 => return Err(()),
            _ => match String::from_utf8(status) {
                Ok(status) => Ok(status),
                _ => return Err(()),
            },
        }
    }

    fn get_user_status(&mut self, friendnumber: i32) -> Result<UserStatus, ()> {
        match unsafe { tox_get_user_status(&*self.raw, friendnumber) as u32 } {
            TOX_USERSTATUS_AWAY => Ok(UserStatusAway),
            TOX_USERSTATUS_NONE => Ok(UserStatusNone),
            TOX_USERSTATUS_BUSY => Ok(UserStatusBusy),
            _ => Err(())
        }
    }

    fn get_self_user_status(&mut self) -> Result<UserStatus, ()> {
        match unsafe { tox_get_self_user_status(&*self.raw) as u32 } {
            TOX_USERSTATUS_AWAY => Ok(UserStatusAway),
            TOX_USERSTATUS_NONE => Ok(UserStatusNone),
            TOX_USERSTATUS_BUSY => Ok(UserStatusBusy),
            _ => Err(())
        }
    }

    fn get_last_online(&mut self, friendnumber: i32) -> Result<u64, ()> {
        match unsafe { tox_get_last_online(&*self.raw, friendnumber) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn set_user_is_typing(&mut self, friendnumber: i32,
                              is_typing: bool) -> Result<(), ()> {
        let raw = unsafe {
            tox_set_user_is_typing(self.raw, friendnumber, is_typing as u8)
        };
        match raw {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn get_is_typing(&mut self, friendnumber: i32) -> bool {
        match unsafe { tox_get_is_typing(&*self.raw, friendnumber) } {
            0 => false,
            _ => true,
        }
    }

    fn count_friendlist(&mut self) -> u32 {
        unsafe { tox_count_friendlist(&*self.raw) }
    }

    fn get_num_online_friends(&mut self) -> u32 {
        unsafe { tox_get_num_online_friends(&*self.raw) }
    }

    fn get_friendlist(&mut self) -> Vec<i32> {
        let size = self.count_friendlist();
        let mut vec = Vec::with_capacity(size as uint);
        unsafe {
            let len = tox_get_friendlist(&*self.raw, vec.as_mut_ptr(), size);
            vec.set_len(len as uint);
        }
        vec
    }

    fn get_nospam(&mut self) -> [u8, ..4] {
        unsafe { std::mem::transmute(tox_get_nospam(&*self.raw).to_be()) }
    }

    fn set_nospam(&mut self, nospam: [u8, ..4]) {
        unsafe { tox_set_nospam(self.raw, Int::from_be(std::mem::transmute(nospam))); }
    }

    fn add_groupchat(&mut self) -> Result<i32, ()> {
        match unsafe { tox_add_groupchat(self.raw) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn del_groupchat(&mut self, groupnumber: i32) -> Result<(),()> {
        match unsafe { tox_del_groupchat(self.raw, groupnumber) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn group_peername(&mut self, groupnumber: i32,
                          peernumber: i32) -> Result<String, ()> {
        let mut vec = Vec::with_capacity(MAX_NAME_LENGTH);
        let len = unsafe {
            let len = tox_group_peername(&*self.raw, groupnumber, peernumber,
                                         vec.as_mut_ptr());
            vec.set_len(len as uint);
            len
        };
        match len {
            -1 => Err(()),
            _ => match String::from_utf8(vec) {
                Ok(name) => Ok(name),
                _ => Err(()),
            }
        }
    }

    fn invite_friend(&mut self, friendnumber: i32, groupnumber: i32) -> Result<(), ()> {
        match unsafe { tox_invite_friend(self.raw, friendnumber, groupnumber) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn join_groupchat(&mut self, friendnumber: i32,
                      data: Vec<u8>) -> Result<i32, ()> {
        let res = unsafe {
            tox_join_groupchat(self.raw, friendnumber, data.as_ptr(), data.len() as u16)
        };
        match res {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn group_message_send(&mut self, groupnumber: i32,
                          mut msg: String) -> Result<(), ()> {
        let res = unsafe {
            tox_group_message_send(self.raw, groupnumber, msg.as_mut_vec().as_ptr(),
                                   msg.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn group_action_send(&mut self, groupnumber: i32, mut act: String) -> Result<(), ()> {
        let res = unsafe {
            tox_group_action_send(self.raw, groupnumber, act.as_mut_vec().as_ptr(),
                                  act.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn group_number_peers(&mut self, groupnumber: i32) -> Result<i32, ()> {
        match unsafe { tox_group_number_peers(&*self.raw, groupnumber) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn group_get_names(&mut self,
                           groupnumber: i32) -> Result<Vec<Option<String>>, ()> {
        let num = match self.group_number_peers(groupnumber) {
            Ok(n) => n as uint,
            _ => return Err(()),
        };
        let mut names = Vec::with_capacity(num);
        let mut lengths = Vec::with_capacity(num);
        let len = unsafe {
            let len = tox_group_get_names(&*self.raw, groupnumber, names.as_mut_ptr(),
                                          lengths.as_mut_ptr(), num as u16);
            names.set_len(len as uint);
            lengths.set_len(len as uint);
            len
        };
        if len == -1 {
            return Err(());
        }
        let mut real_names = Vec::with_capacity(len as uint);
        for (name, &length) in names.iter().zip(lengths.iter()) {
            match std::str::from_utf8(name.slice_to(length as uint)) {
                Some(s) => real_names.push(Some(s.to_string())),
                _ => real_names.push(None),
            }
        }
        Ok(real_names)
    }

    fn count_chatlist(&mut self) -> u32 {
        unsafe { tox_count_chatlist(&*self.raw) }
    }

    fn get_chatlist(&mut self) -> Vec<i32> {
        let num = unsafe { tox_count_chatlist(&*self.raw) };
        let mut vec = Vec::with_capacity(num as uint);
        unsafe {
            let num = tox_get_chatlist(&*self.raw, vec.as_mut_ptr(), num);
            vec.set_len(num as uint);
        }
        vec
    }

    fn set_avatar(&mut self, format: AvatarFormat, data: Vec<u8>) -> Result<(), ()> {
        let res = unsafe {
            tox_set_avatar(self.raw, format as u8, data.as_ptr(), data.len() as u32)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn unset_avatar(&mut self) {
        unsafe { tox_unset_avatar(self.raw); }
    }

    fn get_self_avatar(&mut self) -> Result<(AvatarFormat, Vec<u8>, Hash), ()> {
        let mut data = Vec::with_capacity(AVATAR_MAX_DATA_LENGTH);
        let mut hash: Hash = unsafe { std::mem::uninitialized() };
        let mut format = 0;
        let mut length = 0;
        let res = unsafe {
            tox_get_self_avatar(self.raw as *const _, &mut format, data.as_mut_ptr(),
                                &mut length, AVATAR_MAX_DATA_LENGTH as u32,
                                hash.hash.as_mut_ptr())
        };
        if res == -1 {
            return Err(());
        }
        unsafe { data.set_len(length as uint); }
        data.shrink_to_fit();
        let format = match format as c_uint {
            TOX_AVATAR_FORMAT_NONE => AvatarNone,
            TOX_AVATAR_FORMAT_PNG => AvatarPNG,
            _ => return Err(()),
        };
        Ok((format, data, hash))
    }

    fn request_avatar_info(&self, friendnumber: i32) -> Result<(), ()> {
        let res = unsafe {
            tox_request_avatar_info(self.raw as *const _, friendnumber)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn request_avatar_data(&self, friendnumber: i32) -> Result<(), ()> {
        let res = unsafe {
            tox_request_avatar_data(self.raw as *const _, friendnumber)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn send_avatar_info(&mut self, friendnumber: i32) -> Result<(), ()> {
        let res = unsafe {
            tox_send_avatar_info(self.raw, friendnumber)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn new_file_sender(&mut self, friendnumber: i32, filesize: u64,
                           filename: Path) -> Result<i32, ()> {
        let filename = filename.into_vec();
        let res = unsafe {
            tox_new_file_sender(self.raw, friendnumber, filesize,
                                filename.as_ptr(), filename.len() as u16)
        };
        match res {
            -1 => Err(()),
            n => Ok(n)
        }
    }

    fn file_send_control(&mut self, friendnumber: i32, send_receive: TransferType,
                         filenumber: u8, message_id: u8,
                         data: Vec<u8>) -> Result<(), ()> {
        let res = unsafe {
            tox_file_send_control(self.raw, friendnumber, send_receive as u8, filenumber,
                                  message_id, data.as_ptr(), data.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn file_send_data(&mut self, friendnumber: i32, filenumber: u8,
                      data: Vec<u8>) -> Result<(), ()> {
        let res = unsafe {
            tox_file_send_data(self.raw, friendnumber, filenumber, data.as_ptr(),
                               data.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn file_data_size(&mut self, friendnumber: i32) -> Result<i32, ()> {
        match unsafe { tox_file_data_size(&*self.raw, friendnumber) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn file_data_remaining(&mut self, friendnumber: i32, filenumber: u8,
                               send_receive: TransferType) -> Result<u64, ()> {
        let res = unsafe {
            tox_file_data_remaining(&*self.raw, friendnumber, filenumber,
                                    send_receive as u8)
        };
        match res {
            0 => Err(()),
            n => Ok(n),
        }
    }

    fn bootstrap_from_address(&mut self, mut address: String, port: u16,
                              public_key: Box<ClientId>) -> Result<(), ()> {
        let res = unsafe {
            address.as_mut_vec().push(0);
            tox_bootstrap_from_address(self.raw, address.as_bytes().as_ptr() as *const _,
                                       port, public_key.raw.as_ptr())
        };
        match res {
            1 => Ok(()),
            _ => Err(()),
        }
    }

    fn is_connected(&mut self) -> bool {
        match unsafe { tox_isconnected(&*self.raw) } {
            0 => false,
            _ => true,
        }
    }

    pub fn new(opts: &mut Tox_Options) -> Option<(SyncSender<Control>, Receiver<Event>)> {
        let tox = unsafe { tox_new(opts as *mut _) };
        if tox.is_null() {
            return None;
        }
        let (event_send, event_recv) = channel();
        let mut internal = box Internal { stop: false, events: event_send };
        unsafe {
            let ip = &mut *internal as *mut _ as *mut c_void;
            tox_callback_friend_request(        tox, Some(on_friend_request),        ip);
            tox_callback_friend_message(        tox, Some(on_friend_message),        ip);
            tox_callback_friend_action(         tox, Some(on_friend_action),         ip);
            tox_callback_name_change(           tox, Some(on_name_change),           ip);
            tox_callback_status_message(        tox, Some(on_status_message),        ip);
            tox_callback_user_status(           tox, Some(on_user_status),           ip);
            tox_callback_typing_change(         tox, Some(on_typing_change),         ip);
            tox_callback_read_receipt(          tox, Some(on_read_receipt),          ip);
            tox_callback_connection_status(     tox, Some(on_connection_status),     ip);
            tox_callback_group_invite(          tox, Some(on_group_invite),          ip);
            tox_callback_group_message(         tox, Some(on_group_message),         ip);
            tox_callback_group_action(          tox, Some(on_group_action),          ip);
            tox_callback_group_namelist_change( tox, Some(on_group_namelist_change), ip);
            tox_callback_file_send_request(     tox, Some(on_file_send_request),     ip);
            tox_callback_file_control(          tox, Some(on_file_control),          ip);
            tox_callback_file_data(             tox, Some(on_file_data),             ip);
            tox_callback_avatar_info(           tox, Some(on_avatar_info),           ip);
            tox_callback_avatar_data(           tox, Some(on_avatar_data),           ip);
        }
        let (control_send, control_recv) = sync_channel(1);
        let backend = Backend {
            raw: tox,
            internal: internal,
            control: control_recv,
        };
        task::spawn(proc() backend.run());
        Some((control_send, event_recv))
    }

    fn run(mut self) {
        'outer: loop {
            unsafe { tox_do(self.raw); }
            if self.internal.stop {
                break 'outer;
            }

            'inner: loop {
                match self.control.try_recv() {
                    Ok(Control::Kill) => break 'outer,
                    Ok(ctrl) => self.control(ctrl),
                    Err(std::comm::Disconnected) => break 'outer,
                    _ => break 'inner,
                }
            }

            let interval = unsafe { tox_do_interval(self.raw) as i64 };
            timer::sleep(::std::time::Duration::milliseconds(interval));
        }
    }

    fn control(&mut self, ctrl: Control) {
        match ctrl {
            Control::GetAddress(ret) =>
                ret.send(self.get_address()),
            Control::AddFriend(addr, msg, ret) =>
                ret.send(self.add_friend(addr, msg)),
            Control::AddFriendNorequest(id, ret) =>
                ret.send(self.add_friend_norequest(id)),
            Control::GetFriendNumber(id, ret) =>
                ret.send(self.get_friend_number(id)),
            Control::GetClientId(friend, ret) =>
                ret.send(self.get_client_id(friend)),
            Control::DelFriend(friend, ret) =>
                ret.send(self.del_friend(friend)),
            Control::GetFriendConnectionStatus(friend, ret) =>
                ret.send(self.get_friend_connection_status(friend)),
            Control::FriendExists(friend, ret) =>
                ret.send(self.friend_exists(friend)),
            Control::SendMessage(friend, msg, ret) =>
                ret.send(self.send_message(friend, msg)),
            Control::SendAction(friend, act, ret) =>
                ret.send(self.send_action(friend, act)),
            Control::SetName(name, ret) =>
                ret.send(self.set_name(name)),
            Control::GetSelfName(ret) =>
                ret.send(self.get_self_name()),
            Control::GetName(friend, ret) =>
                ret.send(self.get_name(friend)),
            Control::SetStatusMessage(msg, ret) =>
                ret.send(self.set_status_message(msg)),
            Control::SetUserStatus(status, ret) =>
                ret.send(self.set_user_status(status)),
            Control::GetStatusMessage(friend, ret) =>
                ret.send(self.get_status_message(friend)),
            Control::GetSelfStatusMessage(ret) =>
                ret.send(self.get_self_status_message()),
            Control::GetUserStatus(friend, ret) =>
                ret.send(self.get_user_status(friend)),
            Control::GetSelfUserStatus(ret) =>
                ret.send(self.get_self_user_status()),
            Control::GetLastOnline(friend, ret) =>
                ret.send(self.get_last_online(friend)),
            Control::SetUserIsTyping(friend, is, ret) =>
                ret.send(self.set_user_is_typing(friend, is)),
            Control::GetIsTyping(friend, ret) =>
                ret.send(self.get_is_typing(friend)),
//            Control::SetSendsReceipts(friend, send) =>
//                self.set_sends_receipts(friend, send),
            Control::CountFriendlist(ret) =>
                ret.send(self.count_friendlist()),
            Control::GetNumOnlineFriends(ret) =>
                ret.send(self.get_num_online_friends()),
            Control::GetFriendlist(ret) =>
                ret.send(self.get_friendlist()),
            Control::GetNospam(ret) =>
                ret.send(self.get_nospam()),
            Control::SetNospam(ns) =>
                self.set_nospam(ns),
            Control::AddGroupchat(ret) =>
                ret.send(self.add_groupchat()),
            Control::DelGroupchat(group, ret) =>
                ret.send(self.del_groupchat(group)),
            Control::GroupPeername(group, peer, ret) =>
                ret.send(self.group_peername(group, peer)),
            Control::InviteFriend(friend, group, ret) =>
                ret.send(self.invite_friend(friend, group)),
            Control::JoinGroupchat(friend, group, ret) =>
                ret.send(self.join_groupchat(friend, group)),
            Control::GroupMessageSend(group, msg, ret) =>
                ret.send(self.group_message_send(group, msg)),
            Control::GroupActionSend(group, action, ret) =>
                ret.send(self.group_action_send(group, action)),
            Control::GroupNumberPeers(group, ret) =>
                ret.send(self.group_number_peers(group)),
            Control::GroupGetNames(group, ret) =>
                ret.send(self.group_get_names(group)),
            Control::CountChatlist(ret) =>
                ret.send(self.count_chatlist()),
            Control::GetChatlist(ret) =>
                ret.send(self.get_chatlist()),
            Control::SetAvatar(format, data, ret) =>
                ret.send(self.set_avatar(format, data)),
            Control::UnsetAvatar =>
                self.unset_avatar(),
            Control::GetSelfAvatar(ret) =>
                ret.send(self.get_self_avatar()),
            Control::RequestAvatarInfo(friend, ret) =>
                ret.send(self.request_avatar_info(friend)),
            Control::RequestAvatarData(friend, ret) =>
                ret.send(self.request_avatar_data(friend)),
            Control::SendAvatarInfo(friend, ret) =>
                ret.send(self.send_avatar_info(friend)),
            Control::NewFileSender(friend, size, file, ret) =>
                ret.send(self.new_file_sender(friend, size, file)),
            Control::FileSendControl(friend, ty, num, msg, data, ret) =>
                ret.send(self.file_send_control(friend, ty, num, msg, data)),
            Control::FileSendData(friend, num, data, ret) =>
                ret.send(self.file_send_data(friend, num, data)),
            Control::FileDataSize(friend, ret) =>
                ret.send(self.file_data_size(friend)),
            Control::FileDataRemaining(friend, num, ty, ret) =>
                ret.send(self.file_data_remaining(friend, num, ty)),
            Control::BootstrapFromAddress(addr, port, id, ret) =>
                ret.send(self.bootstrap_from_address(addr, port, id)),
            Control::Isconnected(ret) =>
                ret.send(self.is_connected()),
            Control::Save(ret) =>
                ret.send(self.save()),
            Control::Load(data, ret) =>
                ret.send(self.load(data)),
            _ => unreachable!(),
        }
    }

    fn save(&mut self) -> Vec<u8> {
        let size = unsafe { tox_size(&*self.raw) as uint };
        let mut vec = Vec::with_capacity(size);
        unsafe {
            tox_save(&*self.raw, vec.as_mut_ptr());
            vec.set_len(size);
        }
        vec
    }

    fn load(&mut self, data: Vec<u8>) -> Result<(), ()> {
        match unsafe { tox_load(self.raw, data.as_ptr(), data.len() as u32) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }
}

struct Internal {
    stop: bool,
    events: Sender<Event>,
}

macro_rules! get_int {
    ($i:ident) => {
        unsafe {
            let internal = transmute::<_, &mut Internal>($i);
            if internal.stop { return }
            internal
        }
    }
}

macro_rules! send_or_stop {
    ($internal:ident, $event:expr) => {
        match $internal.events.send_opt($event) {
            Ok(()) => { },
            _ => $internal.stop = true,
        }
    }
}

macro_rules! parse_string {
    ($p:ident, $l:ident) => {
        {
            let slice = to_slice($p as *const u8, $l as uint);
            match std::str::from_utf8(slice) {
                Some(s) => s.to_string(),
                None => return,
            }
        }
    }
}

fn to_slice<'a, T>(p: *const T, l: uint) -> &'a [T] {
    unsafe { transmute(Slice { data: p, len: l }) }
}

extern fn on_friend_request(_: *mut Tox, public_key: *const u8, data: *const u8, length: u16,
                            internal: *mut c_void) {
    let internal = get_int!(internal);
    let msg = parse_string!(data, length);
    let id = ClientId { raw: unsafe { ptr::read(public_key as *const _) } };
    send_or_stop!(internal, FriendRequest(box id, msg));
}

extern fn on_friend_message(_: *mut Tox, friendnumber: i32, msg: *const u8, length: u16,
                            internal: *mut c_void) {
    let internal = get_int!(internal);
    let msg = parse_string!(msg, length);
    send_or_stop!(internal, FriendMessage(friendnumber, msg));
}

extern fn on_friend_action(_: *mut Tox, friendnumber: i32, act: *const u8, length: u16,
                           internal: *mut c_void) {
    let internal = get_int!(internal);
    let act = parse_string!(act, length);
    send_or_stop!(internal, FriendAction(friendnumber, act));
}

extern fn on_name_change(_: *mut Tox, friendnumber: i32, new: *const u8, length: u16,
                         internal: *mut c_void) {
    let internal = get_int!(internal);
    let new = parse_string!(new, length);
    send_or_stop!(internal, NameChange(friendnumber, new));
}

extern fn on_status_message(_: *mut Tox, friendnumber: i32, new: *const u8, length: u16,
                            internal: *mut c_void) {
    let internal = get_int!(internal);
    let new = parse_string!(new, length);
    send_or_stop!(internal, StatusMessage(friendnumber, new));
}

extern fn on_user_status(_: *mut Tox, friendnumber: i32, status: u8,
                         internal: *mut c_void) {
    let internal = get_int!(internal);
    let status = match status as u32 {
        TOX_USERSTATUS_NONE => UserStatusNone,
        TOX_USERSTATUS_AWAY => UserStatusAway,
        TOX_USERSTATUS_BUSY => UserStatusBusy,
        _ => return,
    };
    send_or_stop!(internal, UserStatusVar(friendnumber, status));
}

extern fn on_typing_change(_: *mut Tox, friendnumber: i32, is_typing: u8,
                           internal: *mut c_void) {
    let internal = get_int!(internal);
    send_or_stop!(internal, TypingChange(friendnumber, is_typing != 0));
}

extern fn on_read_receipt(_: *mut Tox, friendnumber: i32, receipt: u32,
                          internal: *mut c_void) {
    let internal = get_int!(internal);
    send_or_stop!(internal, ReadReceipt(friendnumber, receipt));
}

extern fn on_connection_status(_: *mut Tox, friendnumber: i32, status: u8,
                               internal: *mut c_void) {
    let internal = get_int!(internal);
    let status = match status {
        1 => Online,
        _ => Offline,
    };
    send_or_stop!(internal, ConnectionStatusVar(friendnumber, status));
}

extern fn on_group_invite(_: *mut Tox, friendnumber: i32, data: *const u8, length: u16,
                          internal: *mut c_void) {
    let internal = get_int!(internal);
    let data = unsafe {
        CVec::new(data as *mut _, length as uint).as_mut_slice().to_vec()
    };
    send_or_stop!(internal, GroupInvite(friendnumber, data));
}

extern fn on_group_message(_: *mut Tox, groupnumber: i32, frindgroupnumber: i32,
                           message: *const u8, len: u16, internal: *mut c_void) {
    let internal = get_int!(internal);
    let msg = parse_string!(message, len);
    send_or_stop!(internal, GroupMessage(groupnumber, frindgroupnumber, msg));
}

extern fn on_group_action(_: *mut Tox, groupnumber: i32, frindgroupnumber: i32,
                           action: *const u8, len: u16, internal: *mut c_void) {
    let internal = get_int!(internal);
    let action = parse_string!(action, len);
    send_or_stop!(internal, GroupMessage(groupnumber, frindgroupnumber, action));
}

extern fn on_group_namelist_change(_: *mut Tox, groupnumber: i32, peernumber: i32,
                                   change: u8, internal: *mut c_void) {
    let internal = get_int!(internal);
    let change = match change as u32 {
        TOX_CHAT_CHANGE_PEER_ADD => ChatChangePeerAdd,
        TOX_CHAT_CHANGE_PEER_DEL => ChatChangePeerDel,
        TOX_CHAT_CHANGE_PEER_NAME => ChatChangePeerName,
        _ => return,
    };
    send_or_stop!(internal, GroupNamelistChange(groupnumber, peernumber, change));
}

extern fn on_file_send_request(_: *mut Tox, friendnumber: i32, filenumber: u8,
                               filesize: u64, filename: *const u8, len: u16,
                               internal: *mut c_void) {
    let internal = get_int!(internal);
    let slice = to_slice(filename as *const u8, len as uint);
    let path = match Path::new_opt(slice) {
        Some(p) => match p.filename() {
            Some(f) => f.to_vec(),
            None => b"\xbf\xef".to_vec(),
        },
        None => b"\xbf\xef".to_vec(),
    };
    send_or_stop!(internal, FileSendRequest(friendnumber, filenumber, filesize, path));
}

extern fn on_file_control(_: *mut Tox, friendnumber: i32, receive_send: u8,
                          filenumber: u8, control_type: u8, data: *const u8, len: u16,
                          internal: *mut c_void) {
    let internal = get_int!(internal);
    let ty = match control_type as u32 {
        TOX_FILECONTROL_ACCEPT        => ControlAccept,
        TOX_FILECONTROL_PAUSE         => ControlPause,
        TOX_FILECONTROL_KILL          => ControlKill,
        TOX_FILECONTROL_FINISHED      => ControlFinished,
        TOX_FILECONTROL_RESUME_BROKEN => ControlResumeBroken,
        _ => return,
    };
    let tt = match receive_send {
        1 => Sending,
        0 => Receiving,
        _ => return,
    };
    let data = to_slice(data as *const u8, len as uint).to_vec();
    send_or_stop!(internal, FileControl(friendnumber, tt, filenumber, ty, data));
}

extern fn on_file_data(_: *mut Tox, friendnumber: i32, filenumber: u8, data: *const u8,
                       len: u16, internal: *mut c_void) {
    let internal = get_int!(internal);
    let data = to_slice(data as *const u8, len as uint).to_vec();
    send_or_stop!(internal, FileData(friendnumber, filenumber, data));
}

extern fn on_avatar_info(_: *mut Tox, friendnumber: i32, format: u8, hash: *mut u8,
                         internal: *mut c_void) {
    let internal = get_int!(internal);
    let format = match format as c_uint {
        TOX_AVATAR_FORMAT_NONE => AvatarNone,
        TOX_AVATAR_FORMAT_PNG  => AvatarPNG,
        _ => return,
    };
    let hash = unsafe { ptr::read(hash as *const u8 as *const _) };
    send_or_stop!(internal, AvatarInfo(friendnumber, format, hash));
}

extern fn on_avatar_data(_: *mut Tox, friendnumber: i32, format: u8, hash: *mut u8,
                         data: *mut u8, datalen: u32, internal: *mut c_void) {
    let internal = get_int!(internal);
    let format = match format as c_uint {
        TOX_AVATAR_FORMAT_NONE => AvatarNone,
        TOX_AVATAR_FORMAT_PNG  => AvatarPNG,
        _ => return,
    };
    let hash = unsafe { ptr::read(hash as *const u8 as *const _) };
    let data = unsafe { CVec::new(data, datalen as uint).as_mut_slice().to_vec() };
    send_or_stop!(internal, AvatarData(friendnumber, format, hash, data));
}

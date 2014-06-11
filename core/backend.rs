use std;
use core::ll::*;
use core::{Address, ClientId, ConnectionStatus, Online, Offline, MAX_NAME_LENGTH,
           UserStatus, UserStatusAway, UserStatusNone, UserStatusBusy};

pub struct Backend {
    raw: *mut Tox,
}

impl Backend {
    fn get_address(&mut self) -> Address {
        let mut adr: Address = unsafe { std::mem::uninitialized() };
        unsafe { tox_get_address(self.raw, &mut adr as *mut _ as *mut _); }
        adr
    }

    fn add_friend(&mut self, mut address: Box<Address>, mut msg: String) -> i32 {
        unsafe { tox_add_friend(self.raw, &mut *address as *mut _ as *mut _,
                                msg.as_mut_bytes().as_mut_ptr(), msg.len() as u16) }
    }

    fn add_friend_norequest(&mut self, client_id: Box<ClientId>) -> i32 {
        unsafe { tox_add_friend_norequest(self.raw, &client_id.raw as *_) }
    }

    fn get_friend_number(&mut self, client_id: Box<ClientId>) -> Result<i32, ()> {
        let res = unsafe {
            tox_get_friend_number(self.raw, &client_id.raw as *mut _)
        };
        match res {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    fn get_client_id(&mut self, friendnumber: i32) -> Result<ClientId, ()> {
        let mut client: ClientId = unsafe { std::mem::uninitialized() };
        let res = unsafe {
            tox_get_client_id(self.raw, friendnumber, &mut client.raw as *mut _)
        };
        match res {
            -1 => Err(()),
            _ => Ok(client),
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
        match unsafe { tox_get_friend_connection_status(self.raw, friendnumber) } {
            1 => Ok(Online),
            0 => Ok(Offline),
            _ => Err(()),
        }
    }

    fn friend_exists(&mut self, friendnumber: i32) -> bool {
        match unsafe { tox_friend_exists(self.raw, friendnumber) } {
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

    fn send_message_withid(&mut self, friendnumber: i32, theid: u32,
                               mut msg: String) -> Result<u32, ()> {
        let res = unsafe {
            tox_send_message_withid(self.raw, friendnumber, theid,
                                    msg.as_mut_vec().as_mut_ptr(), msg.len() as u32)
        };
        match res {
            0 => Err(()),
            n => Ok(n),
        }
    }

    fn send_action(&mut self, friendnumber: i32, mut action: String) -> Result<u32, ()> {
        let res = unsafe {
            tox_send_action(self.raw, friendnumber,
                            action.as_mut_vec().as_mut_ptr(), action.len() as u32)
        };
        match res {
            0 => Err(()),
            n => Ok(n),
        }
    }

    fn send_action_withid(&mut self, friendnumber: i32, theid: u32,
                          mut action: String) -> Result<u32, ()> {
        let res = unsafe {
            tox_send_action_withid(self.raw, friendnumber, theid,
                                   action.as_mut_vec().as_mut_ptr(), action.len() as u32)
        };
        match res {
            0 => Err(()),
            n => Ok(n),
        }
    }

    fn set_name(&mut self, mut name: String) -> Result<(),()> {
        let res = unsafe {
            tox_set_name(self.raw, name.as_mut_vec().as_mut_ptr(), name.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    fn get_self_name(&mut self) -> Result<String, ()> {
        let mut name = Vec::with_capacity(MAX_NAME_LENGTH);
        let res = unsafe {
            let len = tox_get_self_name(self.raw, name.as_mut_ptr());
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
            let len = tox_get_name(self.raw, friendnumber, name.as_mut_ptr());
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

    pub fn set_status_message(&mut self, mut status: String) -> Result<(),()> {
        let res = unsafe {
            tox_set_status_message(self.raw, status.as_mut_vec().as_mut_ptr(),
                                   status.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn set_user_status(&mut self, userstatus: UserStatus) -> Result<(), ()> {
        match unsafe { tox_set_user_status(self.raw, userstatus as u8) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn get_status_message(&mut self, friendnumber: i32) -> Result<String, ()> {
        let size = unsafe { tox_get_status_message_size(self.raw, friendnumber) };
        let size = match size {
            -1 => return Err(()),
            _ => size,
        };
        let mut status = Vec::with_capacity(size as uint);
        let size = unsafe {
            let len = tox_get_status_message(self.raw, friendnumber, status.as_mut_ptr(),
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

    pub fn get_self_status_message(&mut self) -> Result<String, ()> {
        let size = unsafe { tox_get_self_status_message_size(self.raw) };
        let size = match size {
            -1 => return Err(()),
            _ => size as u32,
        };
        let mut status = Vec::with_capacity(size as uint);
        let size = unsafe {
            let len = tox_get_self_status_message(self.raw, status.as_mut_ptr(), size);
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

    pub fn get_user_status(&mut self, friendnumber: i32) -> Result<UserStatus, ()> {
        match unsafe { tox_get_user_status(self.raw, friendnumber) as u32 } {
            TOX_USERSTATUS_AWAY => Ok(UserStatusAway),
            TOX_USERSTATUS_NONE => Ok(UserStatusNone),
            TOX_USERSTATUS_BUSY => Ok(UserStatusBusy),
            _ => Err(())
        }
    }

    pub fn get_self_user_status(&mut self) -> Result<UserStatus, ()> {
        match unsafe { tox_get_self_user_status(self.raw) as u32 } {
            TOX_USERSTATUS_AWAY => Ok(UserStatusAway),
            TOX_USERSTATUS_NONE => Ok(UserStatusNone),
            TOX_USERSTATUS_BUSY => Ok(UserStatusBusy),
            _ => Err(())
        }
    }

    pub fn get_last_online(&mut self, friendnumber: i32) -> Result<u64, ()> {
        match unsafe { tox_get_last_online(self.raw, friendnumber) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    pub fn set_user_is_typing(&mut self, friendnumber: i32,
                              is_typing: bool) -> Result<(), ()> {
        let raw = unsafe {
            tox_set_user_is_typing(self.raw, friendnumber, is_typing as u8)
        };
        match raw {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn get_is_typing(&mut self, friendnumber: i32) -> bool {
        match unsafe { tox_get_is_typing(self.raw, friendnumber) } {
            0 => false,
            _ => true,
        }
    }

    pub fn set_sends_receipts(&mut self, friendnumber: i32, yesno: bool) {
        unsafe { tox_set_sends_receipts(self.raw, friendnumber, yesno as i32); }
    }

    pub fn count_friendlist(&mut self) -> u32 {
        unsafe { tox_count_friendlist(self.raw) }
    }

    pub fn get_num_online_friends(&mut self) -> u32 {
        unsafe { tox_get_num_online_friends(self.raw) }
    }

    pub fn get_friendlist(&mut self) -> Vec<i32> {
        let size = self.count_friendlist();
        let mut vec = Vec::with_capacity(size as uint);
        unsafe {
            let len = tox_get_friendlist(self.raw, vec.as_mut_ptr(), size);
            vec.set_len(len as uint);
        }
        vec
    }

    pub fn get_nospam(&mut self) -> [u8, ..4] {
        unsafe { std::mem::transmute(std::mem::to_be32(tox_get_nospam(self.raw))) }
    }

    pub fn set_nospam(&mut self, nospam: [u8, ..4]) {
        unsafe { tox_set_nospam(self.raw,
                                std::mem::from_be32(std::mem::transmute(nospam))); }
    }

    pub fn add_groupchat(&mut self) -> Result<i32, ()> {
        match unsafe { tox_add_groupchat(self.raw) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    pub fn del_groupchat(&mut self, groupnumber: i32) -> Result<(),()> {
        match unsafe { tox_del_groupchat(self.raw, groupnumber) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn group_peername(&mut self, groupnumber: i32,
                          peernumber: i32) -> Result<String, ()> {
        let mut vec = Vec::with_capacity(MAX_NAME_LENGTH);
        let len = unsafe {
            let len = tox_group_peername(self.raw, groupnumber, peernumber,
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

    pub fn invite_friend(&mut self, friendnumber: i32,
                         groupnumber: i32) -> Result<(), ()> {
        match unsafe { tox_invite_friend(self.raw, friendnumber, groupnumber) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn join_groupchat(&mut self, friendnumber: i32,
                          mut fgpk: Box<ClientId>) -> Result<i32, ()> {
        let res = unsafe {
            tox_join_groupchat(self.raw, friendnumber, &mut fgpk.raw as *mut _)
        };
        match res {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    pub fn group_message_send(&mut self, groupnumber: i32,
                              mut msg: String) -> Result<(), ()> {
        let res = unsafe {
            tox_group_message_send(self.raw, groupnumber, msg.as_mut_vec().as_mut_ptr(),
                                   msg.len() as u32)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn group_action_send(&mut self, groupnumber: i32,
                             mut act: String) -> Result<(), ()> {
        let res = unsafe {
            tox_group_action_send(self.raw, groupnumber, act.as_mut_vec().as_mut_ptr(),
                                  act.len() as u32)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn group_number_peers(&mut self, groupnumber: i32) -> Result<i32, ()> {
        match unsafe { tox_group_number_peers(self.raw, groupnumber) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    pub fn group_get_names(&mut self,
                           groupnumber: i32) -> Result<Vec<Option<String>>, ()> {
        let num = match self.group_number_peers(groupnumber) {
            Ok(n) => n as uint,
            _ => return Err(()),
        };
        let mut names = Vec::with_capacity(num);
        let mut lengths = Vec::with_capacity(num);
        let len = unsafe {
            let len = tox_group_get_names(self.raw, groupnumber, names.as_mut_ptr(),
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

    pub fn get_chatlist(&mut self) -> Vec<i32> {
        let num = unsafe { tox_count_chatlist(self.raw) };
        let mut vec = Vec::with_capacity(num as uint);
        unsafe {
            let num = tox_get_chatlist(self.raw, vec.as_mut_ptr(), num);
            vec.set_len(num as uint);
        }
        vec
    }

    pub fn new_file_sender(&mut self, friendnumber: i32, filesize: u64,
                           filename: Path) -> Result<i32, ()> {
        let mut filename = filename.into_vec();
        let res = unsafe {
            tox_new_file_sender(self.raw, friendnumber, filesize,
                                filename.as_mut_ptr(), filename.len() as u16)
        };
        match res {
            -1 => Err(()),
            n => Ok(n)
        }
    }

    pub fn file_send_control(&mut self, friendnumber: i32, send_receive: u8,
                             filenumber: u8, message_id: u8,
                             mut data: Vec<u8>) -> Result<(), ()> {
        let res = unsafe {
            tox_file_send_control(self.raw, friendnumber, send_receive, filenumber,
                                  message_id, data.as_mut_ptr(), data.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn file_send_data(&mut self, friendnumber: i32, filenumber: u8,
                          mut data: Vec<u8>) -> Result<(), ()> {
        let res = unsafe {
            tox_file_send_data(self.raw, friendnumber, filenumber, data.as_mut_ptr(),
                               data.len() as u16)
        };
        match res {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn file_data_size(&mut self, friendnumber: i32) -> Result<i32, ()> {
        match unsafe { tox_file_data_size(self.raw, friendnumber) } {
            -1 => Err(()),
            n => Ok(n),
        }
    }

    pub fn file_data_remaining(&mut self, friendnumber: i32, filenumber: u8,
                               send_receive: u8) -> Result<u64, ()> {
        let res = unsafe {
            tox_file_data_remaining(self.raw, friendnumber, filenumber, send_receive)
        };
        match res {
            0 => Err(()),
            n => Ok(n),
        }
    }

    pub fn bootstrap_from_address(&mut self, mut address: String, ipv6enabled: bool,
                                  port: u16,
                                  mut public_key: Box<ClientId>) -> Result<(), ()> {
        let res = unsafe {
            address.push_byte(0);
            tox_bootstrap_from_address(self.raw, address.as_bytes().as_ptr() as *_,
                                       ipv6enabled as u8, std::mem::to_be16(port),
                                       &mut public_key.raw as *mut _)
        };
        match res {
            1 => Ok(()),
            _ => Err(()),
        }
    }

    pub fn is_connected(&mut self) -> bool {
        match unsafe { tox_isconnected(self.raw) } {
            0 => false,
            _ => true,
        }
    }

    // pub fn new(ipv6enabled: bool) -> Tox { }

    pub fn save(&mut self) -> Vec<u8> {
        let size = unsafe { tox_size(self.raw) as uint };
        let mut vec = Vec::with_capacity(size);
        unsafe {
            tox_save(self.raw, vec.as_mut_ptr());
            vec.set_len(size);
        }
        vec
    }

    pub fn load(&mut self, mut data: Vec<u8>) -> Result<(), ()> {
        match unsafe { tox_load(self.raw, data.as_mut_ptr(), data.len() as u32) } {
            0 => Ok(()),
            _ => Err(()),
        }
    }
}

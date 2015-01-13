#![allow(dead_code)]

//use std::option::{Option};
use libc::{c_int, c_uint, c_char, c_void};

pub const TOX_FAERR_TOOLONG:      c_int = -1;
pub const TOX_FAERR_NOMESSAGE:    c_int = -2;
pub const TOX_FAERR_OWNKEY:       c_int = -3;
pub const TOX_FAERR_ALREADYSENT:  c_int = -4;
pub const TOX_FAERR_UNKNOWN:      c_int = -5;
pub const TOX_FAERR_BADCHECKSUM:  c_int = -6;
pub const TOX_FAERR_SETNEWNOSPAM: c_int = -7;
pub const TOX_FAERR_NOMEM:        c_int = -8;

pub const TOX_USERSTATUS_NONE:    c_uint = 0;
pub const TOX_USERSTATUS_AWAY:    c_uint = 1;
pub const TOX_USERSTATUS_BUSY:    c_uint = 2;
pub const TOX_USERSTATUS_INVALID: c_uint = 3;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct Tox;

unsafe impl Send for *mut Tox { }

pub const TOX_CHAT_CHANGE_PEER_ADD:  c_uint = 0;
pub const TOX_CHAT_CHANGE_PEER_DEL:  c_uint = 1;
pub const TOX_CHAT_CHANGE_PEER_NAME: c_uint = 2;

pub const TOX_FILECONTROL_ACCEPT:        c_uint = 0;
pub const TOX_FILECONTROL_PAUSE:         c_uint = 1;
pub const TOX_FILECONTROL_KILL:          c_uint = 2;
pub const TOX_FILECONTROL_FINISHED:      c_uint = 3;
pub const TOX_FILECONTROL_RESUME_BROKEN: c_uint = 4;

pub const TOX_AVATAR_FORMAT_NONE: c_uint = 0;
pub const TOX_AVATAR_FORMAT_PNG:  c_uint = 1;

pub const TOX_GROUPCHAT_TYPE_TEXT: c_uint = 0;
pub const TOX_GROUPCHAT_TYPE_AV:   c_uint = 1;

#[repr(C)]
#[derive(Copy)]
pub struct Tox_Options {
    pub ipv6enabled:   u8,
    pub udp_disabled:  u8,
    pub proxy_type:    u8,
    pub proxy_address: [u8; 256],
    pub proxy_port:    u16,
}

#[link(name = "toxcore")]
extern {
    pub fn tox_get_address(tox: *const Tox, address: *mut u8);
    pub fn tox_add_friend(tox: *mut Tox, address: *const u8, data: *const u8,
                          length: u16) -> i32;
    pub fn tox_add_friend_norequest(tox: *mut Tox, client_id: *const u8) -> i32;
    pub fn tox_get_friend_number(tox: *const Tox, client_id: *const u8) -> i32;
    pub fn tox_get_client_id(tox: *const Tox, friendnumber: i32,
                             client_id: *mut u8) -> c_int;
    pub fn tox_del_friend(tox: *mut Tox, friendnumber: i32) -> c_int;
    pub fn tox_get_friend_connection_status(tox: *const Tox,
                                            friendnumber: i32) -> c_int;
    pub fn tox_friend_exists(tox: *const Tox, friendnumber: i32) -> c_int;
    pub fn tox_send_message(tox: *mut Tox, friendnumber: i32, message: *const u8,
                            length: u32) -> u32;
    pub fn tox_send_action(tox: *mut Tox, friendnumber: i32, action: *const u8,
                           length: u32) -> u32;
    pub fn tox_set_name(tox: *mut Tox, name: *const u8, length: u16) -> c_int;
    pub fn tox_get_self_name(tox: *const Tox, name: *mut u8) -> u16;
    pub fn tox_get_name(tox: *const Tox, friendnumber: i32, name: *mut u8) -> c_int;
    pub fn tox_get_name_size(tox: *const Tox, friendnumber: i32) -> c_int;
    pub fn tox_get_self_name_size(tox: *const Tox) -> c_int;
    pub fn tox_set_status_message(tox: *mut Tox, status: *const u8, length: u16) -> c_int;
    pub fn tox_set_user_status(tox: *mut Tox, userstatus: u8) -> c_int;
    pub fn tox_get_status_message_size(tox: *const Tox, friendnumber: i32) -> c_int;
    pub fn tox_get_self_status_message_size(tox: *const Tox) -> c_int;
    pub fn tox_get_status_message(tox: *const Tox, friendnumber: i32, buf: *mut u8,
                                  maxlen: u32) -> c_int;
    pub fn tox_get_self_status_message(tox: *const Tox, buf: *mut u8,
                                       maxlen: u32) -> c_int;
    pub fn tox_get_user_status(tox: *const Tox, friendnumber: i32) -> u8;
    pub fn tox_get_self_user_status(tox: *const Tox) -> u8;
    pub fn tox_get_last_online(tox: *const Tox, friendnumber: i32) -> u64;
    pub fn tox_set_user_is_typing(tox: *mut Tox, friendnumber: i32,
                                  is_typing: u8) -> c_int;
    pub fn tox_get_is_typing(tox: *const Tox, friendnumber: i32) -> u8;
    pub fn tox_count_friendlist(tox: *const Tox) -> u32;
    pub fn tox_get_num_online_friends(tox: *const Tox) -> u32;
    pub fn tox_get_friendlist(tox: *const Tox, out_list: *mut i32, list_size: u32) -> u32;
    pub fn tox_callback_friend_request(tox: *mut Tox,
                                       function:
                                           /*Option<*/extern fn
                                                    (arg1: *mut Tox,
                                                     arg2: *const u8,
                                                     arg3: *const u8,
                                                     arg4: u16,
                                                     arg5: *mut c_void)/*>*/,
                                       userdata: *mut c_void);
    pub fn tox_callback_friend_message(tox: *mut Tox,
                                       function:
                                           /*Option<*/extern fn
                                             (arg1: *mut Tox,
                                              arg2: i32,
                                              arg3: *const u8,
                                              arg4: u16,
                                              arg5: *mut c_void)/*>*/,
                                       userdata: *mut c_void);
    pub fn tox_callback_friend_action(tox: *mut Tox,
                                      function:
                                          /*Option<*/extern fn
                                                   (arg1: *mut Tox,
                                                    arg2: i32,
                                                    arg3: *const u8,
                                                    arg4: u16,
                                                    arg5: *mut c_void)/*>*/,
                                      userdata: *mut c_void);
    pub fn tox_callback_name_change(tox: *mut Tox,
                                    function:
                                        /*Option<*/extern fn
                                                 (arg1: *mut Tox,
                                                  arg2: i32,
                                                  arg3: *const u8,
                                                  arg4: u16,
                                                  arg5: *mut c_void)/*>*/,
                                    userdata: *mut c_void);
    pub fn tox_callback_status_message(tox: *mut Tox,
                                       function:
                                           /*Option<*/extern fn
                                                    (arg1: *mut Tox,
                                                     arg2: i32,
                                                     arg3: *const u8,
                                                     arg4: u16,
                                                     arg5: *mut c_void)/*>*/,
                                       userdata: *mut c_void);
    pub fn tox_callback_user_status(tox: *mut Tox,
                                    function:
                                        /*Option<*/extern fn
                                                 (arg1: *mut Tox,
                                                  arg2: i32,
                                                  arg3: u8,
                                                  arg4: *mut c_void)/*>*/,
                                    userdata: *mut c_void);
    pub fn tox_callback_typing_change(tox: *mut Tox,
                                      function:
                                          /*Option<*/extern fn
                                                   (arg1: *mut Tox,
                                                    arg2: i32,
                                                    arg3: u8,
                                                    arg4: *mut c_void)/*>*/,
                                      userdata: *mut c_void);
    pub fn tox_callback_read_receipt(tox: *mut Tox,
                                     function:
                                         /*Option<*/extern fn
                                                  (arg1: *mut Tox,
                                                   arg2: i32,
                                                   arg3: u32,
                                                   arg4: *mut c_void)/*>*/,
                                     userdata: *mut c_void);
    pub fn tox_callback_connection_status(tox: *mut Tox,
                                          function:
                                              /*Option<*/extern fn
                                                       (arg1: *mut Tox,
                                                        arg2: i32,
                                                        arg3: u8,
                                                        arg4: *mut c_void)/*>*/,
                                          userdata: *mut c_void);
    pub fn tox_get_nospam(tox: *const Tox) -> u32;
    pub fn tox_set_nospam(tox: *mut Tox, nospam: u32);
    pub fn tox_get_keys(tox: *mut Tox, public_key: *mut u8, secret_key: *mut u8);
    pub fn tox_lossy_packet_registerhandler(tox: *mut Tox,
                                            friendnumber: i32,
                                            byte: u8,
                                            packet_handler_callback:
                                                /*Option<*/extern fn
                                                         (arg1: *mut c_void,
                                                          arg2: *const u8,
                                                          arg3: u32) -> c_int/*>*/,
                                            object: *mut c_void) -> c_int;
    pub fn tox_send_lossy_packet(tox: *const Tox, friendnumber: i32, data: *const u8,
                                 length: u32) -> c_int;
    pub fn tox_lossless_packet_registerhandler(tox: *mut Tox,
                                               friendnumber: i32,
                                               byte: u8,
                                               packet_handler_callback:
                                                   /*Option<*/extern fn
                                                            (arg1: *mut c_void,
                                                             arg2: *const u8,
                                                             arg3: u32) -> c_int/*>*/,
                                               object: *mut c_void) -> c_int;
    pub fn tox_send_lossless_packet(tox: *const Tox, friendnumber: i32, data: *const u8,
                                    length: u32) -> c_int;
    pub fn tox_callback_group_invite(tox: *mut Tox,
                                     function:
                                         /*Option<*/extern fn
                                                  (arg1: *mut Tox,
                                                   arg2: i32,
                                                   arg3: u8,
                                                   arg4: *const u8,
                                                   arg5: u16,
                                                   arg6: *mut c_void)/*>*/,
                                     userdata: *mut c_void);
    pub fn tox_callback_group_message(tox: *mut Tox,
                                      function:
                                          /*Option<*/extern fn
                                                   (arg1: *mut Tox,
                                                    arg2: c_int,
                                                    arg3: c_int,
                                                    arg4: *const u8,
                                                    arg5: u16,
                                                    arg6: *mut c_void)/*>*/,
                                      userdata: *mut c_void);
    pub fn tox_callback_group_action(tox: *mut Tox,
                                     function:
                                         /*Option<*/extern fn
                                                  (arg1: *mut Tox,
                                                   arg2: c_int,
                                                   arg3: c_int,
                                                   arg4: *const u8,
                                                   arg5: u16,
                                                   arg6: *mut c_void)/*>*/,
                                     userdata: *mut c_void);
    pub fn tox_callback_group_title(tox: *mut Tox,
                                    function:
                                        /*Option<*/extern fn
                                                 (arg1: *mut Tox,
                                                  arg2: c_int,
                                                  arg3: c_int,
                                                  arg4: *const u8,
                                                  arg5: u8,
                                                  arg6: *mut c_void)/*>*/,
                                    userdata: *mut c_void);
    pub fn tox_callback_group_namelist_change(tox: *mut Tox,
                                              function:
                                                  /*Option<*/extern fn
                                                           (arg1: *mut Tox,
                                                            arg2: c_int,
                                                            arg3: c_int,
                                                            arg4: u8,
                                                            arg5: *mut c_void)/*>*/,
                                              userdata: *mut c_void);
    pub fn tox_add_groupchat(tox: *mut Tox) -> c_int;
    pub fn tox_del_groupchat(tox: *mut Tox, groupnumber: c_int) -> c_int;
    pub fn tox_group_peername(tox: *const Tox, groupnumber: c_int, peernumber: c_int,
                              name: *mut u8) -> c_int;
    pub fn tox_group_peer_pubkey(tox: *const Tox, groupnumber: c_int, peernumber: c_int,
                                 pk: *mut u8) -> c_int;
    pub fn tox_invite_friend(tox: *mut Tox, friendnumber: i32,
                             groupnumber: c_int) -> c_int;
    pub fn tox_join_groupchat(tox: *mut Tox, friendnumber: i32, data: *const u8,
                              length: u16) -> c_int;
    pub fn tox_group_message_send(tox: *mut Tox, groupnumber: c_int, message: *const u8,
                                  length: u16) -> c_int;
    pub fn tox_group_get_title(tox: *const Tox, groupnumber: c_int,
                               title: *mut u8, max_length: u32) -> c_int;
    pub fn tox_group_set_title(tox: *mut Tox, groupnumber: c_int, title: *const u8,
                               length: u8) -> c_int;
    pub fn tox_group_action_send(tox: *mut Tox, groupnumber: c_int, action: *const u8,
                                 length: u16) -> c_int;
    pub fn tox_group_peernumber_is_ours(tox: *const Tox, groupnumber: c_int,
                                        peernumber: c_int) -> c_uint;
    pub fn tox_group_number_peers(tox: *const Tox, groupnumber: c_int) -> c_int;
    pub fn tox_group_get_names(tox: *const Tox, groupnumber: c_int,
                               names: *mut [u8; 128us], lengths: *mut u16,
                               length: u16) -> c_int;
    pub fn tox_count_chatlist(tox: *const Tox) -> u32;
    pub fn tox_get_chatlist(tox: *const Tox, out_list: *mut i32, list_size: u32) -> u32;
    pub fn tox_group_get_type(tox: *const Tox, groupnumber: c_int) -> c_int;
    pub fn tox_callback_avatar_info(tox: *mut Tox,
                                    function:
                                        /*Option<*/extern fn
                                                 (arg1: *mut Tox,
                                                  arg2: i32,
                                                  arg3: u8,
                                                  arg4: *mut u8,
                                                  arg5: *mut c_void)/*>*/,
                                    userdata: *mut c_void);
    pub fn tox_callback_avatar_data(tox: *mut Tox,
                                    function:
                                        /*Option<*/extern fn
                                                 (arg1: *mut Tox,
                                                  arg2: i32,
                                                  arg3: u8,
                                                  arg4: *mut u8,
                                                  arg5: *mut u8,
                                                  arg6: u32,
                                                  arg7: *mut c_void)/*>*/,
                                    userdata: *mut c_void);
    pub fn tox_set_avatar(tox: *mut Tox, format: u8, data: *const u8,
                          length: u32) -> c_int;
    pub fn tox_unset_avatar(tox: *mut Tox) -> c_int;
    pub fn tox_get_self_avatar(tox: *const Tox, format: *mut u8, buf: *mut u8,
                               length: *mut u32, maxlen: u32, hash: *mut u8) -> c_int;
    pub fn tox_hash(hash: *mut u8, data: *const u8, datalen: u32) -> c_int;
    pub fn tox_request_avatar_info(tox: *const Tox, friendnumber: i32) -> c_int;
    pub fn tox_send_avatar_info(tox: *mut Tox, friendnumber: i32) -> c_int;
    pub fn tox_request_avatar_data(tox: *const Tox, friendnumber: i32) -> c_int;
    pub fn tox_callback_file_send_request(tox: *mut Tox,
                                          function:
                                              /*Option<*/extern fn
                                                       (arg1: *mut Tox,
                                                        arg2: i32,
                                                        arg3: u8,
                                                        arg4: u64,
                                                        arg5: *const u8,
                                                        arg6: u16,
                                                        arg7: *mut c_void)/*>*/,
                                          userdata: *mut c_void);
    pub fn tox_callback_file_control(tox: *mut Tox,
                                     function:
                                         /*Option<*/extern fn
                                                  (arg1: *mut Tox,
                                                   arg2: i32,
                                                   arg3: u8,
                                                   arg4: u8,
                                                   arg5: u8,
                                                   arg6: *const u8,
                                                   arg7: u16,
                                                   arg8: *mut c_void)/*>*/,
                                     userdata: *mut c_void);
    pub fn tox_callback_file_data(tox: *mut Tox,
                                  function:
                                      /*Option<*/extern fn
                                               (arg1: *mut Tox,
                                                arg2: i32,
                                                arg3: u8,
                                                arg4: *const u8,
                                                arg5: u16,
                                                arg6: *mut c_void)/*>*/,
                                  userdata: *mut c_void);
    pub fn tox_new_file_sender(tox: *mut Tox, friendnumber: i32, filesize: u64,
                               filename: *const u8, filename_length: u16) -> c_int;
    pub fn tox_file_send_control(tox: *mut Tox, friendnumber: i32, send_receive: u8,
                                 filenumber: u8, message_id: u8, data: *const u8,
                                 length: u16) -> c_int;
    pub fn tox_file_send_data(tox: *mut Tox, friendnumber: i32, filenumber: u8,
                              data: *const u8, length: u16) -> c_int;
    pub fn tox_file_data_size(tox: *const Tox, friendnumber: i32) -> c_int;
    pub fn tox_file_data_remaining(tox: *const Tox, friendnumber: i32, filenumber: u8,
                                   send_receive: u8) -> u64;
    pub fn tox_bootstrap_from_address(tox: *mut Tox, address: *const c_char, port: u16,
                                      public_key: *const u8) -> c_int;
    pub fn tox_add_tcp_relay(tox: *mut Tox, address: *const c_char, port: u16,
                             public_key: *const u8) -> c_int;
    pub fn tox_isconnected(tox: *const Tox) -> c_int;
    pub fn tox_new(options: *mut Tox_Options) -> *mut Tox;
    pub fn tox_kill(tox: *mut Tox);
    pub fn tox_do_interval(tox: *mut Tox) -> u32;
    pub fn tox_do(tox: *mut Tox);
    pub fn tox_size(tox: *const Tox) -> u32;
    pub fn tox_save(tox: *const Tox, data: *mut u8);
    pub fn tox_load(tox: *mut Tox, data: *const u8, length: u32) -> c_int;
}

use libc::{c_void, c_int, c_uint, c_char};

use core::ll::{Tox};

use av::{CallState, CallSettings, Capability};

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct ToxAv;

unsafe impl Send for *mut ToxAv { }

pub type ToxAVCallback = extern fn(agent: *mut c_void, call_idx: i32, arg: *mut c_void);
pub type ToxAvAudioCallback = extern fn(agent: *mut c_void, call_idx: i32,
                                        PCM: *const i16, size: u16, data: *mut c_void);

pub type ToxAudioCallback = extern fn(tox: *mut Tox, groupnumber: c_int,
                                      peernumber: c_int, pcm: *const i16, samples: c_uint,
                                      channels: u8, sample_rate: c_uint,
                                      userdata: *mut c_void);

#[repr(C)]
#[derive(Copy)]
pub enum ToxAvCallbackId {
    av_OnInvite,
    av_OnRinging,
    av_OnStart,
    av_OnCancel,
    av_OnReject,
    av_OnEnd,
    av_OnRequestTimeout,
    av_OnPeerTimeout,
    av_OnPeerCSChange,
    av_OnSelfCSChange,
}

#[link(name = "toxav")]
extern {
    pub static av_DefaultSettings: CallSettings;

    pub fn toxav_new(messenger: *mut Tox, max_calls: i32) -> *mut ToxAv;
    pub fn toxav_kill(av: *mut ToxAv);
    pub fn toxav_do_interval(av: *mut ToxAv) -> u32;
    pub fn toxav_do(av: *mut ToxAv);
    pub fn toxav_register_callstate_callback(av: *mut ToxAv, cb: Option<ToxAVCallback>,
                                             id: ToxAvCallbackId, userdata: *mut c_void);
    pub fn toxav_register_audio_callback(av: *mut ToxAv, cb: Option<ToxAvAudioCallback>,
                                         userdata: *mut c_void);
    pub fn toxav_call(av: *mut ToxAv, call_index: *mut i32, friend_id: c_int,
                      csettings: *const CallSettings, ringing_seconds: c_int) -> c_int;
    pub fn toxav_hangup(av: *mut ToxAv, call_index: i32) -> c_int;
    pub fn toxav_answer(av: *mut ToxAv, call_index: i32,
                        csettings: *const CallSettings) -> c_int;
    pub fn toxav_reject(av: *mut ToxAv, call_index: i32, reason: *const c_char) -> c_int;
    pub fn toxav_cancel(av: *mut ToxAv, call_index: i32, peer_id: c_int,
                        reason: *const c_char) -> c_int;
    pub fn toxav_change_settings(av: *mut ToxAv, call_index: i32,
                                 csettings: *const CallSettings) -> c_int;
    pub fn toxav_stop_call(av: *mut ToxAv, call_index: i32) -> c_int;
    pub fn toxav_prepare_transmission(av: *mut ToxAv, call_index: i32,
                                      support_video: c_int) -> c_int;
    pub fn toxav_kill_transmission(av: *mut ToxAv, call_index: i32) -> c_int;
    pub fn toxav_prepare_audio_frame(av: *mut ToxAv, call_index: i32, dest: *mut u8,
                                     dest_max: c_int, frame: *const i16,
                                     frame_size: c_int) -> c_int;
    pub fn toxav_send_audio(av: *mut ToxAv, call_index: i32, frame: *const u8,
                            size: c_uint) -> c_int;
    pub fn toxav_get_peer_csettings(av: *mut ToxAv, call_index: i32, peer: c_int,
                                    dest: *mut CallSettings) -> c_int;
    pub fn toxav_get_peer_id(av: *mut ToxAv, call_index: i32, peer: c_int) -> c_int;
    pub fn toxav_get_call_state(av: *mut ToxAv, call_index: i32) -> CallState;
    pub fn toxav_capability_supported(av: *mut ToxAv, call_index: i32,
                                      capability: Capability) -> c_int;
    pub fn toxav_get_tox(av: *mut ToxAv) -> *mut Tox;
    pub fn toxav_get_active_count(av: *mut ToxAv) -> c_int;
    pub fn toxav_add_av_groupchat(tox: *mut Tox, audio_callback: Option<ToxAudioCallback>,
                                  userdata: *mut c_void) -> c_int;
    pub fn toxav_join_av_groupchat(tox: *mut Tox, friendnumber: i32, data: *const u8,
                                   length: u16, audio_callback: Option<ToxAudioCallback>,
                                   userdata: *mut c_void) -> c_int;
    pub fn toxav_group_send_audio(tox: *mut Tox, groupnumber: c_int, pcm: *const i16,
                                  samples: c_uint, channels: u8,
                                  sample_rate: c_uint) -> c_int;
}

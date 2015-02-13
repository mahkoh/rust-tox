use core::ll::{Tox};
use av::ll::*;
use av::{Event, CallSettings, CallState, Capability, AudioBit, ControlProducer, AvEvents};

use comm::{self, spsc};

use libc::{c_void, c_int, c_uint, c_char};
use std::mem::{transmute, zeroed};
use std::{self, slice};
use std::old_io::{timer};

type OneSpaceProducer<T> = spsc::one_space::Producer<T>;

pub enum Control {
    Call(i32, Option<Box<CallSettings>>, i32, OneSpaceProducer<Result<i32, i32>>), 
    Hangup(i32, OneSpaceProducer<Result<(), i32>>),
    Answer(i32, Option<Box<CallSettings>>, OneSpaceProducer<Result<(), i32>>),
    Reject(i32, OneSpaceProducer<Result<(), i32>>),
    Cancel(i32, i32, OneSpaceProducer<Result<(), i32>>),
    ChangeSettings(i32, Option<Box<CallSettings>>, OneSpaceProducer<Result<(), i32>>),
    StopCall(i32, OneSpaceProducer<Result<(), i32>>),
    PrepareTransmission(i32, bool, OneSpaceProducer<Result<(), i32>>),
    KillTransmission(i32, OneSpaceProducer<Result<(), i32>>),
    PrepareAudioFrame(i32, Vec<u8>, Vec<i16>, OneSpaceProducer<Result<(Vec<u8>, Vec<i16>), (i32, Vec<u8>, Vec<i16>)>>),
    SendAudio(i32, Vec<u8>, OneSpaceProducer<Result<Vec<u8>, (i32, Vec<u8>)>>),
    GetPeerCallSettings(i32, i32, OneSpaceProducer<Result<Box<CallSettings>, i32>>),
    GetPeerId(i32, i32, OneSpaceProducer<Result<i32, i32>>),
    GetCallState(i32, OneSpaceProducer<CallState>),
    CapabilitySupported(i32, Capability, OneSpaceProducer<Result<bool, i32>>),
    GetActiveCount(OneSpaceProducer<Result<usize, i32>>),
    AddAvGroupchat(OneSpaceProducer<Result<i32, i32>>),
    JoinAvGroupchat(i32, Vec<u8>, OneSpaceProducer<Result<i32, i32>>),
    GroupSendAudio(i32, AudioBit, OneSpaceProducer<Result<AudioBit, (i32, AudioBit)>>),
}

pub struct Backend {
    raw: *mut ToxAv,
    raw_tox: *mut Tox,
    internal: Box<Internal>,
    control: spsc::one_space::Consumer<Control>,
    _send_end: spsc::one_space::Producer<()>,
}

impl Backend {
    pub fn call(&mut self, friend_id: i32, settings: Option<Box<CallSettings>>,
                timeout: i32) -> Result<i32, i32> {
        let settings = match settings {
            Some(ref s) => &**s,
            _ => &av_DefaultSettings,
        };
        let mut index = 0;
        let res = unsafe {
            toxav_call(self.raw, &mut index, friend_id as c_int, settings,
                       timeout as c_int)
        };
        match res {
            0 => Ok(index),
            _ => Err(res),
        }
    }

    pub fn hangup(&mut self, call_id: i32) -> Result<(), i32> {
        let res = unsafe { toxav_hangup(self.raw, call_id) };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn answer(&mut self, call_id: i32,
                  settings: Option<Box<CallSettings>>) -> Result<(), i32> {
        let settings = match settings {
            Some(ref s) => &**s,
            _ => &av_DefaultSettings,
        };
        let res = unsafe { toxav_answer(self.raw, call_id, settings) };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn reject(&mut self, call_id: i32) -> Result<(), i32> {
        let res = unsafe { toxav_reject(self.raw, call_id, 0 as *const c_char) };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn cancel(&mut self, call_id: i32, peer_id: i32) -> Result<(), i32> {
        let res = unsafe { toxav_cancel(self.raw, call_id, peer_id as c_int, 0 as *const c_char) };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn change_settings(&mut self, call_id: i32,
                           settings: Option<Box<CallSettings>>) -> Result<(), i32>{
        let settings = match settings {
            Some(ref s) => &**s,
            _ => &av_DefaultSettings,
        };
        let res = unsafe { toxav_change_settings(self.raw, call_id, settings) };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn stop_call(&mut self, call_id: i32) -> Result<(), i32> {
        let res = unsafe { toxav_stop_call(self.raw, call_id) };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn prepare_transmission(&mut self, call_id: i32,
                                support_video: bool) -> Result<(), i32>{
        let res = unsafe {
            toxav_prepare_transmission(self.raw, call_id, support_video as c_int)
        };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn kill_transmission(&mut self, call_id: i32) -> Result<(), i32> {
        let res = unsafe { toxav_kill_transmission(self.raw, call_id) };
        match res {
            0 => Ok(()),
            _ => Err(res),
        }
    }

    pub fn prepare_audio_frame(&mut self, _call_id: i32, _dest: Vec<u8>,
                               _src: Vec<i16>) -> Result<(Vec<u8>, Vec<i16>), (i32, Vec<u8>, Vec<i16>)> {
        // Seriously wtf
        // This piece of shit code has no comments
        // How am I supposed to wrap this shit?
        unimplemented!()
    }

    pub fn send_audio(&mut self, call_id: i32,
                      src: Vec<u8>) -> Result<Vec<u8>, (i32, Vec<u8>)> {
        let res = unsafe {
            toxav_send_audio(self.raw, call_id, src.as_ptr(), src.len() as c_uint)
        };
        match res {
            0 => Ok(src),
            _ => Err((res, src)),
        }
    }

    pub fn get_peer_call_settings(&mut self, call_id: i32,
                                  peer_id: i32) -> Result<Box<CallSettings>, i32> {
        let mut settings = unsafe { zeroed() };
        let res = unsafe {
            toxav_get_peer_csettings(self.raw, call_id, peer_id as c_int, &mut settings)
        };
        match res {
            0 => Ok(Box::new(settings)),
            _ => Err(res),
        }
    }

    pub fn get_peer_id(&mut self, call_id: i32, peer_id: i32) -> Result<i32, i32> {
        let res = unsafe { toxav_get_peer_id(self.raw, call_id, peer_id as c_int) };
        if res < 0 {
            Err(res)
        } else {
            Ok(res)
        }
    }

    pub fn get_call_state(&mut self, call_id: i32) -> CallState {
        let state = unsafe { toxav_get_call_state(self.raw, call_id) };
        std::num::FromPrimitive::from_i64(state as i64).unwrap()
    }

    pub fn capability_supported(&mut self, call_id: i32,
                                capability: Capability) -> Result<bool, i32> {
        let res = unsafe { toxav_capability_supported(self.raw, call_id, capability) };
        match res {
            1 => Ok(true),
            0 => Ok(false),
            _ => Err(res),
        }
    }

    pub fn get_active_count(&mut self) -> Result<usize, i32> {
        let res = unsafe { toxav_get_active_count(self.raw) };
        if res >= 0 {
            Ok(res as usize)
        } else {
            Err(res)
        }
    }

    pub fn add_av_groupchat(&mut self) -> Result<i32, i32> {
        let ip = &mut *self.internal as *mut _ as *mut c_void;
        let ret = unsafe {
            toxav_add_av_groupchat(self.raw_tox, Some(on_group_audio),
                                   ip)
        };
        match ret {
            -1 => Err(ret),
            _ => Ok(ret),
        }
    }

    pub fn join_av_groupchat(&mut self, friend_id: i32,
                             data: Vec<u8>) -> Result<i32, i32> {
        let ip = &mut *self.internal as *mut _ as *mut c_void;
        let ret = unsafe {
            toxav_join_av_groupchat(self.raw_tox, friend_id, data.as_ptr(),
                                    data.len() as u16, Some(on_group_audio), ip)
        };
        match ret {
            -1 => Err(ret),
            _ => Ok(ret),
        }
    }

    pub fn group_send_audio(&mut self, group_id: i32,
                            bit: AudioBit) -> Result<AudioBit, (i32, AudioBit)> {
        if !bit.validate() {
            return Err((-1, bit));
        }
        let ret = unsafe {
            toxav_group_send_audio(self.raw_tox, group_id as c_int, bit.pcm.as_ptr(),
                                   bit.samples as c_uint, bit.channels,
                                   bit.sample_rate as c_uint)
        };
        match ret {
            0 => Ok(bit),
            _ => Err((ret, bit)),
        }
    }

    pub fn new(tox: *mut Tox, max_calls: i32, send_end: spsc::one_space::Producer<()>)
                        -> Option<(ControlProducer, AvEvents)> {
        let av = unsafe { toxav_new(tox, max_calls) };
        if av.is_null() {
            return None;
        }
        let (event_send, event_recv) = spsc::bounded::new(64);
        let mut internal = Box::new(Internal { stop: false, events: event_send });

        unsafe {
            let ip = &mut *internal as *mut _ as *mut c_void;
            macro_rules! rcsc {
                ($func:ident, $id:ident) => {
                    toxav_register_callstate_callback(av, Some($func),
                                                      ToxAvCallbackId::$id, ip);
                }
            };
            rcsc!( on_invite          , av_OnInvite       );
            rcsc!( on_ringing         , av_OnRinging      );
            rcsc!( on_start           , av_OnStart        );
            rcsc!( on_cancel          , av_OnCancel       );
            rcsc!( on_reject          , av_OnReject       );
            rcsc!( on_end             , av_OnEnd          );
            rcsc!( on_request_timeout , av_OnEnd          );
            rcsc!( on_peer_timeout    , av_OnPeerTimeout  );
            rcsc!( on_peer_cs_change  , av_OnPeerCSChange );
            rcsc!( on_self_cs_change  , av_OnSelfCSChange );
            toxav_register_audio_callback(av, Some(on_audio), ip);
        }
        let (control_send, control_recv) = spsc::one_space::new();
        let backend = Backend {
            raw: av,
            raw_tox: tox,
            internal: internal,
            control: control_recv,
            _send_end: send_end,
        };
        std::thread::Thread::spawn(move || backend.run());
        Some((control_send, event_recv))
    }

    fn run(mut self) {
        'outer: loop {
            unsafe { toxav_do(self.raw); }
            if self.internal.stop {
                break 'outer;
            }

            'inner: loop {
                match self.control.recv_async() {
                    Ok(ctrl) => self.control(ctrl),
                    Err(comm::Error::Disconnected) => break 'outer,
                    _ => break 'inner,
                }
            }

            let interval = unsafe { toxav_do_interval(self.raw) as i64 };
            timer::sleep(::std::time::Duration::milliseconds(interval));
        }
    }

    fn control(&mut self, ctrl: Control) {
        match ctrl {
            Control::Call(friend_id, settings, timeout, ret) =>
                ret.send(self.call(friend_id, settings, timeout)).unwrap(),
            Control::Hangup(call_id, ret) =>
                ret.send(self.hangup(call_id)).unwrap(),
            Control::Answer(call_id, settings, ret) =>
                ret.send(self.answer(call_id, settings)).unwrap(),
            Control::Reject(call_id, ret) =>
                ret.send(self.reject(call_id)).unwrap(),
            Control::Cancel(call_id, peer_id, ret) =>
                ret.send(self.cancel(call_id, peer_id)).unwrap(),
            Control::ChangeSettings(call_id, settings, ret) =>
                ret.send(self.change_settings(call_id, settings)).unwrap(),
            Control::StopCall(call_id, ret) =>
                ret.send(self.stop_call(call_id)).unwrap(),
            Control::PrepareTransmission(call_id, support_video, ret) =>
                ret.send(self.prepare_transmission(call_id, support_video)).unwrap(),
            Control::KillTransmission(call_id, ret) =>
                ret.send(self.kill_transmission(call_id)).unwrap(),
            Control::PrepareAudioFrame(call_id, dest, frame, ret) =>
                ret.send(self.prepare_audio_frame(call_id, dest, frame)).unwrap(),
            Control::SendAudio(call_id, frame, ret) =>
                ret.send(self.send_audio(call_id, frame)).unwrap(),
            Control::GetPeerCallSettings(call_id, peer_id, ret) =>
                ret.send(self.get_peer_call_settings(call_id, peer_id)).unwrap(),
            Control::GetPeerId(call_id, peer_id, ret) =>
                ret.send(self.get_peer_id(call_id, peer_id)).unwrap(),
            Control::GetCallState(call_id, ret) =>
                ret.send(self.get_call_state(call_id)).unwrap(),
            Control::CapabilitySupported(call_id, capability, ret) =>
                ret.send(self.capability_supported(call_id, capability)).unwrap(),
            Control::GetActiveCount(ret) =>
                ret.send(self.get_active_count()).unwrap(),
            Control::AddAvGroupchat(ret) =>
                ret.send(self.add_av_groupchat()).unwrap(),
            Control::JoinAvGroupchat(friend_id, data, ret) =>
                ret.send(self.join_av_groupchat(friend_id, data)).unwrap(),
            Control::GroupSendAudio(group_id, bit, ret) =>
                ret.send(self.group_send_audio(group_id, bit)).unwrap(),
        }
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        unsafe { toxav_kill(self.raw); }
    }
}

struct Internal {
    stop: bool,
    events: spsc::bounded::Producer<Event>,
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
        match $internal.events.send_sync($event) {
            Ok(()) => { },
            _ => $internal.stop = true,
        }
    }
}

macro_rules! state_callback {
    ($f:ident, $event:ident) => {
        extern fn $f(_: *mut c_void, call_idx: i32, arg: *mut c_void) {
            let internal = get_int!(arg);
            send_or_stop!(internal, Event::$event(call_idx));
        }
    }
}

state_callback!(on_invite,          Invite);
state_callback!(on_ringing,         Ringing);
state_callback!(on_start,           Start);
state_callback!(on_cancel,          Cancel);
state_callback!(on_reject,          Reject);
state_callback!(on_end,             End);
state_callback!(on_request_timeout, RequestTimeout);
state_callback!(on_peer_timeout,    PeerTimeout);
state_callback!(on_peer_cs_change,  PeerCsChange);
state_callback!(on_self_cs_change,  SelfCsChange);

extern fn on_audio(_agent: *mut c_void, _call_idx: i32, _pcm: *const i16, _size: u16,
                   _data: *mut c_void) {
    unimplemented!();
}

extern fn on_group_audio(_: *mut Tox, group_id: c_int, peer_id: c_int,
                         pcm: *const i16, samples: c_uint, channels: u8,
                         sample_rate: c_uint, userdata: *mut c_void) {
    let internal = get_int!(userdata);
    let pcm = unsafe {
        slice::from_raw_parts(pcm, samples as usize * channels as usize).to_vec()
    };
    let bit = AudioBit {
        pcm: pcm,
        samples: samples as u32,
        channels: channels as u8,
        sample_rate: sample_rate as u32,
    };
    send_or_stop!(internal, Event::GroupAudio(group_id, peer_id, bit));
}

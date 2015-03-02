use core::ll::{Tox};

use comm::{spsc};

pub mod ll;
mod backend;

#[derive(Clone, Debug)]
pub enum Event {
    Invite(i32),
    Ringing(i32),
    Start(i32),
    Cancel(i32),
    Reject(i32),
    End(i32),
    RequestTimeout(i32),
    PeerTimeout(i32),
    PeerCsChange(i32),
    SelfCsChange(i32),
    GroupAudio(i32, i32, AudioBit),
}

unsafe impl Send for Event { }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum CallType {
    Audio = 192,
    Video,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum CallState {
    NonExistent = -1,
    Inviting,
    Starting,
    Active,
    Hold,
    HungUp,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum Error {
    ErrorNone                   = 0,
    ErrorUnknown                = -1,
    ErrorNoCall                 = -20,
    ErrorInvalidState           = -21,
    ErrorAlreadyInCallWithPeer  = -22,
    ErrorReachedCallLimit       = -23,
    ErrorInitializingCodecs     = -30,
    ErrorSettingVideoResolution = -31,
    ErrorSettingVideoBitrate    = -32,
    ErrorSplittingVideoPayload  = -33,
    ErrorEncodingVideo          = -34,
    ErrorEncodingAudio          = -35,
    ErrorSendingPayload         = -40,
    ErrorCreatingRtpSessions    = -41,
    ErrorNoRtpSession           = -50,
    ErrorInvalidCodecState      = -51,
    ErrorPacketTooLarge         = -52,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum Capability {
    AudioEncoding = 1 << 0,
    AudioDecoding = 1 << 1,
    VideoEncoding = 1 << 2,
    VideoDecoding = 1 << 3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CallSettings {
    pub call_type: CallType,

    pub video_bitrate: u32,
    pub max_video_width: u16,
    pub max_video_height: u16,

    pub audio_bitrate: u32,
    pub audio_frame_duration: u16,
    pub audio_sample_rate: u32,
    pub audio_channels: u32,
}

#[derive(Clone, Debug)]
pub struct AudioBit {
    pub pcm: Vec<i16>,
    pub samples: u32,
    pub channels: u8,
    pub sample_rate: u32,
}

impl AudioBit {
    pub fn validate(&self) -> bool {
        self.pcm.len() == self.samples as usize * self.channels as usize
    }
}

pub struct AvControl {
    control: spsc::one_space::Producer<'static, backend::Control>,
}

type ControlProducer = spsc::one_space::Producer<'static, backend::Control>;
pub type AvEvents = spsc::bounded::Consumer<'static, Event>;

macro_rules! forward {
    ($slf:expr, $name:expr, ($($pp:ident),+), ->) => {{
            let (snd, rcv) = spsc::one_space::new();
            $slf.control.send($name($($pp),*, snd)).map_err(|e|e.1).unwrap();
            rcv.recv_sync().unwrap()
    }};
    ($slf:expr, $name:expr, ->) => {{
            let (snd, rcv) = spsc::one_space::new();
            $slf.control.send($name(snd)).map_err(|e|e.1).unwrap();
            rcv.recv_sync().unwrap()
    }};
    ($slf:expr, $name:expr, ($($pp:ident),+)) => {{
            $slf.control.send($name($($pp),*)).unwrap()
    }};
    ($slf:expr, $name:expr) => {
            $slf.control.send($name).unwrap()
    };

}

impl AvControl {
    #[inline]
    pub fn new(tox: *mut Tox, max_calls: i32,
               send_end: spsc::one_space::Producer<'static, ()>)
                                -> Option<(AvControl, AvEvents)> {
        match backend::Backend::new(tox, max_calls, send_end) {
            Some((ctrl, events)) => Some((AvControl { control: ctrl }, events)),
            None => return None,
        }
    }

    #[inline]
    pub fn call(&self, friend_id: i32, settings: Option<Box<CallSettings>>,
                timeout: i32) -> Result<i32, i32> {
        forward!(self, backend::Control::Call, (friend_id, settings, timeout), ->)
    }

    #[inline]
    pub fn hangup(&self, call_id: i32) -> Result<(), i32> {
        forward!(self, backend::Control::Hangup, (call_id), ->)
    }

    #[inline]
    pub fn answer(&self, call_id: i32,
                  settings: Option<Box<CallSettings>>) -> Result<(), i32> {
        forward!(self, backend::Control::Answer, (call_id, settings), ->)
    }

    #[inline]
    pub fn reject(&self, call_id: i32) -> Result<(), i32> {
        forward!(self, backend::Control::Reject, (call_id), ->)
    }

    #[inline]
    pub fn cancel(&self, call_id: i32, peer_id: i32) -> Result<(), i32> {
        forward!(self, backend::Control::Cancel, (call_id, peer_id), ->)
    }

    #[inline]
    pub fn change_settings(&self, call_id: i32,
                           settings: Option<Box<CallSettings>>) -> Result<(), i32>{
        forward!(self, backend::Control::ChangeSettings, (call_id, settings), ->)
    }

    #[inline]
    pub fn stop_call(&self, call_id: i32) -> Result<(), i32> {
        forward!(self, backend::Control::StopCall, (call_id), ->)
    }

    #[inline]
    pub fn prepare_transmission(&self, call_id: i32,
                                support_video: bool) -> Result<(), i32>{
        forward!(self, backend::Control::PrepareTransmission, (call_id, support_video), ->)
    }

    #[inline]
    pub fn kill_transmission(&self, call_id: i32) -> Result<(), i32> {
        forward!(self, backend::Control::KillTransmission, (call_id), ->)
    }

    #[inline]
    pub fn prepare_audio_frame(&self, call_id: i32, dest: Vec<u8>,
                               src: Vec<i16>) -> Result<(Vec<u8>, Vec<i16>), (i32, Vec<u8>, Vec<i16>)> {
        forward!(self, backend::Control::PrepareAudioFrame, (call_id, dest, src), ->)
    }

    #[inline]
    pub fn send_audio(&self, call_id: i32,
                      src: Vec<u8>) -> Result<Vec<u8>, (i32, Vec<u8>)> {
        forward!(self, backend::Control::SendAudio, (call_id, src), ->)
    }

    #[inline]
    pub fn get_peer_call_settings(&self, call_id: i32,
                                  peer_id: i32) -> Result<Box<CallSettings>, i32> {
        forward!(self, backend::Control::GetPeerCallSettings, (call_id, peer_id), ->)
    }

    #[inline]
    pub fn get_peer_id(&self, call_id: i32, peer_id: i32) -> Result<i32, i32> {
        forward!(self, backend::Control::GetPeerId, (call_id, peer_id), ->)
    }

    #[inline]
    pub fn get_call_state(&self, call_id: i32) -> CallState {
        forward!(self, backend::Control::GetCallState, (call_id), ->)
    }

    #[inline]
    pub fn capability_supported(&self, call_id: i32,
                                capability: Capability) -> Result<bool, i32> {
        forward!(self, backend::Control::CapabilitySupported, (call_id, capability), ->)
    }

    #[inline]
    pub fn get_active_count(&self) -> Result<usize, i32> {
        forward!(self, backend::Control::GetActiveCount, ->)
    }

    #[inline]
    pub fn add_av_groupchat(&self) -> Result<i32, i32> {
        forward!(self, backend::Control::AddAvGroupchat, ->)
    }

    #[inline]
    pub fn join_av_groupchat(&self, friend_id: i32,
                             data: Vec<u8>) -> Result<i32, i32> {
        forward!(self, backend::Control::JoinAvGroupchat, (friend_id, data), ->)
    }

    #[inline]
    pub fn group_send_audio(&self, group_id: i32,
                            bit: AudioBit) -> Result<AudioBit, (i32, AudioBit)> {
        forward!(self, backend::Control::GroupSendAudio, (group_id, bit), ->)
    }
}

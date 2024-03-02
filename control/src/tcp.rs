use crate::audiocontrol::AudioMode;

pub enum TcpRequest {
    GoToMode(AudioMode),
    PlayModePlaylist(AudioMode, String),
}

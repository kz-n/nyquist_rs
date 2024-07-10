use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::track::TrackBuilder;

use bytes::Bytes;
use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc::Receiver};
use std::sync::Arc;
use std::time::Duration;
use kira::sound::FromFileError;
use kira::sound::PlaybackState::{Paused, Playing};
use kira::tween::Tween;

// Mutex-wrapped database type alias
pub type Db = Arc<Mutex<HashMap<String, Bytes>>>;

// Track structure representing a single audio track
#[derive(Clone, Debug)]
pub struct Track {
    pub path: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message {
    None,
    PlaylistUpdated,
    PlaybackPause,
    PlaybackResume,
    EffectVolume,
}

#[derive(Clone, Debug)]
pub struct MessageValue {
    pub float: Option<f64>,
    pub int: Option<i32>,
    pub string: Option<String>,
}
impl MessageValue {
    pub fn none() -> MessageValue {
        Self {
            float: None,
            int: None,
            string: None,
        }
    }

    pub fn float(float: f64) -> Self {
        Self { float: Some(float), int: (None), string: (None) }
    }

    pub fn int(int: Option<i32>) -> Self {
        Self { float: (None), int, string: (None) }
    }

    pub fn string(string: Option<String>) -> Self {
        Self { float: (None), int: (None), string }
    }
}

impl Track {
    fn new(path: String) -> Track {
        Track { path }
    }
}

// Playlist structure maintaining the queue of tracks and playback state
pub struct Playlist {
    pub queue: Vec<Track>,
    pub playing: Option<Track>,
    pub paused: bool,
}

// Creates the playlist for use in the program
pub fn create_playlist() -> Arc<Mutex<Playlist>> {
    Arc::new(Mutex::new(Playlist { queue: vec![], playing: None, paused: false }))
}

// Main audio thread that listens for messages and controls playback
pub async fn audio_thread(playlist: Arc<Mutex<Playlist>>, mut rx: Receiver<(Message, MessageValue)>) {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    let kira_track = manager.add_sub_track(TrackBuilder::default()).unwrap();
    let mut current_sound_handle: Option<StreamingSoundHandle<FromFileError>> = None; // Optional sound handle for the current playing sound

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = rx.recv() => {
                if let Some(message) = msg {
                    match message.0 {
                        Message::PlaybackPause => {
                            // Pause the current sound if it's playing
                            if let Some(mut handle) = current_sound_handle.take() {
                                if handle.state() == Playing {
                                    handle.pause(Tween::default());
                                    println!("Playback paused");
                                    let mut playlist_guard = playlist.lock().await;
                                    playlist_guard.paused = true;
                                    current_sound_handle = Some(handle);
                                }
                            }
                        }
                        Message::PlaybackResume => {
                            // Resume the current sound if it's paused
                            if let Some(mut handle) = current_sound_handle.take() {
                                if handle.state() == Paused {
                                    handle.resume(Tween::default());
                                    println!("Playback resumed");
                                    let mut playlist_guard = playlist.lock().await;
                                    playlist_guard.paused = false;
                                    current_sound_handle = Some(handle);
                                }
                            }
                        }
                        Message::EffectVolume => {
                            if let Some(mut handle) = current_sound_handle.take() {
                                handle.set_volume(message.1.float.unwrap(), Default::default());
                                current_sound_handle = Some(handle);
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Conditional block that runs if no sound is currently playing/paused
            _ = tokio::task::yield_now(), if current_sound_handle.is_none() => {
                let mut playlist_guard = playlist.lock().await;
                // Check if there is a track to play
                if let Some(playing) = &playlist_guard.playing {
                    let sound_data = StreamingSoundData::from_file(&playing.path).unwrap().output_destination(&kira_track);
                    current_sound_handle = Some(manager.play(sound_data).unwrap());
                    // Pause the sound immediately if the playlist is in a paused state
                    if playlist_guard.paused {
                        if let Some(mut handle) = current_sound_handle.take() {
                            handle.pause(Tween::default());
                            current_sound_handle = Some(handle);
                        }
                    }
                }
            }
        }

        // Check the state of the current sound handle and yield if it's playing/paused
        if let Some(mut handle) = current_sound_handle.take() {
            if handle.state() == Playing || handle.state() == Paused {
                tokio::task::yield_now().await;
                current_sound_handle = Some(handle); // Reassign the handle back
            }
        }
    }
}

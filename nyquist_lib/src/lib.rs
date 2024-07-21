use kira::manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::track::TrackBuilder;

use kira::sound::FromFileError;
use kira::sound::PlaybackState::{Paused, Playing};
use kira::tween::Tween;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, mpsc::Receiver, Mutex};

// Message Passer (useful to Nyquist struct)
struct MessagePasser {
    tx: Sender<(Message, MessageValue)>,
}
impl MessagePasser {
    pub fn new(tx: Sender<(Message, MessageValue)>) -> Self {
        //, rx: Receiver<(Message, MessageValue)>
        Self { tx } //, rx
    }
}

// Lib entry point, this object has to stay alive for the lib to function
pub struct Nyquist {
    pub playlist: Arc<Mutex<Playlist>>,
    message_passer: MessagePasser,
}

impl Nyquist {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<(Message, MessageValue)>(32);
        let playlist = create_playlist();

        // Correctly set playlist_clone
        let playlist_clone = Arc::clone(&playlist);
        tokio::spawn(audio_thread(playlist_clone, rx));

        // Set to used Arc<Mutex<Playlist>> correctly
        Self {
            playlist: playlist,
            message_passer: MessagePasser { tx },
        }
    }

    pub async fn add_to_playlist(&self, track: Track) {
        let mut playlist_guard = &mut self.playlist.lock().await;
        playlist_guard.queue.push(track.clone());
        playlist_guard.playing = Some(track);
        println!("bazinga")
    }

    pub async fn list(&self) -> Vec<Track> {
        let playlist_guard = &self.playlist.lock().await;
        return playlist_guard.queue.clone();
    }

    pub async fn pause_playback(&self) {
        &self
            .message_passer
            .tx
            .send((Message::PlaybackPause, MessageValue::none()))
            .await
            .unwrap();
    }

    pub async fn resume_playback(&self) {
        &self
            .message_passer
            .tx
            .send((Message::PlaybackResume, MessageValue::none()))
            .await
            .unwrap();
    }

    pub async fn get_time(&self) -> (Duration, Duration) {
        let playlist_guard = &self.playlist.lock().await;
        return (playlist_guard.current_duration, playlist_guard.current_time);
    }
    pub async fn get_vol(&self) -> f64 {
        return self.playlist.lock().await.current_volume;
    }

    pub async fn set_vol(&self, vol: f64) {
        self.playlist.lock().await.current_volume = vol;
    }
}

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
        Self {
            float: Some(float),
            int: (None),
            string: (None),
        }
    }

    pub fn int(int: Option<i32>) -> Self {
        Self {
            float: (None),
            int,
            string: (None),
        }
    }

    pub fn string(string: Option<String>) -> Self {
        Self {
            float: (None),
            int: (None),
            string,
        }
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
    pub current_duration: Duration,
    pub current_time: Duration,
    pub current_volume: f64,
}

// Creates the playlist for use in the program
pub fn create_playlist() -> Arc<Mutex<Playlist>> {
    Arc::new(Mutex::new(Playlist {
        queue: vec![],
        playing: None,
        paused: false,
        current_duration: Default::default(),
        current_time: Default::default(),
        current_volume: 100.0,
    }))
}

// Main audio thread that listens for messages and controls playback
pub async fn audio_thread(
    playlist: Arc<Mutex<Playlist>>,
    mut rx: Receiver<(Message, MessageValue)>,
) {
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
                                let mut playlist_guard = playlist.lock().await;
                                playlist_guard.current_volume = message.1.float.unwrap();
                                handle.set_volume(playlist_guard.current_volume, Default::default());
                                current_sound_handle = Some(handle);
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ =
                tokio::task::yield_now()
            , if true => {
                // This block checks for new tracks to play if none are currently playing/paused
                if current_sound_handle.is_none() {
                    let mut playlist_guard = playlist.lock().await;
                    // Check if there is a track to play
                    if let Some(playing) = &playlist_guard.playing {
                        let sound_data = StreamingSoundData::from_file(&playing.path).unwrap().output_destination(&kira_track);
                        playlist_guard.current_duration = sound_data.duration().clone();
                        current_sound_handle = Some(manager.play(sound_data).unwrap());
                        // Pause the sound immediately if the playlist is in a paused state
                        if playlist_guard.paused {
                            if let Some(mut handle) = current_sound_handle.take() {
                                handle.pause(Tween::default());
                                current_sound_handle = Some(handle);
                            }
                        }
                    }
                // Updates current time
                } else {
                    if let Some(mut handle) = current_sound_handle.as_mut() {
                            let mut playlist_guard = playlist.lock().await;
                            playlist_guard.current_time = Duration::from_secs_f64(handle.position());
                    }
                }

            }
        }

        // Update the current playback time for the track (runs every loop iteration)

        // Yield execution to avoid blocking
        //tokio::task::yield_now().await;
    }
}

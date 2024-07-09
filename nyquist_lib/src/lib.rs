use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::streaming::StreamingSoundData;
use kira::track::TrackBuilder;

use bytes::Bytes;
use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc::Receiver};
use std::sync::Arc;
use std::time::Duration;
use kira::sound::PlaybackState::{Paused, Playing};
use kira::tween::Tween;

// Mutex stuff
pub type Db = Arc<Mutex<HashMap<String, Bytes>>>;

// Track and Playlist
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
}

impl Track {
    fn new(path: String) -> Track {
        Track { path }
    }
}

pub struct Playlist {
    pub queue: Vec<Track>,
    pub playing: Option<Track>,
    pub paused: bool,
}

// Creates the playlist for use in the program
pub fn create_playlist() -> Arc<Mutex<Playlist>> {
    Arc::new(Mutex::new(Playlist { queue: vec![], playing: None, paused: false }))
}

pub async fn test(path: String, db: &Arc<Mutex<HashMap<String, String>>>) {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    let track = manager.add_sub_track(TrackBuilder::default()).unwrap();
    let sound_data = StreamingSoundData::from_file(path).unwrap().output_destination(&track);
    manager.play(sound_data).unwrap();

    loop {
        let db = db.lock().await;
        print!("{:#?}", db.get("test").unwrap());
    }
}

pub async fn audio_thread(playlist: Arc<Mutex<Playlist>>, mut rx: Receiver<Message>) {
    let arc_message: Arc<Mutex<Option<Message>>> = Arc::new(Mutex::new(None));
    let arc_message_clone = arc_message.clone();
    tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            *arc_message_clone.lock().await = Some(m);
            println!("{:?}", arc_message_clone.lock().await);
        }
    });
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    let kira_track = manager.add_sub_track(TrackBuilder::default()).unwrap();
    loop {
        tokio::task::yield_now().await;

        let option = arc_message.lock().await.clone();
        let message = option.unwrap_or(Message::None);
        let mut playlist_guard = playlist.lock().await;
        if let Some(playing) = &playlist_guard.playing {
            {
                println!("sexo");
                let nyq_track = playing.clone();
                let sound_data = StreamingSoundData::from_file(nyq_track.path).unwrap().output_destination(&kira_track);
                let mut sound_handle = manager.play(sound_data).unwrap();
                sound_handle.pause(Tween {
                    start_time: Default::default(),
                    duration: Duration::from_millis(0),
                    easing: Default::default(),
                });
                if !playlist_guard.paused {
                    println!("asd");
                    sound_handle.resume(Default::default());
                }
                while sound_handle.state() == Playing || sound_handle.state() == Paused {
                    tokio::task::yield_now().await;

                    let option = arc_message.lock().await.clone();
                    if let Some(message) = option {
                        match message {
                            Message::PlaybackPause => {
                                if sound_handle.state() == Playing {
                                    sound_handle.pause(Default::default());
                                    println!("Playback paused");
                                    playlist_guard.paused = true;
                                }
                            }
                            Message::PlaybackResume => {
                                if sound_handle.state() == Paused {
                                    sound_handle.resume(Default::default());
                                    println!("Playback resumed");
                                    playlist_guard.paused = false;
                                }
                            }
                            _ => {}
                        }
                        // Clear the message after handling
                        *arc_message.lock().await = None;
                    }
                }
            }
        }
    }
}

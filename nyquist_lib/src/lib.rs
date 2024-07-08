use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::streaming::StreamingSoundData;
use kira::track::TrackBuilder;

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use kira::sound::PlaybackState::{Paused, Playing};
use tokio::sync::mpsc::Receiver;

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
    PlaybackResume
}

impl Track {
    fn new(path: String) -> Track {
        Track { path }
    }
}

pub struct Playlist {
    pub queue: Vec<Track>,
    pub playing: Option<Track>,
}

// Creates the playlist for use in the program
pub fn create_playlist() -> Arc<Mutex<Playlist>> {
    Arc::new(Mutex::new(Playlist { queue: vec![], playing: None }))
}

pub async fn test(path: String, db: &Arc<Mutex<HashMap<String, String>>>) {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    let track = manager.add_sub_track(TrackBuilder::default()).unwrap();
    let sound_data = StreamingSoundData::from_file(path).unwrap().output_destination(&track);
    manager.play(sound_data).unwrap();

    loop {
        let db = db.lock().unwrap();
        print!("{:#?}", db.get("test").unwrap());
    }
}

pub async fn audio_thread(playlist: &Arc<Mutex<Playlist>>, mut rx: Receiver<Message>) {
    let mut arc_message: Arc<Mutex<Option<Message>>>= Arc::new(Mutex::new(None));
    let mut arc_message_clone = arc_message.clone();
    tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            (*arc_message_clone.lock().unwrap()) = Some(m);
            print!("{:?}", arc_message_clone.lock().unwrap())
        }
    });
    let mut should_loop = true;
    while should_loop {
        tokio::task::yield_now().await;

        let option = arc_message.lock().unwrap().clone();
        let mut message = Message::None;
        if !option.is_none() {
            message = option.unwrap();
        }
        let playing = playlist.lock().unwrap().playing.clone();
        if !playing.is_none() {
            let nyq_track = playing.unwrap();
            let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
            let kira_track = manager.add_sub_track(TrackBuilder::default()).unwrap();
            let sound_data = StreamingSoundData::from_file(nyq_track.path).unwrap().output_destination(&kira_track);
            let mut sound_handle = manager.play(sound_data).unwrap();
            while sound_handle.state() == Playing {

                if message == Message::PlaybackPause {
                    sound_handle.pause(Default::default())
                }
                should_loop = false;
                tokio::task::yield_now().await;
            }
            while sound_handle.state() == Paused {
                tokio::task::yield_now().await;
                if message == Message::PlaybackResume {
                    sound_handle.resume(Default::default())
                }
                should_loop = true;
                tokio::task::yield_now().await;
            }
            should_loop = true;
        }
    }
}

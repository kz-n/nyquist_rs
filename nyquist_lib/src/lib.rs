use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::streaming::StreamingSoundData;
use kira::track::TrackBuilder;

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Mutex stuff
pub type Db = Arc<Mutex<HashMap<String, Bytes>>>;

// Track and Playlist
#[derive(Clone)]
pub struct Track {
    pub path: String,
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

pub async fn audio_thread(playlist: &Arc<Mutex<Playlist>>) {
    loop {
        let nyq_track_option = playlist.lock().unwrap().playing.clone();
        if let Some(nyq_track) = nyq_track_option {
            let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
            let kira_track = manager.add_sub_track(TrackBuilder::default()).unwrap();
            let sound_data = StreamingSoundData::from_file(nyq_track.path).unwrap().output_destination(&kira_track);
            manager.play(sound_data).unwrap();
        } else {
            // No track is currently playing, so wait and check again
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

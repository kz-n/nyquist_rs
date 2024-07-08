use kira::manager::{
    AudioManager, AudioManagerSettings,
    backend::DefaultBackend,
};
use kira::sound::streaming::StreamingSoundData;
use kira::track::TrackBuilder;

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<HashMap<String, Bytes>>>;

pub async fn test(path: String, db: &Arc<Mutex<HashMap<String, String>>>) {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    let track = manager.add_sub_track(TrackBuilder::default()).unwrap();
    let sound_data = StreamingSoundData::from_file(path).unwrap().output_destination(&track);
    manager.play(sound_data).unwrap();

    loop {
        let mut db = db.lock().unwrap();
        print!("{:#?}", db.get("test").unwrap())
    }
}
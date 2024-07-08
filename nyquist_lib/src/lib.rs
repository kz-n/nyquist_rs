use kira::manager::{
    AudioManager, AudioManagerSettings,
    backend::DefaultBackend,
};
use kira::sound::streaming::StreamingSoundData;
use kira::track::TrackBuilder;

pub async fn test(path: String) {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
    let track = manager.add_sub_track(TrackBuilder::default()).unwrap();
    let sound_data = StreamingSoundData::from_file(path).unwrap().output_destination(&track);
    manager.play(sound_data).unwrap();

    loop {}
}
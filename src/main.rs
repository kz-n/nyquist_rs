use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use nyquist_lib;
use nyquist_lib::{audio_thread, create_playlist, Track};

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let playlist = create_playlist();
    println!("Hello, world!");

    // Clone the Arc for the tokio task
    let playlist_clone = Arc::clone(&playlist);
    tokio::spawn(async move {
        audio_thread(&playlist_clone).await;
    });

    use std::io::{stdin, stdout, Write};
    loop {
        let mut input = String::new();
        print!("Please enter some text: ");
        let _ = stdout().flush();
        stdin().read_line(&mut input).expect("Did not enter a correct string");

        if let Some('\n') = input.chars().next_back() {
            input.pop();
        }
        if let Some('\r') = input.chars().next_back() {
            input.pop();
        }

        // Create a new Arc clone for each iteration
        let db_clone = Arc::clone(&db);

        if input.starts_with("add") {
            let playlist_add = input.replace("add ", "");
            let track = Track { path: playlist_add };

            let mut playlist_guard = playlist.lock().unwrap();
            playlist_guard.queue.push(track.clone());
            playlist_guard.playing = Some(track);
        }

        if input.starts_with("play") {
            let _play = input.replace("play ", "");
            // Implementation for play (currently does nothing)
        }
    }
}

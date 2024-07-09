use nyquist_lib::Track;
use nyquist_lib::{audio_thread, create_playlist};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{select, task};
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use nyquist_lib::Message;
// Your library crate name

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let db = Arc::new(tokio::sync::Mutex::new(HashMap::<String, String>::new()));
    let playlist = create_playlist();
    let (tx, rx) = mpsc::channel::<Message>(32);
    println!("Hello, world!");

    let playlist_clone = Arc::clone(&playlist);
    tokio::spawn(audio_thread(playlist_clone, rx));

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    loop {
        select! {
            Ok(Some(input)) = stdin.next_line() => {
                let input = input.trim();
                let db_clone = Arc::clone(&db);

                if input.starts_with("add") {
                    let playlist_add = input.replace("add ", "");
                    let track = Track { path: playlist_add };

                    let mut playlist_guard = playlist.lock().await;
                    playlist_guard.queue.push(track.clone());
                    playlist_guard.playing = Some(track);
                } else if input.starts_with("play") {
                    let _play = input.replace("play ", "");
                    // Implementation for play (currently does nothing)
                } else if input.starts_with("list") {
                    let playlist_guard = playlist.lock().await;
                    println!("{:#?}", playlist_guard.queue);
                } else if input.starts_with("pause") {
                    tx.send(Message::PlaybackPause).await.unwrap();
                } else if input.starts_with("resume") {
                    tx.send(Message::PlaybackResume).await.unwrap();
                }
            }
            else => {
                break;
            }
        }
    }
}

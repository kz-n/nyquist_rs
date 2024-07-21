use nyquist_lib::Message;
use nyquist_lib::{audio_thread, create_playlist};
use nyquist_lib::{MessageValue, Nyquist, Track};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::{select, task};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let nyquist = Nyquist::new();
    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    loop {
        select! {
            Ok(Some(input)) = stdin.next_line() => {
                let input = input.trim();

                if input.starts_with("add") {
                    let playlist_add = input.replace("add ", "");
                    let track = Track { path: playlist_add };
                    nyquist.add_to_playlist(track).await;

                } else if input.starts_with("play") {
                    let _play = input.replace("play ", "");
                    // Implementation for play (add more if needed)

                } else if input.starts_with("list") {
                    println!("{:#?}", nyquist.list().await);

                } else if input.starts_with("pause") {
                    nyquist.pause_playback().await;

                } else if input.starts_with("resume") {
                    nyquist.resume_playback().await;

                } else if input.starts_with("vol") {
                    let vol = input.replace("vol ", "").parse::<i32>().unwrap();
                    nyquist.set_vol(vol as f64 / 100.0).await;

                } else if input.starts_with("time") {
                    println!("{:?}", nyquist.get_time().await)
                }
            }
            else => {
                tokio::task::yield_now().await;
            }
        }
    }
    // let playlist = create_playlist();
    // let (tx, rx) = mpsc::channel::<(Message, MessageValue)>(32);
    // println!("Hello, world!");
    //
    // let playlist_clone = Arc::clone(&playlist);
    // tokio::spawn(audio_thread(playlist_clone, rx));
    //
    // let mut stdin = BufReader::new(tokio::io::stdin()).lines();
    //
    // loop {
    //     select! {
    //         Ok(Some(input)) = stdin.next_line() => {
    //             let input = input.trim();
    //
    //             if input.starts_with("add") {
    //                 let playlist_add = input.replace("add ", "");
    //                 let track = Track { path: playlist_add };
    //
    //                 let mut playlist_guard = playlist.lock().await;
    //                 playlist_guard.queue.push(track.clone());
    //                 playlist_guard.playing = Some(track);
    //             } else if input.starts_with("play") {
    //                 let _play = input.replace("play ", "");
    //                 // Implementation for play (currently does nothing)
    //             } else if input.starts_with("list") {
    //                 let playlist_guard = playlist.lock().await;
    //                 println!("{:#?}", playlist_guard.queue);
    //             } else if input.starts_with("pause") {
    //                 tx.send((Message::PlaybackPause, MessageValue::none())).await.unwrap();
    //             } else if input.starts_with("resume") {
    //                 tx.send((Message::PlaybackResume, MessageValue::none())).await.unwrap();
    //             } else if input.starts_with("vol") {
    //                 let vol = input.replace("vol ", "").parse::<i32>().unwrap();
    //                 tx.send((Message::EffectVolume, MessageValue::float(vol as f64 / 100.0))).await.unwrap();
    //             } else if input.starts_with("time") {
    //                 let playlist_guard = playlist.lock().await;
    //                 println!("{:?} {:?}" ,playlist_guard.current_duration, playlist_guard.current_time)
    //             }
    //         }
    //         else => {
    //             // yield execution to allow other tasks to run
    //             tokio::task::yield_now().await;
    //         }
    //     }
    // }
}

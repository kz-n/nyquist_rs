use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use nyquist_lib;
use nyquist_lib::test;

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    println!("Hello, world!");
    use std::io::{stdin,stdout,Write};
    loop {
        let mut s=String::new();
        print!("Please enter some text: ");
        let _ =stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        if let Some('\n')=s.chars().next_back() {
            s.pop();
        }
        if let Some('\r')=s.chars().next_back() {
            s.pop();
        }// Inside your loop, create a new Arc clone for each iteration
        let db_clone = Arc::clone(&db);

        if s.starts_with("setdb") {
            let mut db_add = s.replace("setdb ", "");
            let mut guard = db_clone.lock().unwrap();
            if guard.contains_key("test") {
                *guard.get_mut("test").unwrap() = db_add;
            } else {
                guard.insert("test".to_string(), s.clone());
            }
        }

        if s.starts_with("play") {
            let mut play = s.replace("play ", "");
        }

        // Spawn the async block using the cloned Arc
        tokio::spawn(async move {
            test("/run/media/kz-n/Extra/Musica/Fox Stevenson/Fox Stevenson & Curbi - Hoohah.flac".to_string(), &db_clone).await;
        });

    }
}

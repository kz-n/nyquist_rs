use nyquist_lib;
use nyquist_lib::test;

#[tokio::main]
async fn main() {

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
        }
        println!("You typed: {}",s);
        tokio::spawn(async move {
            test(s.clone()).await;
        });
    }
}

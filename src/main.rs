use clap::Parser;
use ord::{
    index::{event::Event, Index},
    options::Options,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

static SHUTDOWN_SIGNAL: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    println!("Starting custom ord indexer...");

    let index_options = Options::parse();
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Event>(128);
    let index = Index::open_with_event_sender(&index_options, Some(sender)).unwrap();

    // Handle Ctrl-C
    ctrlc::set_handler(move || {
        println!("Received Ctrl-C, shutting down...");
        SHUTDOWN_SIGNAL.store(true, Ordering::SeqCst);
    })
    .unwrap();

    let receiver_handle = tokio::spawn(async move {
        while !SHUTDOWN_SIGNAL.load(Ordering::SeqCst) {
            if let Some(event) = receiver.recv().await {
                match event {
                    Event::InscriptionCreated { .. } => {
                        println!("Inscription created: {:?}", event);
                    }
                    Event::InscriptionTransferred { .. } => {
                        println!("Inscription moved: {:?}", event);
                    }
                }
            }
        }
    });

    let index_handle = thread::spawn(move || loop {
        if SHUTDOWN_SIGNAL.load(Ordering::SeqCst) {
            break;
        }
        index.update().unwrap();
        thread::sleep(std::time::Duration::from_secs(3));
    });

    receiver_handle.await.unwrap();
    index_handle.join().unwrap();

    println!("Finished custom ord indexer.");
}

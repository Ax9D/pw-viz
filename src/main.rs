use simple_logger::SimpleLogger;
use std::{rc::Rc, thread};

mod pipewire_impl;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if SimpleLogger::new().init().is_err() {
        println!("Failed to init logger");
    }

    let (sender, receiver) = std::sync::mpsc::channel();
    let (pwsender, pwreciever) = pipewire::channel::channel();

    let pw_thread_handle = thread::spawn(move || {
        let sender = Rc::new(sender);

        pipewire_impl::thread_main(sender, pwreciever).expect("Failed to init pipewire client");
    });

    ui::run_graph_ui(receiver, pwsender);

    pw_thread_handle.join().expect("👽👽👽");

    Ok(())
}

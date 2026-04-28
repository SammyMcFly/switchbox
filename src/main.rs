// SPDX-License-Identifier: MPL-2.0

mod app;
mod pipewire_connection;
mod config;
mod i18n;
use std::{sync::mpsc, thread};

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(500.0)
            .min_height(320.0),
    );

    // init pipewire
    let (main_sender, main_receiver) = mpsc::channel();
    let (pw_sender, pw_receiver) = pipewire::channel::channel();

    let pw_thread = thread::spawn(move || pipewire_connection::pw_thread(main_sender, pw_receiver));

    // Count up to three "Hello"'s.
    let mut n = 0;
    while n < 3 {
        match main_receiver.recv().unwrap() {
            pipewire_connection::PipewireMessage::Message(s) => {
                println!("{s}");
            }
        }
        n += 1;
    }

    // Starts the application's event loop with `()` as the application's flags.
    let _ = cosmic::app::run::<app::AppModel>(settings, ());

    // Terminate the pipewire thread
    pw_sender.send(pipewire_connection::PipewireCommand::Terminate).expect("Failed to send termination message to pipewire thread");
    pw_thread.join().expect("Failed join pipewire thread");

    Ok(())
}

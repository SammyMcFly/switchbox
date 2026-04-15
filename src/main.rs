// SPDX-License-Identifier: MPL-2.0

mod app;
mod config;
mod i18n;
use std::{time::Duration, sync::mpsc, thread};

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(360.0)
            .min_height(180.0),
    );

    // init pipewire
    let (main_sender, main_receiver) = mpsc::channel();
    let (pw_sender, pw_receiver) = pipewire::channel::channel();

    let pw_thread = thread::spawn(move || pw_thread(main_sender, pw_receiver));

    // Count up to three "Hello"'s.
    let mut n = 0;
    while n < 3 {
        println!("{}", main_receiver.recv().unwrap());
        n += 1;
    }

    // Starts the application's event loop with `()` as the application's flags.
    let _ = cosmic::app::run::<app::AppModel>(settings, ());

    // Terminate the pipewire thread
    let _ = pw_sender.send(Terminate);
    let _ = pw_thread.join();

    Ok(())
}

struct Terminate;

// This is the code that will run in the pipewire thread.
fn pw_thread(
    main_sender: mpsc::Sender<String>,
    pw_receiver: pipewire::channel::Receiver<Terminate>
) {
    let mainloop = pipewire::main_loop::MainLoopRc::new(None).expect("Failed to create main loop");

    // When we receive a `Terminate` message, quit the main loop.
    let _receiver = pw_receiver.attach(mainloop.loop_(), {
        let mainloop = mainloop.clone();
        move |_| mainloop.quit()
    });

    // Every 100ms, send `"Hello"` to the main thread.
    let timer = mainloop.loop_().add_timer(move |_| {
        let _ = main_sender.send(String::from("Hello"));
    });
    timer.update_timer(
        Some(Duration::from_millis(1)), // Send the first message immediately
        Some(Duration::from_millis(100))
    );

    mainloop.run();
}

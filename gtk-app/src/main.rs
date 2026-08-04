mod app;
mod daemon;
mod model;
mod theme;
mod ui;

use adw::prelude::*;
use gtk::glib;

pub const DEFAULT_ART: &[u8] = include_bytes!("../../app/assets/placeholder.png");

const APP_ID: &str = "com.mavenried.AuroraPlayerGtk";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();

    let application = adw::Application::builder().application_id(APP_ID).build();

    application.connect_activate(|app| {
        let shared = ui::build_root(app);

        let (req_tx, evt_rx) = daemon::start();
        shared.borrow_mut().req_tx = Some(req_tx);

        let shared_loop = shared.clone();
        glib::spawn_future_local(async move {
            while let Ok(event) = evt_rx.recv().await {
                app::handle_daemon_event(&shared_loop, event);
            }
        });

        shared.borrow().widgets.window.present();
    });

    application.run()
}

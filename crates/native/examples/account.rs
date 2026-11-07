//! `cargo run -p oxdrive-native --example account -- <profile> <config_dir>` prints what the
//! account fetch reports: address, title, and the photo size.

use futures::StreamExt as _;

fn main() -> std::process::ExitCode {
    let (Some(profile), Some(dir)) = (std::env::args().nth(1), std::env::args().nth(2)) else {
        eprintln!("usage: account <profile> <config_dir>");
        return std::process::ExitCode::FAILURE;
    };
    let (tx, rx) = futures::channel::mpsc::unbounded();
    oxdrive_native::account::fetch(&profile, std::path::Path::new(&dir), tx);
    futures::executor::block_on(rx.for_each(|event| {
        match event {
            oxdrive_native::client::ClientEvent::Email(email) => println!("email {email}"),
            oxdrive_native::client::ClientEvent::Title(title) => println!("title {title}"),
            oxdrive_native::client::ClientEvent::Avatar(bytes) => {
                println!("photo {} bytes", bytes.len());
            }
            other => println!("{other:?}"),
        }
        futures::future::ready(())
    }));
    std::process::ExitCode::SUCCESS
}

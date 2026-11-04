//! `cargo run -p oxdrive-native --example realm -- user@example.com` prints the flyout title.

fn main() -> std::process::ExitCode {
    let Some(email) = std::env::args().nth(1) else {
        eprintln!("usage: realm <email>");
        return std::process::ExitCode::FAILURE;
    };
    match oxdrive_native::realm::lookup(&email) {
        Ok(title) => {
            println!("{title}");
            std::process::ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("{why}");
            std::process::ExitCode::FAILURE
        }
    }
}

use std::process::ExitCode;

fn main() -> ExitCode {
    match oxdrive_ui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("oxdrive: {message}");
            ExitCode::FAILURE
        }
    }
}

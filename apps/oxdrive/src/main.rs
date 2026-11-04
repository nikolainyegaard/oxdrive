use std::process::ExitCode;

fn main() -> ExitCode {
    // The onedrive client found us on its PATH as `xdg-open`: open the sign-in popup and leave.
    if oxdrive_native::popup::called_as_shim() {
        let url = std::env::args().nth(1).unwrap_or_default();
        return match oxdrive_native::popup::open(&url) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("oxdrive: {message}");
                ExitCode::FAILURE
            }
        };
    }
    match oxdrive_ui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("oxdrive: {message}");
            ExitCode::FAILURE
        }
    }
}

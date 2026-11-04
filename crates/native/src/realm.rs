//! Asks Microsoft which tenant a sign-in address belongs to, through `curl`, the way the sign-in
//! page does before it shows a password box. No token, no scope, no HTTP crate.

use std::fmt::Write as _;
use std::process::Command;

/// The flyout title for the account behind `email`, see `oxdrive_core::realm::title`.
///
/// # Errors
/// `curl` is missing, the request failed, or the reply names no tenant.
pub fn lookup(email: &str) -> oxdrive_core::Result<String> {
    let url = format!(
        "https://login.microsoftonline.com/getuserrealm.srf?login={}&json=1",
        encode(email)
    );
    let output = Command::new("curl")
        .args(["--silent", "--show-error", "--max-time", "10", &url])
        .output()
        .map_err(|e| format!("could not run curl: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "the realm lookup failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    oxdrive_core::realm::title(&String::from_utf8_lossy(&output.stdout))
        .ok_or_else(|| format!("Microsoft does not know a tenant for {email}"))
}

/// Percent-encodes everything but the unreserved characters.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

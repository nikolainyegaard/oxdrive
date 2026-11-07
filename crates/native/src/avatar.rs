//! The account photo: Graph `/me/photo/$value`, with an access token refreshed from the
//! client's own `refresh_token`. Needs `User.Read`, which the sign-in popup adds to the client's
//! scopes (popup.rs); an account signed in before that has no photo until it signs in again.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

use futures::channel::mpsc::UnboundedSender;

use crate::client::ClientEvent;

/// The client's public application id (abraunegg/onedrive `config.d`, `defaultApplicationId`).
const CLIENT_ID: &str = "d50ca740-c83f-4d1b-b616-12c519384f0c";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const REDIRECT: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";
const PHOTO_URL: &str = "https://graph.microsoft.com/v1.0/me/photo/$value";

fn cache_path(profile: &str) -> Option<PathBuf> {
    Some(crate::settings::state_dir()?.join(format!("{profile}.jpg")))
}

/// The photo from the last fetch, as Graph sent it (JPEG). None when there never was one.
#[must_use]
pub fn load(profile: &str) -> Option<Vec<u8>> {
    std::fs::read(cache_path(profile)?).ok()
}

/// Fetches the photo on its own thread, caches it and reports it as `ClientEvent::Avatar`.
/// Nothing is sent when there is no token, no `User.Read`, no photo or no network.
pub fn fetch(profile: &str, config_dir: &Path, events: UnboundedSender<ClientEvent>) {
    let profile = profile.to_owned();
    let config_dir = config_dir.to_owned();
    thread::spawn(move || {
        if let Some(bytes) = download(&config_dir) {
            if let Some(cache) = cache_path(&profile) {
                let _ = std::fs::write(cache, &bytes);
            }
            let _ = events.unbounded_send(ClientEvent::Avatar(bytes));
        }
    });
}

fn download(config_dir: &Path) -> Option<Vec<u8>> {
    let refresh = std::fs::read_to_string(config_dir.join("refresh_token")).ok()?;
    // ponytail: tenant is `common` like the client's default; an `azure_tenant_id` in the
    // client config is not read. Microsoft keeps the old refresh token valid after a refresh,
    // so the client's copy on disk stays good. Both tokens go in on stdin, never on argv.
    let body = format!(
        "client_id={CLIENT_ID}&redirect_uri={REDIRECT}&grant_type=refresh_token&refresh_token={}",
        refresh.trim()
    );
    let json = curl(&[TOKEN_URL, "-d", "@-"], &body)?;
    let token = access_token(&String::from_utf8_lossy(&json))?.to_owned();
    curl(
        &["-K", "-"],
        &format!("url = \"{PHOTO_URL}\"\nheader = \"Authorization: Bearer {token}\"\n"),
    )
}

/// `-f` turns a 401 (no `User.Read`) or 404 (no photo) into a failure, so None.
fn curl(args: &[&str], stdin: &str) -> Option<Vec<u8>> {
    let mut child = Command::new("curl")
        .args(["-sf", "--max-time", "30"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(stdin.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    out.status.success().then_some(out.stdout)
}

/// The `access_token` value of a token response, without a JSON parser: the token is
/// base64url and never holds a quote or a backslash.
fn access_token(json: &str) -> Option<&str> {
    json.split("\"access_token\":\"").nth(1)?.split('"').next()
}

#[cfg(test)]
mod tests {
    #[test]
    fn access_token_is_cut_out_of_the_response() {
        let json = r#"{"token_type":"Bearer","scope":"User.Read","access_token":"eyJ0.abc-_","refresh_token":"x"}"#;
        assert_eq!(super::access_token(json), Some("eyJ0.abc-_"));
        assert_eq!(super::access_token(r#"{"error":"invalid_grant"}"#), None);
    }
}

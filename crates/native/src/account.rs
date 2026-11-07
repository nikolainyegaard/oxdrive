//! Who the account is: the sign-in address, the flyout title and the photo, from one token
//! refresh with the client's own `refresh_token`. The address is in the `id_token` of the reply,
//! the title comes from `realm::lookup` on it, and the photo from Graph `/me/photo/$value`.
//! The photo needs `User.Read`, which the sign-in popup adds to the client's scopes (popup.rs);
//! an account signed in before that has no photo until it signs in again.

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
pub fn load_photo(profile: &str) -> Option<Vec<u8>> {
    std::fs::read(cache_path(profile)?).ok()
}

/// On its own thread: refreshes a token, then reports `ClientEvent::Email`, `Title` and
/// `Avatar` as each comes in. A part that fails (no token, no `User.Read`, no photo, no
/// network) is simply not sent.
pub fn fetch(profile: &str, config_dir: &Path, events: UnboundedSender<ClientEvent>) {
    let profile = profile.to_owned();
    let config_dir = config_dir.to_owned();
    thread::spawn(move || {
        let Some((email, token)) = refresh(&config_dir, "Files.ReadWrite") else {
            return;
        };
        // ponytail: the realm lookup runs on every client start, one small request to
        // Microsoft; cache it per address if it ever shows.
        if let Some(email) = email {
            if let Ok(title) = crate::realm::lookup(&email) {
                let _ = events.unbounded_send(ClientEvent::Title(title));
            }
            let _ = events.unbounded_send(ClientEvent::Email(email));
        }
        if let Some(bytes) = photo(&token) {
            if let Some(cache) = cache_path(&profile) {
                let _ = std::fs::write(cache, &bytes);
            }
            let _ = events.unbounded_send(ClientEvent::Avatar(bytes));
        }
    });
}

/// `(sign-in address, access token)` from a refresh with the client's token.
pub(crate) fn refresh(config_dir: &Path, scope: &str) -> Option<(Option<String>, String)> {
    let refresh = std::fs::read_to_string(config_dir.join("refresh_token")).ok()?;
    // ponytail: tenant is `common` like the client's default; an `azure_tenant_id` in the
    // client config is not read. Microsoft keeps the old refresh token valid after a refresh,
    // so the client's copy on disk stays good. Both tokens go in on stdin, never on argv.
    // `scope` is what makes the reply carry an `id_token`; a refresh without one returns
    // only the access token (probed 2026-11-07). Files.ReadWrite keeps it a Graph token.
    let body = format!(
        "client_id={CLIENT_ID}&redirect_uri={REDIRECT}&grant_type=refresh_token\
         &scope=openid%20profile%20email%20{scope}&refresh_token={}",
        refresh.trim()
    );
    let json = String::from_utf8_lossy(&curl(&[TOKEN_URL, "-d", "@-"], &body)?).into_owned();
    let token = field(&json, "access_token")?.to_owned();
    let email = field(&json, "id_token")
        .and_then(jwt_payload)
        .and_then(|payload| field(&payload, "preferred_username").map(str::to_owned));
    Some((email, token))
}

fn photo(token: &str) -> Option<Vec<u8>> {
    graph(PHOTO_URL, token)
}

/// A Graph GET as bytes; the token goes in on stdin, never on argv.
pub(crate) fn graph(url: &str, token: &str) -> Option<Vec<u8>> {
    curl(
        &["-K", "-"],
        &format!("url = \"{url}\"\nheader = \"Authorization: Bearer {token}\"\n"),
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

/// A string value of a top-level key, without a JSON parser. Enough for tokens and an
/// address, which hold no quote or backslash.
pub(crate) fn field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    json.split(&format!("\"{key}\":\""))
        .nth(1)?
        .split('"')
        .next()
}

/// The claims of a JWT, the base64url middle segment decoded. No signature check: the token
/// came straight from Microsoft over TLS.
fn jwt_payload(token: &str) -> Option<String> {
    let segment = token.split('.').nth(1)?;
    let mut bits: u32 = 0;
    let mut count = 0;
    let mut out = Vec::with_capacity(segment.len() * 3 / 4);
    for c in segment.bytes() {
        let value = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            b'=' => break,
            _ => return None,
        };
        bits = (bits << 6) | u32::from(value);
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push(u8::try_from((bits >> count) & 0xff).ok()?);
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn token_fields_and_claims_are_cut_out_of_the_response() {
        // The id_token payload is {"preferred_username":"a@b.c"} in base64url, no padding.
        let json = r#"{"token_type":"Bearer","access_token":"eyJ0.abc-_","id_token":"eyJhbGciOiJub25lIn0.eyJwcmVmZXJyZWRfdXNlcm5hbWUiOiJhQGIuYyJ9.","refresh_token":"x"}"#;
        assert_eq!(super::field(json, "access_token"), Some("eyJ0.abc-_"));
        assert_eq!(
            super::field(r#"{"error":"invalid_grant"}"#, "access_token"),
            None
        );
        let claims = super::field(json, "id_token").and_then(super::jwt_payload);
        assert_eq!(claims.as_deref(), Some(r#"{"preferred_username":"a@b.c"}"#));
    }
}

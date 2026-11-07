# The onedrive client

oxdrive drives `onedrive` (abraunegg), minimum v2.5.11, as a child process. The client has no JSON output and no IPC; everything is parsed from its stdout. This page records what is known so the parser is written against facts.

## Invocation

| Purpose | Command |
|---------|---------|
| Run and keep syncing | `onedrive --confdir <dir> --monitor --disable-notifications` |
| Sign in again, then keep syncing | `onedrive --confdir <dir> --monitor --reauth --disable-notifications` |
| One-off sync | `onedrive --confdir <dir> --synchronize` |
| Sign in without a terminal | `onedrive --confdir <dir> --auth-files <url-file>:<response-file>` |
| Sign in with a pasted URL | `onedrive --confdir <dir> --auth-response <url>` |
| Quota | `onedrive --confdir <dir> --display-quota`, runs fine beside the monitor |
| Effective config | `onedrive --confdir <dir> --display-config` |
| Sync status without syncing | `onedrive --confdir <dir> --display-sync-status` |

`<dir>` is the profile's config directory, for example `~/.config/onedrive/accounts/<name>`. The `config` file inside is the client's own INI-like format.

## Sign in

v2.5.11 redirects to a loopback listener the client opens itself (`http://127.0.0.1:<port>/?code=...`), observed on 2026-11-02 with a plain `onedrive --confdir <dir>` run. The client opens the browser itself, always `xdg-open <url>` from `PATH` (`$BROWSER` is ignored, probed 2026-11-04 with a shim that logged its arguments), and prints `Opening the Microsoft authorisation URL in your default browser ...` followed by `Waiting for the Microsoft authorisation response on http://127.0.0.1:<port>/`; it never prints the URL, even when the browser command fails. The browser lands on the loopback URL, the client takes the code and continues into monitor mode. So the Sign in button only reruns the client with `--reauth`; oxdrive parses no URL.

The popup: the `--reauth` child gets `$XDG_RUNTIME_DIR/oxdrive/bin` first on its `PATH`, holding an `xdg-open` symlink to the oxdrive binary. Started under that name, oxdrive opens the URL with the first Chromium-family browser on `PATH` (`google-chrome`, `chromium`, `microsoft-edge`, `brave`) in `--app` mode, 480 x 720, with its own profile under `~/.local/state/oxdrive/popup-profile`, writes the browser pid to `popup.pid` and exits. When the first sync after Blocked starts, the app sends that pid SIGTERM, so the window closes itself like the Windows client's. The own profile means no first-run prompts; it persists, so the Microsoft session is kept and a later sign-in is a click on the account, not a full login. Before opening, the shim appends `User.Read` to the URL's `scope`: the client asks for Files and Sites scopes only, and the account photo needs that one; the code exchange and the token refresh send no `scope`, so the consent sticks to the refresh token. No Chromium-family browser means no shim: the real xdg-open opens a tab in the default browser, and no `User.Read` is added. Firefox has no app mode. `--auth-files` and `--auth-response` still exist for the case where the browser runs on another machine; not used.

If the client that opened the tab is gone by the time the browser redirects, the browser shows "Unable to connect" to 127.0.0.1. Press Sign in again for a fresh tab.

The MFA-expired case is a real one (fixture `monitor-auth-expired.txt`): the client prints an `AADSTS50078` line and then `ERROR: You will need to issue a --reauth and re-authorise this client to obtain a fresh auth token.` and exits. That is the Blocked state.

## The account photo

`avatar.rs`. The client saves only its `refresh_token`, so oxdrive refreshes an access token of its own: `curl` posts the client's public application id (`d50ca740-c83f-4d1b-b616-12c519384f0c`), the nativeclient redirect and the refresh token from the profile's `config_dir` to `login.microsoftonline.com/common/oauth2/v2.0/token`, body on stdin. Microsoft keeps the old refresh token valid, so the client's copy still works. The token then fetches `graph.microsoft.com/v1.0/me/photo/$value` (a JPEG), cached as `~/.local/state/oxdrive/<profile>.jpg` and shown on the Account page. A 403 (signed in before `User.Read` was requested, probed 2026-11-07) or 404 (no photo) leaves the cached one, or the person glyph. Runs once per client start and once after a sign-in. "Sign in again" on the Account page reruns `--reauth` for an existing account so it picks the scope up.

## The tenant name

The Windows client titles its flyout `OneDrive - <tenant>` for work and school accounts and `OneDrive - Personal` for Microsoft accounts. The onedrive client never prints the tenant: it only says `Account Type: business` or `personal`, and the tenant name on Graph sits behind `GET /organization`, a scope the client does not ask for.

The sign-in page gets the name without any token. Before it knows where to send the browser it asks home realm discovery, `https://login.microsoftonline.com/getuserrealm.srf?login=<email>&json=1`, which is public and answers with `FederationBrandName`. `native::realm::lookup` runs that query through `curl` and `core::realm::title` turns the reply into the title: brand `Windows Live` means Personal, any other brand is the tenant, no brand means the domain is not Microsoft's. The sign-in address is the one input the client cannot give us; it comes from settings. Try it:

```
cargo run -p oxdrive-native --example realm -- user@example.com
```

## Fixtures

Captured from onedrive v2.5.11 on 2026-11-02, business account, Plasma session. Secrets, drive and root IDs, trace IDs, user name and real paths are replaced; the line formats are untouched. Monitor runs used `-v`, the same flag OneDriveGUI passes, so the fixtures show the verbose set and the plain set is a subset of it. Per-directory lines (`Monitoring directory:`, `Processing:`, `The file has not changed`, `Stopped monitoring directory`) repeat once per item in reality; the fixture keeps a few of each.

| File | Shows |
|------|-------|
| `display-sync-status.txt` | Healthy "no pending changes" plus the new-version banner |
| `display-quota.txt` | Deleted, Remaining, State, Total, Used with bytes in brackets |
| `display-config.txt` | Every `Config option '<key>' = <value>` line |
| `monitor-upload-delete.txt` | Start-up block, initial sync, a new local file uploaded, the same file deleted, a WebSocket signal, clean shutdown |
| `monitor-auth-expired.txt` | Start-up block ending in the AADSTS50078 error and the `--reauth` instruction |
| `monitor-sign-in.txt` | `--reauth` run up to the browser hand-off, captured 2026-11-03 with an empty config dir |

## Lines that drive state

From the fixtures above. Each row becomes a parser test.

| Line | Meaning |
|------|---------|
| `Starting a sync with Microsoft OneDrive` | Syncing |
| `Sync with Microsoft OneDrive is complete` | Idle |
| `Uploading new file: <name> ... done` | Activity entry, upload |
| `Successfully deleted item from Microsoft OneDrive: <name>` | Activity entry, delete |
| `[M] New local file added: <name>` | inotify event, informational |
| `Remaining Free Space:  <n> GB (<bytes> bytes)` | Quota, printed at start-up |
| `Total:` and `Used:` with bytes in brackets | Storage bar, from `--display-quota` |
| `Account Type:          <business|personal>` | Account line |
| `ERROR: You will need to issue a --reauth ...` | Blocked |
| `Opening the Microsoft authorisation URL in your default browser ...` | Blocked, with "Finish signing in in your browser" |
| `Received termination signal, attempting to cleanly shutdown application` | Disabled follows when the process exits |
| `INFO: A new onedrive client version is available.` | Ignore |

Still to capture: a download, a modified-file upload, a failed upload (`Failed to upload`), the `--resync is required` prompt, and a non-verbose run.

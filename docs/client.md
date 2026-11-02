# The onedrive client

oxdrive drives `onedrive` (abraunegg), minimum v2.5.11, as a child process. The client has no JSON output and no IPC; everything is parsed from its stdout. Nothing here is implemented yet; this page records what is known so the parser is written against facts.

## Invocation

| Purpose | Command |
|---------|---------|
| Run and keep syncing | `onedrive --confdir <dir> --monitor` |
| One-off sync | `onedrive --confdir <dir> --synchronize` |
| Sign in without a terminal | `onedrive --confdir <dir> --auth-files <url-file>:<response-file>` |
| Sign in with a pasted URL | `onedrive --confdir <dir> --auth-response <url>` |
| Quota | `onedrive --confdir <dir> --display-quota` |
| Effective config | `onedrive --confdir <dir> --display-config` |
| Sync status without syncing | `onedrive --confdir <dir> --display-sync-status` |

`<dir>` is the profile's config directory, for example `~/.config/onedrive/accounts/<name>`. The `config` file inside is the client's own INI-like format.

## Sign in

v2.5.11 redirects to a loopback listener the client opens itself (`http://127.0.0.1:<port>/?code=...`), observed on 2026-11-02 with a plain `onedrive --confdir <dir>` run. The browser lands on that URL and the client takes the code; nothing has to be pasted. `--auth-files` and `--auth-response` still exist for the case where the browser runs on another machine. The flyout therefore opens the URL and waits; a paste box is the fallback, shown only if the client keeps waiting.

The MFA-expired case is a real one (fixture `monitor-auth-expired.txt`): the client prints an `AADSTS50078` line and then `ERROR: You will need to issue a --reauth and re-authorise this client to obtain a fresh auth token.` and exits. That is the Blocked state.

## Fixtures

Captured from onedrive v2.5.11 on 2026-11-02, business account, Plasma session. Secrets, drive and root IDs, trace IDs, user name and real paths are replaced; the line formats are untouched. Monitor runs used `-v`, the same flag OneDriveGUI passes, so the fixtures show the verbose set and the plain set is a subset of it. Per-directory lines (`Monitoring directory:`, `Processing:`, `The file has not changed`, `Stopped monitoring directory`) repeat once per item in reality; the fixture keeps a few of each.

| File | Shows |
|------|-------|
| `display-sync-status.txt` | Healthy "no pending changes" plus the new-version banner |
| `display-quota.txt` | Deleted, Remaining, State, Total, Used with bytes in brackets |
| `display-config.txt` | Every `Config option '<key>' = <value>` line |
| `monitor-upload-delete.txt` | Start-up block, initial sync, a new local file uploaded, the same file deleted, a WebSocket signal, clean shutdown |
| `monitor-auth-expired.txt` | Start-up block ending in the AADSTS50078 error and the `--reauth` instruction |

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
| `Account Type:          <business|personal>` | Account line |
| `ERROR: You will need to issue a --reauth ...` | Blocked |
| `Received termination signal, attempting to cleanly shutdown application` | Disabled follows when the process exits |
| `INFO: A new onedrive client version is available.` | Ignore |

Still to capture: a download, a modified-file upload, a failed upload (`Failed to upload`), the `--resync is required` prompt, and a non-verbose run.

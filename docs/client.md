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

The redirect URI is Microsoft's fixed `nativeclient` page, so there is no loopback or custom scheme to catch. `--auth-files` is the clean path: the client writes the sign-in URL to the first file and waits for the response URL in the second. oxdrive opens the URL in the browser and offers a paste box for the response.

## Output to parse

To be filled from fixtures captured on this machine (`crates/core/tests/fixtures/`, secrets scrubbed). Known line families, from the client's source and from what OneDriveGUI matches:

- `Processing <n> OneDrive items ...` while scanning
- `Uploading file ... <path> ... done` / `Downloading file ...` / `Deleting ...`
- `<Uploading|Downloading>: <name> ... <n>%` progress
- `Free Space: <n> bytes` and the account type line after `--display-quota`
- `Failed to upload|download: <path>` and `Failed items ...: <n>`
- The multi-line error blocks and the "authorise this application" and "--resync is required" prompts

Each family becomes one fixture file and one test before any parser code.

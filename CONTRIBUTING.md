# Contributing community sources

Community sources run on other people's PCs. Treat every script as code you would install on your own machine.

## How to add a source

1. Open one PR per source.
2. Add `community/<name>/` with exactly these three files:
   - `pulse-<name>` — executable bash or python3 script that prints [Pulse output](docs/GUIDE.md#5-what-a-source-can-print) (a number, text, or JSON `items` / `rows` / `value`)
   - `source.toml` — a `[[source]]` block to paste into `~/.config/omarchy-pulse/pulse.toml`
   - `README.md` — what it shows, packages/tokens/permissions, install steps (same wording as the other READMEs), optional screenshot with no personal data
3. License: MIT, same as the rest of this repository.

## Security rules

- **No `curl | sh`**, no download-and-execute of remote code, **no `sudo`**.
- Contact **only** the service declared in the README. **No telemetry**, no extra hosts.
- **No secrets in the files.** Tokens and passwords come from `source.toml` or environment variables, never hard-coded in the script.
- **Read-only.** A source shows data; it must not create, change or delete anything on the service.
- Output must match the Pulse formats in [docs/GUIDE.md §5](docs/GUIDE.md#5-what-a-source-can-print). Use reasonable timeouts. No personal data in examples or screenshots.
- One source per PR.

## Install wording (copy into every README)

```bash
cp community/<name>/pulse-<name> ~/.config/omarchy-pulse/sources/
chmod +x ~/.config/omarchy-pulse/sources/pulse-<name>
```

Then paste `source.toml` into `~/.config/omarchy-pulse/pulse.toml` (tokens and addresses go there or in environment variables, never inside the script).

## Checks

The `community-sources` GitHub Action on every PR verifies the three files, executable bit, `bash -n` / Python AST parse, and valid TOML. Run the same locally before you open the PR.

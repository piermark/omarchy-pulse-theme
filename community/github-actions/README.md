# github-actions

Last GitHub Actions runs of a repository as status tiles (success / failure / in progress).

## Needs

- `gh` (authenticated: `gh auth login`), `jq`
- `GITHUB_REPO` — `owner/name`

## Config

| Variable | Required | Description |
|---|---|---|
| `GITHUB_REPO` | yes | Repository `owner/name` |
| `GITHUB_ACTIONS_LIMIT` | no | How many runs (default 6) |

## Install

```bash
cp community/github-actions/pulse-github-actions ~/.config/omarchy-pulse/sources/
chmod +x ~/.config/omarchy-pulse/sources/pulse-github-actions
```

Then paste `source.toml` into `~/.config/omarchy-pulse/pulse.toml` (tokens and addresses go there or in environment variables, never inside the script).

## Notes

Read-only: `gh run list` only. Contacts only `api.github.com` (via `gh`). No token in the script — uses your local `gh` login.

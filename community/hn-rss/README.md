# hn-rss

Hacker News frontpage headlines for the ticker (or a list panel).

## Needs

- `curl`, `python3`
- Outbound HTTPS to `hnrss.org` (or another HN RSS URL you set)

## Config

| Variable | Required | Description |
|---|---|---|
| `HN_RSS_URL` | no | RSS URL (default `https://hnrss.org/frontpage`) |
| `HN_LIMIT` | no | Max titles (default 8) |

## Install

```bash
cp community/hn-rss/pulse-hn-rss ~/.config/omarchy-pulse/sources/
chmod +x ~/.config/omarchy-pulse/sources/pulse-hn-rss
```

Then paste `source.toml` into `~/.config/omarchy-pulse/pulse.toml` (tokens and addresses go there or in environment variables, never inside the script).

## Notes

Read-only GET of a public RSS feed. No account, no token. Contacts only the URL you configure.

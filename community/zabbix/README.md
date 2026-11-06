# zabbix

Current Zabbix problems as green/amber/red status tiles (severity ≥ 4 = crit, ≥ 2 = warn).

## Needs

- `curl`, `jq`
- Zabbix API token (*User settings → API tokens*)
- Network reachability to your Zabbix server

## Config

| Variable | Required | Description |
|---|---|---|
| `ZABBIX_URL` | yes | Base URL or full `…/api_jsonrpc.php` |
| `ZABBIX_TOKEN` | yes | API bearer token |
| `ZABBIX_LIMIT` | no | Max problems (default 8) |

## Install

```bash
cp community/zabbix/pulse-zabbix ~/.config/omarchy-pulse/sources/
chmod +x ~/.config/omarchy-pulse/sources/pulse-zabbix
```

Then paste `source.toml` into `~/.config/omarchy-pulse/pulse.toml` (tokens and addresses go there or in environment variables, never inside the script).

## Notes

Read-only: calls `problem.get` only. Contacts only the host in `ZABBIX_URL`.

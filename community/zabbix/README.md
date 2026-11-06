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

Then paste `source.toml` into `~/.config/omarchy-pulse/pulse.toml`.

Pulse runs as a systemd user service and does **not** see your terminal environment. Set the variables in the command line of `source.toml` (see the active `command =` there). Environment variables only work if you set them for the systemd user service (for example with `Environment=` / `EnvironmentFile=` in the unit).

## Notes

Read-only: calls `problem.get` only. Contacts only the host in `ZABBIX_URL`.

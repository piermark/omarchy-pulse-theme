# Community sources for Omarchy Pulse

Drop-in scripts that turn any service into a Pulse panel. Each folder is one source: copy the script, paste the TOML, done.

## Sources

| Name | Shows | Needs |
|---|---|---|
| [zabbix](zabbix/) | Current Zabbix problems as status tiles | `curl`, `jq`, Zabbix API token |
| [proxmox](proxmox/) | Proxmox VMs/CTs as status tiles | `ssh`, `jq`, SSH to a node with `pvesh` |
| [github-actions](github-actions/) | Recent GitHub Actions runs as status tiles | `gh`, `jq`, `gh auth login` |
| [hn-rss](hn-rss/) | Hacker News frontpage headlines (ticker) | `curl`, `python3` |
| [docker](docker/) | Running containers as a name → status table | `docker`, `python3` |

## Install (any source)

```bash
cp community/<name>/pulse-<name> ~/.config/omarchy-pulse/sources/
chmod +x ~/.config/omarchy-pulse/sources/pulse-<name>
```

Then paste that folder's `source.toml` into `~/.config/omarchy-pulse/pulse.toml` (tokens and addresses go there or in environment variables, never inside the script).

## Wanted

Sources we would love as PRs (see [CONTRIBUTING.md](../CONTRIBUTING.md)):

- UniFi
- Pi-hole
- OctoPrint / Klipper
- Uptime Kuma
- Grafana
- TrueNAS
- Synology
- Wazuh
- Jellyfin
- Tailscale

One source per PR. Keep it read-only, no secrets in the files, MIT like the rest of the repo.

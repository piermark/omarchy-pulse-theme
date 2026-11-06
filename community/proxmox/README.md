# proxmox

Proxmox VMs and containers as status tiles (`running` = ok, otherwise warn).

## Needs

- `ssh`, `jq`
- Passwordless SSH to a node that can run `pvesh` (usually root@pve or a user with PVEAuditor)
- `PROXMOX_SSH` — SSH target, e.g. `root@pve` or `pve`

## Config

| Variable | Required | Description |
|---|---|---|
| `PROXMOX_SSH` | yes | SSH target that can run `pvesh` |

## Install

```bash
cp community/proxmox/pulse-proxmox ~/.config/omarchy-pulse/sources/
chmod +x ~/.config/omarchy-pulse/sources/pulse-proxmox
```

Then paste `source.toml` into `~/.config/omarchy-pulse/pulse.toml` (tokens and addresses go there or in environment variables, never inside the script).

## Notes

Read-only: `pvesh get /cluster/resources --type vm`. Contacts only the SSH host you set. Prefer an SSH key with no passphrase (or an agent) so the source never prompts.

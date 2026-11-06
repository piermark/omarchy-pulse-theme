# docker

Running Docker containers as a two-column table (name → status).

## Needs

- `docker` CLI, permission to talk to the local Docker socket (usually your user in the `docker` group)
- `python3`

## Config

| Variable | Required | Description |
|---|---|---|
| `DOCKER_PS_FILTER` | no | Passed to `docker ps --filter` (e.g. `status=running`) |

## Install

```bash
cp community/docker/pulse-docker ~/.config/omarchy-pulse/sources/
chmod +x ~/.config/omarchy-pulse/sources/pulse-docker
```

Then paste `source.toml` into `~/.config/omarchy-pulse/pulse.toml` (tokens and addresses go there or in environment variables, never inside the script).

## Notes

Read-only: `docker ps` only (no start/stop). Contacts only the local Docker daemon. No secrets in the script.

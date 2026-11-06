## Community source PR

- [ ] One source only (`community/<name>/`)
- [ ] Contains `pulse-<name>`, `source.toml`, and `README.md`
- [ ] Script is executable (`chmod +x`)
- [ ] No `curl | sh`, no download-and-execute, no `sudo`
- [ ] Contacts only the service declared in the README (no telemetry)
- [ ] No secrets in the files (tokens/passwords from `source.toml` or env vars)
- [ ] Read-only: shows data, does not change the service
- [ ] Output is a Pulse format (number / text / JSON `items` | `rows` | `value`)
- [ ] Reasonable timeouts; no personal data in examples or screenshots
- [ ] MIT license, same as the repo
- [ ] How I tested it: …

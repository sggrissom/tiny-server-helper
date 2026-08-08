# tiny-server-helper

Bash toolkit for running small self-contained binaries on a VPS. Capistrano-style
releases with atomic symlink switching, health checks, and auto-rollback.
Assumes systemd + Caddy on the server.

Apps must listen on `$PORT` and expose `/healthz`.

## Layout

```
bin/appctl          server-side app management
bin/bootstrap-vps   one-time server setup
bin/deploy          local deploy script
bin/monitor         one-off / watch health check
systemd/            app@ and internal@ unit templates
monitor-tui/        terminal dashboard for all sites
metrics-server/     internal service exposing host metrics
```

On the server, each app lives at:

```
/srv/apps/<app>/
├── releases/           # YYYY-MM-DD_HHMMSS_<sha>
├── shared/.env         # PORT + secrets
├── shared/data/        # persistent data
└── current -> releases/...
```

## One-time server setup

```bash
sudo ./bin/bootstrap-vps
```

Creates the `apps` user, `/srv/apps/`, installs the systemd templates, and points
Caddy at `/etc/caddy/sites/*.caddy`.

## Adding an app

On the VPS:

```bash
sudo appctl init myapp 8670          # dirs + shared/.env with PORT
sudo nano /srv/apps/myapp/shared/.env  # add secrets
sudo appctl domain myapp myapp.com   # Caddy vhost + TLS
```

From your machine:

```bash
./bin/deploy myapp vps ./build/myapp
```

Then start it:

```bash
sudo systemctl enable --now app@myapp
curl https://myapp.com/healthz
```

## Deploying

```bash
deploy <app> <user@host> <binary> [service_type]
```

`service_type` is `app` (default, public via Caddy) or `internal` (no vhost).
Symlink `bin/deploy` into `~/.local/bin/` so project Makefiles can call it.

Each deploy uploads to a new timestamped release, flips `current`, restarts the
unit, health-checks `http://127.0.0.1:$PORT/healthz`, rolls back on failure, and
prunes to the last 5 releases.

## Server commands

```bash
appctl status                  # all apps: running state + current release
appctl logs <app>              # journalctl -f
sudo appctl rollback <app>     # previous release
sudo appctl domain <app> <fqdn>
```

## Monitoring

Quick check from anywhere:

```bash
./bin/monitor https://myapp.com/healthz     # once
./bin/monitor https://myapp.com/healthz 10  # watch every 10s
```

TUI dashboard — copy `monitor-tui/sites.toml.example` to `sites.toml`,
`~/.config/monitor/sites.toml`, or `/etc/monitor/sites.toml`, then:

```bash
cargo run -p monitor-tui
```

It polls each `[[sites]]` URL, tracks history, and sends terminal-bell/desktop
alerts on status transitions. Add a `[server_metrics]` section pointing at a
deployed `metrics-server` to also show host CPU/memory/disk and internal service
state.

## metrics-server

Internal-only service (`/healthz`, `/metrics`), deployed as `internal@`:

```bash
make deploy    # cargo build --release + ./bin/deploy metrics-server vps ... internal
```

Reads `PORT` and optional `API_KEY` from `shared/.env`; set `API_KEY` to keep it
non-public.

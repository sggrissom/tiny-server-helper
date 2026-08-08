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
bin/mailctl         server-side outbound mail (Postfix + DKIM)
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

## Sending mail

Outbound SMTP is a server capability, not something each site carries. One
Postfix instance listens on `127.0.0.1:25` only, OpenDKIM signs with a separate
key per registered domain, and apps need to know nothing beyond an SMTP host,
a port, and their own From address. No relay service, no per-repo config.

Domains opt in explicitly — deploying a site does not enable mail for it.

Debian/Ubuntu only. `mailctl` never installs packages; like `bootstrap-vps`
does for Caddy, it prints the `apt-get` line and exits.

### Requirements you have to arrange yourself

```bash
mailctl doctor    # checks all of these, run it first
```

- **Outbound TCP 25 must be open.** Most VPS providers block it by default and
  will unblock on request; Google Cloud never does. Nothing else matters until
  this passes.
- **A static public IP**, since SPF records hardcode it.
- **Reverse DNS (PTR)** for that IP, set at the VPS provider console — not the
  registrar — pointing at your mail hostname, with a matching A record.
- **SPF, DKIM, and DMARC records** at your DNS provider. `mailctl` generates
  them; publishing them is manual.

### Setup

```bash
sudo mailctl setup mail.example.com     # one Postfix + OpenDKIM, one hostname
sudo mailctl add example.com            # per-domain key, prints DNS to publish
# ...publish the records it printed...
sudo mailctl verify example.com         # confirms DNS, then turns signing on
sudo mailctl test example.com you@gmail.com
```

`add` stages a domain: the key exists but nothing signs with it until `verify`
confirms the published DKIM record matches the private key. That way a domain
never signs mail against DNS that isn't live yet.

Subdomains ride on the parent's key — registering `example.com` also signs
`app.example.com` with `d=example.com`, which passes DMARC under relaxed
alignment. Register subdomains separately only if you need strict alignment.

### Day to day

```bash
sudo mailctl status                     # domains, selectors, DNS state, queue
sudo mailctl dns example.com            # reprint the records to publish
sudo mailctl rotate example.com 202702  # new key alongside the old
sudo mailctl activate example.com 202702  # switch, once the new record is live
sudo mailctl remove example.com
```

Rotation is two commands on purpose: `rotate` never changes what is being
signed, and `activate` refuses unless the new key is provably published.

### What an app needs to know

`SMTP_HOST` and `SMTP_PORT` arrive automatically. `mailctl setup` writes
`/etc/tiny-server-helper/mail.env` and a systemd drop-in that every `app@` and
`internal@` unit reads, so there is nothing to add to a site's repo:

```
SMTP_HOST=127.0.0.1
SMTP_PORT=25
```

Set the From address per app, in its own env file:

```bash
sudo nano /srv/apps/myapp/shared/.env    # MAIL_FROM=noreply@example.com
sudo systemctl restart app@myapp
```

Connect with no authentication and no TLS — the socket is loopback-only, which
is what keeps it from being an open relay. Submission ports 465 and 587 are
deliberately never enabled.

Mail configuration is entirely under `/etc` and survives redeploys untouched.
`deploy` needs no changes and no mail-related steps, and `EnvironmentFile` is
re-read on every start, so the restart a deploy already does picks up any
change. **Private DKIM keys live only in `/etc/opendkim/keys/` on the server
and never enter git.** There is no export path; if you lose them, rotate.

If OpenDKIM is down, Postfix returns `451 4.7.1` and the app's send fails
loudly rather than mail leaving unsigned and quietly eroding the sending
reputation shared by every domain on the box.

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

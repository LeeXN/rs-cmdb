# rs-cmdb

[English](README.md) | [中文](README_CN.md)

**rs-cmdb** is a lightweight Configuration Management Database (CMDB) system built entirely in Rust.

## 🚀 Features

- **Full Stack Rust**: Built with Rust from backend to UI, ensuring memory safety and high performance.
- **Automated Discovery**: Cross-platform agents (`rs-cmdb-client`) automatically collect hardware specifications (CPU, RAM, Disk, Network, GPU, IPMI/BMC) and report to the server.
- **Asset Management**: Detailed hardware inventory, component stock tracking, rack/data-center visualization, and project/person relationship management.
- **Efficient Change History**: Stores only hardware deltas (not full snapshots), with built-in CLI for analysis, cleanup, and migration.
- **Remote Command Execution**: Admin-controlled remote exec with three-layer security (enable check → exec policy → danger detection), SSE log streaming, and an approval workflow.
- **Permission System**: Three-role RBAC (Admin/User/Viewer) with resource-level ownership enforcement, data-scope filtering, exec policies, and web terminal policies.
- **Audit Logging**: All security-relevant operations are recorded with operator identity.
- **Modern Dashboard**: Real-time analytics, resource usage statistics, and health monitoring.
- **Zero-Dependency Database**: Uses `Redb`, an embedded key-value store — no PostgreSQL or MySQL required.
- **Internationalization**: Native support for English and Simplified Chinese.

## 📺 Demo

- **URL**: http://138.2.83.32:8080/
- **Username**: `demo`
- **Password**: `demo@2025.com`

---

## ✨ What's New

### Permission System & Remote Command Execution

This release introduces a complete permission system and a secure remote command execution framework.

#### Permission System

A three-role RBAC (Admin / User / Viewer) model now applies at three levels:

| Level | What it controls |
|---|---|
| Route-level | Which API endpoints each role can call |
| Ownership-level | Users can only modify resources they created; Admin bypasses |
| Data-scope | List endpoints automatically filter results based on the caller's scope |

**Resource ownership** is tracked on every entity via a `created_by` field. Non-Admin users that attempt to update or delete a resource they did not create receive `403 Forbidden`.

**Data-scope filtering** is injected via `PermissionMiddleware`. By default:
- Admin → sees all records
- User → sees only own records
- Viewer → sees all records (read-only)

Custom `PermissionRule`s (stored in Redb, cached in memory) allow overriding the default scope per subject per resource type.

**Exec Policies** add a second layer of control for remote command execution: allowlist/blocklist command patterns, per-target-scope restrictions, and an optional `require_approval` gate.

**Web Terminal Policies** control terminal session behavior: ReadOnly / ReadWrite mode, command whitelist for ReadOnly sessions, idle timeout, and concurrent session limits.

**Approval Workflow**: when a policy has `require_approval: true`, submitting a command creates a `PendingApproval` record instead of executing immediately. Admins approve or reject; requests auto-expire every 5 minutes via a cron job.

**New API endpoints:**

```
# Permission rules (Admin only)
GET/POST   /api/v1/permissions/rules
GET/PUT/DELETE /api/v1/permissions/rules/{id}

# Exec policies (Admin only)
GET/POST   /api/v1/permissions/exec-policies
GET/PUT/DELETE /api/v1/permissions/exec-policies/{id}

# Web terminal policies (Admin only)
GET/POST   /api/v1/permissions/web-terminal-policies
GET/PUT/DELETE /api/v1/permissions/web-terminal-policies/{id}

# Approval workflow
GET    /api/v1/permissions/pending-approvals        (Admin: list all)
GET    /api/v1/permissions/pending-approvals/{id}   (Admin)
POST   /api/v1/permissions/pending-approvals/{id}/approve
POST   /api/v1/permissions/pending-approvals/{id}/reject
GET    /api/v1/permissions/my-approvals             (any authenticated user)
```

#### Remote Command Execution

Admins can execute commands on registered clients. The system uses a **three-layer security model**:

1. **Layer 1 — Feature gate**: remote exec must be explicitly enabled (`PUT /api/v1/remote-exec/config`)
2. **Layer 2 — Exec Policy**: `ExecPolicyEngine` evaluates matching policies (highest priority first) → Allow / Deny / Warning / RequireApproval
3. **Layer 3 — Danger Detection**: built-in rules classify commands as Safe / Warning / Blocked (e.g. `rm -rf /`, `mkfs`, `shutdown` are always blocked regardless of policies)

Commands that pass all three layers are queued as `CommandTask`s, dispatched to the agent, and streamed back via SSE.

**New API endpoints:**

```
GET  /api/v1/remote-exec/config                     Read current config
PUT  /api/v1/remote-exec/config                     Enable/disable (Admin)
POST /api/v1/remote-exec/commands                   Submit a command (Admin)
GET  /api/v1/remote-exec/commands                   List commands
GET  /api/v1/remote-exec/commands/{id}              Get command detail
GET  /api/v1/remote-exec/commands/{id}/logs         Get buffered logs
GET  /api/v1/remote-exec/commands/{id}/stream       SSE real-time log stream

# Agent-side (agent token auth)
GET  /api/v1/agent/commands/pending
POST /api/v1/agent/commands/{id}/start
POST /api/v1/agent/commands/{id}/logs
POST /api/v1/agent/commands/{id}/complete
```

#### Audit Log

All security-relevant operations are written to the audit log with the real operator username:

| Action | Trigger |
|---|---|
| `user_created` | Admin registers a new user |
| `user_deleted` | Admin deletes a user |
| `user_role_changed` | Admin changes a user's role |
| `config_change` | Admin enables/disables remote exec |
| `command_create` | Command queued |
| `command_blocked` | Danger detection blocks a command |
| `command_completed` | Command execution finishes |
| `client_created` | Client registered via import |
| `component_created` | Component created |

#### Other improvements in this release

- **API error messages sanitized**: internal database/server details no longer leak to API consumers; full errors are logged server-side only.
- **Batch operation limits**: `import_clients` and `batch_create_components` now return `413 Payload Too Large` if the payload exceeds `max_batch_size` (default 1000).
- **Search parameter length validation**: search/filter string params are limited to 256 characters.
- **`expose_version` config switch**: set `expose_version = false` to hide version info from the public `/api/v1/version` endpoint.
- **Docker non-root user**: the container now runs as a dedicated `cmdb` system user.

---

## ⚡ Quick Start

```bash
mkdir -p /opt/rs-cmdb
cd /opt/rs-cmdb
export RSCMDB_VERSION="0.0.1"

mkdir -p binaires/linux/{x86_64,aarch64}
curl -L -o ./binaires/linux/x86_64/rs-cmdb-client \
  https://github.com/LeeXN/rs-cmdb/releases/download/${RSCMDB_VERSION}/rs-cmdb-client-x86_64-linux-musl
curl -L -o ./binaires/linux/aarch64/rs-cmdb-client \
  https://github.com/LeeXN/rs-cmdb/releases/download/${RSCMDB_VERSION}/rs-cmdb-client-aarch64-linux-musl
chmod +x ./binaires/linux/x86_64/rs-cmdb-client
chmod +x ./binaires/linux/aarch64/rs-cmdb-client

docker run -itd \
  --name rs-cmdb \
  -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/binaires:/app/binaires \
  -e CMDB_JWT_SECRET="$(openssl rand -base64 32)" \
  -e CMDB_ADMIN_PASSWORD="YourSecureP@ssword123" \
  leex2019/rs-cmdb:${RSCMDB_VERSION}
```

Access the UI at `http://localhost:8080`. Default admin username is `admin`.

## 🏗️ Architecture

```
rs-cmdb-client (Linux agent)
    └── HTTP JSON push → server

rs-cmdb-server (Axum, :8080)
    ├── API → Middleware (Auth / RBAC / Permission / RateLimit)
    │         → Service → Repository → Redb (embedded KV)
    ├── Message Queue (Flume) — async hardware processing & audit
    ├── SSE Hub — real-time command log streaming
    └── Scheduler (tokio-cron) — approval expiry, cleanup jobs

rs-cmdb-front (Yew WASM SPA)
    └── served as static files by the server
```

**Cargo workspace crates:**

| Crate | Description |
|---|---|
| `server/` | Backend API server (Axum) |
| `client/` | Hardware collection agent |
| `front/` | Yew WASM frontend |
| `common/` | Shared models, errors, entities |

## ⚙️ Configuration

The server reads `config/default.toml` and any environment variable with the `CMDB_` prefix.

### Security (required)

```bash
# JWT secret — must be at least 32 characters; server refuses to start without it
export CMDB_JWT_SECRET=$(openssl rand -base64 32)

# Initial admin password — used on first startup to create the admin account
export CMDB_ADMIN_PASSWORD="YourSecureP@ssword123"
```

Password requirements: ≥12 chars, uppercase, lowercase, digit, special character.

### `config/default.toml`

```toml
host = "0.0.0.0"
port = 8080
log_level = "info"

# Must be overridden via CMDB_JWT_SECRET
jwt_secret = "change_me_in_production"

# Batch import/create limit (returns 413 if exceeded)
max_batch_size = 1000

# Set false to hide version info from the /api/v1/version endpoint
expose_version = true

poll_interval = 300          # seconds between client heartbeats
client_timeout = 3600        # seconds before client is marked offline
component_missing_grace_period_hours = 24

ssh_known_hosts_file = "/etc/cmdb/ssh_known_hosts"

enable_tls = false
# tls_cert = "path/to/cert.pem"
# tls_key = "path/to/key.pem"

[database]
path = "data/cmdb.redb"

# Primary IP auto-detection (optional)
[primary_ip]
# subnet = "10.0.0.0/8"

[queue]
capacity = 1000
```

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `CMDB_JWT_SECRET` | — | **Required**, ≥32 chars |
| `CMDB_ADMIN_PASSWORD` | — | **Required on first startup** |
| `CMDB_HOST` | `0.0.0.0` | Bind address |
| `CMDB_PORT` | `8080` | Server port |
| `CMDB_LOG_LEVEL` | `info` | `debug` / `info` / `warn` / `error` |
| `CMDB_DATABASE__PATH` | `data/cmdb.redb` | Redb file path |
| `CMDB_PRIMARY_IP__SUBNET` | — | CIDR for primary IP auto-detection |
| `CMDB_MAX_BATCH_SIZE` | `1000` | Max items per batch import/create |
| `CMDB_EXPOSE_VERSION` | `true` | Expose version via `/api/v1/version` |
| `CMDB_SSH_KNOWN_HOSTS_FILE` | `/etc/cmdb/ssh_known_hosts` | SSH known hosts path |

## 🛠️ Build & Run

### Makefile (recommended)

```bash
make build          # Build all crates (glibc)
make build-musl     # Build fully static musl binaries
make test           # Run all tests
make docker         # Build Docker image
make clean          # Remove build artifacts
make help           # Show all targets
```

### Docker

```bash
docker build -t rs-cmdb .
docker run -p 8080:8080 \
  -v /path/to/data:/app/data \
  -e CMDB_JWT_SECRET="$(openssl rand -base64 32)" \
  -e CMDB_ADMIN_PASSWORD="YourSecureP@ssword123" \
  rs-cmdb
```

### Manual

**Prerequisites:** Rust (stable), [Trunk](https://trunkrs.dev/), Node.js + npm

```bash
# Frontend (WebAssembly)
cd front && npm install && trunk build --release

# Server
cargo build --release --package server

# Agent
cargo build --release --package client
```

## 💻 Client Standalone Usage

Run the agent as a standalone hardware info tool (no server required):

```bash
./rs-cmdb-client
```

Prints a full system report: CPU, OS, RAM, Disk, Network, GPU, IPMI/BMC.

## 📦 Deployment

**Directory structure:**

```text
/opt/rs-cmdb
  ├── rs-cmdb-server
  ├── dist/              # Frontend static files
  │   └── index.html
  ├── config/
  │   └── default.toml
  └── data/              # Redb database (auto-created)
```

```bash
cd /opt/rs-cmdb
export CMDB_JWT_SECRET=$(openssl rand -base64 32)
export CMDB_ADMIN_PASSWORD="YourSecureP@ssword123"
./rs-cmdb-server
```

First login: username `admin`, password as set above.

### SSH Known Hosts

Required for remote service management:

```bash
sudo mkdir -p /etc/cmdb
ssh-keyscan -H 192.168.1.100 >> /etc/cmdb/ssh_known_hosts
sudo chmod 644 /etc/cmdb/ssh_known_hosts
```

---

## 📌 Primary IP

Each client record has a `primary_ip` field displayed preferentially over `ip_address`. It is populated via the following priority chain:

1. **Manual override** — `PUT /api/v1/clients/{id}/primary-ip` or the frontend edit form
2. **Agent auto-detection** — if `[primary_ip] subnet = "10.0.0.0/8"` is set in `client.toml`, the agent matches its NIC IPv4 on registration
3. **Server auto-detection** — if `[primary_ip] subnet` is set in `config/default.toml`, the server auto-detects on each hardware push
4. **Fallback** — `None`; UI shows `ip_address`

No database migration needed — the field is `Option<String>` and defaults to `None` on all existing records.

**Upgrade compatibility matrix:**

| Server | Client | Behavior |
|---|---|---|
| New | New | Full support: agent detects on registration + server auto-detects on hardware push |
| New | Old | Server auto-detects from NICs on hardware push (if subnet configured) |
| Old | New | New client sends `primary_ip`; old server ignores the unknown field (no `deny_unknown_fields`) |

## 🔧 History Maintenance (CLI)

```
rs-cmdb-server history <COMMAND>

Commands:
  analyze   Show per-client snapshot counts and age
  cleanup   Remove old entries, keep newest N per client
  migrate   Convert old full-snapshot entries to delta format
  compact   Rewrite DB to reclaim disk space
```

```bash
# Preview (keep 50 newest per client)
rs-cmdb-server history cleanup --keep-last 50 --dry-run --db-path data/cmdb.redb

# Execute
rs-cmdb-server history cleanup --keep-last 50 --db-path data/cmdb.redb

# Compact after cleanup
rs-cmdb-server history compact --db-path data/cmdb.redb
```

Example: 121,298 entries / 4.1 GB → 24,814 entries / 26 MB after a full migrate+cleanup+compact cycle.

---

## 📋 Changelog

### Permission System & Remote Command Execution

- **Permission system** — Three-level RBAC (route / ownership / data-scope). Resource ownership tracked via `created_by` on all entities. `PermissionMiddleware` injects `PermissionContext` for automatic list-endpoint filtering. Custom `PermissionRule`s configurable via API.
- **Exec policies** — Fine-grained command execution controls: allowlist/blocklist patterns, per-scope restrictions, `require_approval` gate.
- **Web terminal policies** — Per-subject session controls: ReadOnly/ReadWrite mode, command whitelist, idle timeout, concurrent session limits.
- **Approval workflow** — `PendingApproval` entities with approve/reject/auto-expire (5-minute cron). Operators see their own approvals; Admins manage all.
- **Remote command execution** — Three-layer security (feature gate → exec policy → danger detection). Agent polling + SSE log streaming.
- **Audit log** — Structured `AuditLogEntry` with `AuditAction` enum; operator username recorded on all security events.
- **API hardening** — Error messages sanitized (no internal details to clients), batch size limits (413), search param length validation (256 chars), `expose_version` switch, Docker non-root user.

### Primary IP & Hardware History Delta

- **Primary IP field** — `primary_ip: Option<String>` on `Client`; sourced from manual override, agent detection, or server auto-detection via CIDR subnet config.
- **Delta hardware history** — History stores only changed fields instead of full snapshots, dramatically reducing storage. Built-in CLI commands: `analyze`, `cleanup`, `migrate`, `compact`.
- **Export filtered clients** — `POST /api/v1/clients/export_filtered` with hardware-spec filters; returns CSV/JSON.

## 📄 License

This project is licensed under the [MIT license](LICENSE).

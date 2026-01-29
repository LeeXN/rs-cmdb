# rs-cmdb

[English](README.md) | [中文](README_CN.md)

**rs-cmdb** is a lightweight Configuration Management Database (CMDB) system built entirely in Rust.

## 🚀 Features

*   **Full Stack Rust**: Built with Rust from the kernel to the UI, ensuring memory safety and high performance.
*   **Automated Discovery**: Cross-platform agents (`rs-cmdb-client`) automatically collect hardware specifications (CPU, RAM, Disk, Network) and report to the server.
*   **Asset Management**:
    *   Detailed hardware inventory tracking.
    *   Change history logging (track hardware modifications over time).
    *   Rack and data center visualization.
*   **Modern Dashboard**: Real-time analytics, resource usage statistics, and health monitoring.
*   **Security**: Role-Based Access Control (RBAC) and secure API authentication.
*   **Zero-Dependency Database**: Uses `Redb`, an embedded key-value store, eliminating the need for external database setup (like PostgreSQL or MySQL).
*   **Internationalization**: Native support for English and Simplified Chinese.

## 📺 Demo

Experience the live demo:
*   **URL**: http://138.2.83.32:8080/
*   **Username**: `demo`
*   **Password**: `demo@2025.com`

## ⚡ Quick Start

You can quickly start the rs-cmdb server using Docker.

```bash
# 1. Create a directory for the project
mkdir -p /opt/rs-cmdb
cd /opt/rs-cmdb

# 2. Set the version
export RSCMDB_VERSION="0.0.1"

# 3. Prepare directories for client binaries (optional, for auto-update/download from server)
mkdir -p binaires/linux/{x86_64,aarch64}

# 4. Download client binaries
# You can download them from the GitHub Release page or build them yourself.
# https://github.com/LeeXN/rs-cmdb/releases
curl -L -o ./binaires/linux/x86_64/rs-cmdb-client https://github.com/LeeXN/rs-cmdb/releases/download/${RSCMDB_VERSION}/rs-cmdb-client-x86_64-linux-musl
curl -L -o ./binaires/linux/aarch64/rs-cmdb-client https://github.com/LeeXN/rs-cmdb/releases/download/${RSCMDB_VERSION}/rs-cmdb-client-aarch64-linux-musl

# 5. Make binaries executable
chmod +x ./binaires/linux/x86_64/rs-cmdb-client
chmod +x ./binaires/linux/aarch64/rs-cmdb-client

# 6. Run the server using Docker
docker run -itd \
  --name rs-cmdb \
  -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/binaires:/app/binaires \
  leex2019/rs-cmdb:${RSCMDB_VERSION}
```

After starting, access the UI at `http://localhost:8080`.

**⚠️ Important:** The server requires secure environment variables to be set for first-time initialization. See the [Configuration](#-configuration) section below.

## �� Architecture

The project follows a monorepo structure:

*   **`server/`**: The backend API server built with **Axum**. It handles API requests, manages the **Redb** database, and serves the frontend static files.
*   **`front/`**: The single-page application (SPA) frontend built with **Yew** (WebAssembly) and **TailwindCSS**.
*   **`client/`**: The lightweight agent that runs on target machines to collect system information.
*   **`common/`**: Shared Rust crates containing data models and utility functions used by all components.

## 🛠️ Build & Run

### Using Makefile (Recommended)

We provide a `Makefile` to simplify the build and test process.

*   **Build Everything**: `make build` (Builds Server, Client, and Frontend)
*   **Run Tests**: `make test`
*   **Build Docker Image**: `make docker`
*   **Clean Artifacts**: `make clean`
*   **Show Help**: `make help`

### Using Docker

You can build and run the entire system using Docker.

```bash
# Build the image
docker build -t rs-cmdb .

# Run the container
docker run -p 8080:8080 -v /path/to/data:/app/data rs-cmdb
```

### Manual Build

#### Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install) (latest stable)
*   [Trunk](https://trunkrs.dev/) (for building the frontend): `cargo install trunk`
*   Node.js & npm (for TailwindCSS)

#### 1. Build Frontend

The frontend compiles to WebAssembly.

```bash
cd front
npm install
trunk build --release
```

The build artifacts will be generated in `front/dist`.

#### 2. Build Server

```bash
cargo build --release --package server
```

The binary will be located at `target/release/rs-cmdb-server`.

#### 3. Build Client (Agent)

```bash
cargo build --release --package client
```

The binary will be located at `target/release/rs-cmdb-client`.

## 💻 Client Standalone Usage

The `rs-cmdb-client` can run as a standalone tool to collect and display detailed hardware information directly in the terminal, in addition to reporting to the server.

**How to Run:**

Execute the compiled binary directly:

```bash
./rs-cmdb-client
```

**Sample Output:**

```text
================================================================================
                               System Information                               
================================================================================
System Vendor:          Dell
System Product Name:    Dell PowerEdge R740
System Serial Number:   SN-EXAMPLE-01
System Product Version:  

================================================================================
                          Operating System Information                          
================================================================================
System Name:    Rocky Linux
System Version: 8.8
Kernel Version: 4.18.0-477.10.1.el8_8.x86_64
Architecture:   x86_64
Hostname:       node-01-example
IP Address:     192.168.1.100
DNS Servers:    8.8.8.8

================================================================================
                                CPU Information                                 
================================================================================
Vendor ID:      GenuineIntel
Model Name:     Intel(R) Xeon(R) Silver 4410Y
CPU Count:      2
Core Count:     24
Thread Count:   48
CPU Speed:      3900 MHz
...

================================================================================
                                Disk Information                                
================================================================================
Vendor          Model           Capacity     Type                 Firmware       
--------------------------------------------------------------------------------
ATA             INTEL SSDSC2KB96 894.3      GB SSD                  0120           
Unknown         Unknown         0          B  HDD                  Unknown        
ATA             INTEL SSDSCKKB48 447.1      GB SSD                  0120           
...

================================================================================
                         Network Interface Information                          
================================================================================
Interface Name:   ens14f0
Vendor:           Intel Corporation(0x8086)
Model:            I350 Gigabit Network Connection(0x1521)
MAC Address:      00:11:22:33:44:55
PCI Slot:         0000:99:00.0
Driver:           igb
NIC Type:         Ethernet
Link Status:      Up
Link Speed:       1000 Mbps
...

================================================================================
                                GPU Information                                 
================================================================================
Vendor:         NVIDIA
Model:          AD102GL [L20]
Device ID:      0000:63:00.0
Serial Number:  GPU-SN-001
Driver Version: 550.163.01
...

================================================================================
                                RAM Information                                 
================================================================================
Total Memory:    256 GB (8 modules)
Memory Type:     DDR5
Vendor:          Samsung
Speed:           4800 MHz
================================================================================

================================================================================
                              IPMI/BMC Information                              
================================================================================
IPMI Status:    Available
IP Address:     10.0.0.10
MAC Address:    b0:31:a6:4f:d6:57
Subnet Mask:    255.255.254.0
Gateway:        10.0.0.254
Channel:        1
Device ID:      32
Firmware:       6.76
Manufacturer:   0x019046

BMC Users:
User ID  Username         Enabled  Privilege   
--------------------------------------------------
2        Test           No       User   ****
```

## ⚙️ Configuration

The server is configured via a TOML file. By default, it looks for `config/default.toml`. You can also override settings using environment variables (prefixed with `CMDB_`).

### 🔐 Security Configuration (Required)

For security reasons, the server requires two environment variables to be set before first-time startup:

#### 1. JWT Secret (`CMDB_JWT_SECRET`)

The JWT secret is used to sign authentication tokens. **The server will NOT start** if this is not set or uses the default value.

**Generate a secure JWT secret (32+ characters):**

```bash
# Using OpenSSL (recommended)
export CMDB_JWT_SECRET=$(openssl rand -base64 32)

# Or use /dev/urandom
export CMDB_JWT_SECRET=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1)

# Or generate a longer secret for extra security
export CMDB_JWT_SECRET=$(openssl rand -hex 64)
```

#### 2. Admin Password (`CMDB_ADMIN_PASSWORD`)

On first startup, the server checks if an admin user exists. If not, it creates one using this password. The password must meet complexity requirements:
- At least 12 characters
- At least one uppercase letter (A-Z)
- At least one lowercase letter (a-z)
- At least one number (0-9)
- At least one special character

**Set a secure admin password:**

```bash
# Example (change this to your own secure password)
export CMDB_ADMIN_PASSWORD="YourSecureP@ssword123"

# Or generate a random secure password
export CMDB_ADMIN_PASSWORD=$(openssl rand -base64 16 | tr -d '=' | tr '+/' '@#')
```

**Docker Example with Environment Variables:**

```bash
docker run -itd \
  --name rs-cmdb \
  -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/binaires:/app/binaires \
  -e CMDB_JWT_SECRET="$(openssl rand -base64 32)" \
  -e CMDB_ADMIN_PASSWORD="YourSecureP@ssword123" \
  leex2019/rs-cmdb:${RSCMDB_VERSION}
```

**Without Environment Variables:**

If you don't set `CMDB_ADMIN_PASSWORD`, the server will prompt you interactively for the admin password on first startup (not recommended for automated deployments).

### Configuration File (`config/default.toml`)

```toml
# Server settings
host = "0.0.0.0"           # Bind address
port = 8080                # Server port
log_level = "info"         # Log level: debug, info, warn, error

# Security
# jwt_secret should be set via CMDB_JWT_SECRET environment variable
# Do NOT use the default value in production!
jwt_secret = "change_me_in_production"

# SSH Configuration (for remote service management)
ssh_known_hosts_file = "/etc/cmdb/ssh_known_hosts"  # Path to SSH known_hosts file

# TLS (Optional)
enable_tls = false
# tls_cert = "path/to/cert.pem"
# tls_key = "path/to/key.pem"

# Client Management
poll_interval = 300        # How often clients should report (seconds)
client_timeout = 3600      # Time before a client is marked offline (seconds)
component_missing_grace_period_hours = 24 # Grace period before alerting on missing components

# Database
[database]
path = "data/cmdb.redb"    # Path to the Redb database file

# Message Queue
[queue]
capacity = 1000            # Internal message queue capacity
```

### Environment Variables

Every setting can be overridden by environment variables. Use double underscores `__` for nested keys (e.g. `CMDB_DATABASE__PATH`). Single-level keys stay as `CMDB_<KEY>` (e.g. `CMDB_JWT_SECRET`). For safety, `CMDB_JWT_SECRET` is read explicitly and always overrides defaults.

**Security Variables:**
- `CMDB_JWT_SECRET` - **Required**, minimum 32 characters (use `openssl rand -base64 32` to generate)
- `CMDB_ADMIN_PASSWORD` - **Required on first startup**, must meet complexity requirements

**Optional Variables:**
- `CMDB_HOST` - Server bind address (default: `0.0.0.0`)
- `CMDB_PORT` - Server port (default: `8080`)
- `CMDB_LOG_LEVEL` - Log level: debug, info, warn, error (default: `info`)
- `CMDB_DATABASE__PATH` - Path to database file (default: `data/cmdb.redb`)
- `CMDB_SSH_KNOWN_HOSTS_FILE` - Path to SSH known_hosts file (default: `/etc/cmdb/ssh_known_hosts`)

### SSH Known Hosts Setup

For remote service management via SSH, you need to set up the SSH known_hosts file:

```bash
# Create the directory
sudo mkdir -p /etc/cmdb

# Add a client host to known_hosts (run from server)
ssh-keyscan -H 192.168.1.100 >> /etc/cmdb/ssh_known_hosts

# Set proper permissions
sudo chmod 644 /etc/cmdb/ssh_known_hosts
```

### Password Complexity Requirements

When creating users or changing passwords, the following requirements apply:

- **Minimum length:** 12 characters
- **Uppercase letter:** At least one (A-Z)
- **Lowercase letter:** At least one (a-z)
- **Number:** At least one (0-9)
- **Special character:** At least one (!@#$%^&*, etc.)

**Example valid passwords:**
- `SecureP@ssword123`
- `MyStr0ng!Pass`
- `C0mplex#SecuriTy`

## 📦 Deployment

To deploy the full system manually:

1.  **Prepare Directory**: Create a deployment folder (e.g., `/opt/rs-cmdb`).
2.  **Copy Binaries**:
    *   Copy `target/release/rs-cmdb-server` to `/opt/rs-cmdb/`.
    *   Copy `target/release/rs-cmdb-client` to target client machines.
3.  **Copy Static Files**:
    *   Copy the `front/dist` directory to `/opt/rs-cmdb/dist`.
4.  **Configuration**:
    *   Create `/opt/rs-cmdb/config/default.toml`.
    *   Ensure the `data` directory exists or is writable.

**Directory Structure:**

```text
/opt/rs-cmdb
  ├── rs-cmdb-server
  ├── dist/              <-- Static frontend files
  │    ├── index.html
  │    └── ...
  ├── config/
  │    └── default.toml
  └── data/              <-- Database file will be created here
```

**Run the Server:**

```bash
cd /opt/rs-cmdb

# Set required environment variables
export CMDB_JWT_SECRET=$(openssl rand -base64 32)
export CMDB_ADMIN_PASSWORD="YourSecureP@ssword123"

# Run the server
./rs-cmdb-server
```

Access the UI at `http://localhost:8080`.

**First Login:**

Use the admin username and the password you set via `CMDB_ADMIN_PASSWORD`:
- Username: `admin`
- Password: *(your chosen password)*

## 📄 License

This project is licensed under the [MIT license](LICENSE).

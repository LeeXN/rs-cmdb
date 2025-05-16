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
*   **Default Admin**: `admin` / `admin`

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

### Configuration File (`config/default.toml`)

```toml
# Server settings
host = "0.0.0.0"           # Bind address
port = 8080                # Server port
log_level = "info"         # Log level: debug, info, warn, error
jwt_secret = "change_me"   # Secret key for JWT tokens

# Security (Optional)
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

Every setting can be overridden by environment variables. Use double underscores `__` for nested keys.

*   `CMDB_HOST`
*   `CMDB_PORT`
*   `CMDB_DATABASE__PATH`
*   `CMDB_JWT_SECRET`

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
./rs-cmdb-server
```

Access the UI at `http://localhost:8080`.

**Default Credentials:**
- Username: `admin`
- Password: `admin`

## 📄 License

This project is licensed under the [MIT license](LICENSE).

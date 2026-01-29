# rs-cmdb

[English](README.md) | [中文](README_CN.md)

**rs-cmdb** 是一个完全使用 Rust 构建的轻量级配置管理数据库 (CMDB) 系统.

## 🚀 功能特性

*   **全栈 Rust**: 从内核到 UI 全部使用 Rust 构建，确保内存安全和高性能。
*   **自动发现**: 跨平台代理 (`rs-cmdb-client`) 自动采集硬件规格（CPU、内存、磁盘、网络）并向服务器报告。
*   **资产管理**:
    *   详细的硬件库存跟踪。
    *   变更历史记录（跟踪随时间变化的硬件修改）。
    *   机架和数据中心可视化。
*   **现代化仪表盘**: 实时分析、资源使用统计和健康监控。
*   **安全性**: 基于角色的访问控制 (RBAC) 和安全的 API 认证。
*   **零依赖数据库**: 使用 `Redb`（嵌入式键值存储），无需安装外部数据库（如 PostgreSQL 或 MySQL）。
*   **国际化**: 原生支持英语和简体中文。

## 📺 演示

体验在线演示：
*   **URL**: http://138.2.83.32:8080/
*   **用户名**: `demo`
*   **密码**: `demo@2025.com`

## ⚡ 快速开始

您可以使用 Docker 快速启动 rs-cmdb 服务器。

```bash
# 1. 创建项目目录
mkdir -p /opt/rs-cmdb
cd /opt/rs-cmdb

# 2. 设置版本
export RSCMDB_VERSION="0.0.1"

# 3. 准备客户端二进制目录 (可选，用于自动更新/从服务器下载)
mkdir -p binaires/linux/{x86_64,aarch64}

# 4. 下载客户端二进制文件
# 您可以从 GitHub Release 页面下载，或自行构建。
# https://github.com/LeeXN/rs-cmdb/releases
curl -L -o ./binaires/linux/x86_64/rs-cmdb-client https://github.com/LeeXN/rs-cmdb/releases/download/${RSCMDB_VERSION}/rs-cmdb-client-x86_64-linux-musl
curl -L -o ./binaires/linux/aarch64/rs-cmdb-client https://github.com/LeeXN/rs-cmdb/releases/download/${RSCMDB_VERSION}/rs-cmdb-client-aarch64-linux-musl

# 5. 添加执行权限
chmod +x ./binaires/linux/x86_64/rs-cmdb-client
chmod +x ./binaires/linux/aarch64/rs-cmdb-client

# 6. 使用 Docker 运行服务器
docker run -itd \
  --name rs-cmdb \
  -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/binaires:/app/binaires \
  leex2019/rs-cmdb:${RSCMDB_VERSION}
```

启动后，访问 UI: `http://localhost:8080`。

**⚠️ 重要提示:** 服务器首次启动需要设置安全的环境变量。请参阅下方的[配置](#-配置)部分。

## 🏗 架构

本项目采用 Monorepo 结构：

*   **`server/`**: 基于 **Axum** 构建的后端 API 服务器。它处理 API 请求，管理 **Redb** 数据库，并提供前端静态文件服务。
*   **`front/`**: 基于 **Yew** (WebAssembly) 和 **TailwindCSS** 构建的单页应用 (SPA) 前端。
*   **`client/`**: 运行在目标机器上的轻量级代理，用于采集系统信息。
*   **`common/`**: 包含所有组件使用的数据模型和实用函数的共享 Rust crate。

## 🛠️ 构建与运行

### 使用 Makefile (推荐)

我们提供了 `Makefile` 来简化构建和测试流程。

*   **构建所有组件**: `make build` (构建 Server, Client 和 Frontend)
*   **运行测试**: `make test`
*   **构建 Docker 镜像**: `make docker`
*   **清理产物**: `make clean`
*   **显示帮助**: `make help`

### 使用 Docker

您可以使用 Docker 构建并运行整个系统。

```bash
# 构建镜像
docker build -t rs-cmdb .

# 运行容器
docker run -p 8080:8080 -v $(pwd)/data:/app/data rs-cmdb
```

### 手动构建

#### 前置要求

*   [Rust](https://www.rust-lang.org/tools/install) (最新稳定版)
*   [Trunk](https://trunkrs.dev/) (用于构建前端): `cargo install trunk`
*   Node.js & npm (用于 TailwindCSS)

#### 1. 构建前端

前端编译为 WebAssembly。

```bash
cd front
npm install
trunk build --release
```

构建产物将生成在 `front/dist` 目录中。

#### 2. 构建服务器

```bash
cargo build --release --package server
```

二进制文件位于 `target/release/rs-cmdb-server`。

#### 3. 构建客户端 (Agent)

```bash
cargo build --release --package client
```

二进制文件位于 `target/release/rs-cmdb-client`。

## 💻 客户端独立使用

`rs-cmdb-client` 不仅可以作为代理向服务器报告数据，还可以作为独立工具运行，直接在终端输出详细的硬件信息。

**运行方式:**

直接运行编译好的二进制文件：

```bash
./rs-cmdb-client
```

**输出示例:**

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
MAC Address:    b0:31:a6:71:d6:57
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

## ⚙️ 配置

服务器通过 TOML 文件进行配置。默认情况下，它会读取 `config/default.toml`。您也可以使用环境变量（前缀为 `CMDB_`）覆盖设置。

### 🔐 安全配置（必需）

出于安全考虑，服务器在首次启动前需要设置两个环境变量：

#### 1. JWT 密钥 (`CMDB_JWT_SECRET`)

JWT 密钥用于签名认证令牌。**如果未设置或使用默认值，服务器将无法启动**。

**生成安全的 JWT 密钥（32 字符以上）：**

```bash
# 使用 OpenSSL（推荐）
export CMDB_JWT_SECRET=$(openssl rand -base64 32)

# 或使用 /dev/urandom
export CMDB_JWT_SECRET=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1)

# 或生成更长的密钥以提高安全性
export CMDB_JWT_SECRET=$(openssl rand -hex 64)
```

#### 2. 管理员密码 (`CMDB_ADMIN_PASSWORD`)

首次启动时，服务器会检查是否存在管理员用户。如果不存在，将使用此密码创建管理员。密码必须满足复杂度要求：
- 至少 12 个字符
- 至少一个大写字母 (A-Z)
- 至少一个小写字母 (a-z)
- 至少一个数字 (0-9)
- 至少一个特殊字符

**设置安全的管理员密码：**

```bash
# 示例（请更改为您自己的安全密码）
export CMDB_ADMIN_PASSWORD="YourSecureP@ssword123"

# 或生成随机安全密码
export CMDB_ADMIN_PASSWORD=$(openssl rand -base64 16 | tr -d '=' | tr '+/' '@#')
```

**Docker 使用环境变量示例：**

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

**不使用环境变量：**

如果未设置 `CMDB_ADMIN_PASSWORD`，服务器会在首次启动时交互式提示您输入管理员密码（不推荐用于自动化部署）。

### 配置文件 (`config/default.toml`)

```toml
# 服务器设置
host = "0.0.0.0"           # 绑定地址
port = 8080                # 服务器端口
log_level = "info"         # 日志级别: debug, info, warn, error

# 安全
# jwt_secret 应通过 CMDB_JWT_SECRET 环境变量设置
# 请勿在生产环境中使用默认值！
jwt_secret = "change_me_in_production"

# SSH 配置（用于远程服务管理）
ssh_known_hosts_file = "/etc/cmdb/ssh_known_hosts"  # SSH known_hosts 文件路径

# TLS（可选）
enable_tls = false
# tls_cert = "path/to/cert.pem"
# tls_key = "path/to/key.pem"

# 客户端管理
poll_interval = 300        # 客户端上报间隔 (秒)
client_timeout = 3600      # 客户端离线超时时间 (秒)
component_missing_grace_period_hours = 24 # 组件缺失告警宽限期

# 数据库
[database]
path = "data/cmdb.redb"    # Redb 数据库文件路径

# 消息队列
[queue]
capacity = 1000            # 内部消息队列容量
```

### 环境变量

每个设置都可以通过环境变量覆盖。嵌套键使用双下划线 `__` 分隔（如 `CMDB_DATABASE__PATH`），单层键保持 `CMDB_<KEY>`（如 `CMDB_JWT_SECRET`）。为避免解析歧义，`CMDB_JWT_SECRET` 会被显式读取并覆盖默认值。

**安全变量：**
- `CMDB_JWT_SECRET` - **必需**，最少 32 个字符（使用 `openssl rand -base64 32` 生成）
- `CMDB_ADMIN_PASSWORD` - **首次启动时必需**，必须满足复杂度要求

**可选变量：**
- `CMDB_HOST` - 服务器绑定地址（默认：`0.0.0.0`）
- `CMDB_PORT` - 服务器端口（默认：`8080`）
- `CMDB_LOG_LEVEL` - 日志级别：debug, info, warn, error（默认：`info`）
- `CMDB_DATABASE__PATH` - 数据库文件路径（默认：`data/cmdb.redb`）
- `CMDB_SSH_KNOWN_HOSTS_FILE` - SSH known_hosts 文件路径（默认：`/etc/cmdb/ssh_known_hosts`）

### SSH Known Hosts 设置

如需通过 SSH 进行远程服务管理，您需要设置 SSH known_hosts 文件：

```bash
# 创建目录
sudo mkdir -p /etc/cmdb

# 将客户端主机添加到 known_hosts（在服务器上运行）
ssh-keyscan -H 192.168.1.100 >> /etc/cmdb/ssh_known_hosts

# 设置正确的权限
sudo chmod 644 /etc/cmdb/ssh_known_hosts
```

### 密码复杂度要求

创建用户或更改密码时，需要满足以下要求：

- **最小长度：** 12 个字符
- **大写字母：** 至少一个 (A-Z)
- **小写字母：** 至少一个 (a-z)
- **数字：** 至少一个 (0-9)
- **特殊字符：** 至少一个 (!@#$%^&* 等)

**有效密码示例：**
- `SecureP@ssword123`
- `MyStr0ng!Pass`
- `C0mplex#SecuriTy`

## 📦 部署

手动部署完整系统：

1.  **准备目录**: 创建部署文件夹 (例如 `/opt/rs-cmdb`)。
2.  **复制二进制文件**:
    *   将 `target/release/rs-cmdb-server` 复制到 `/opt/rs-cmdb/`。
    *   将 `target/release/rs-cmdb-client` 复制到目标客户端机器。
3.  **复制静态文件**:
    *   将 `front/dist` 目录复制到 `/opt/rs-cmdb/dist`。
4.  **配置**:
    *   创建 `/opt/rs-cmdb/config/default.toml`。
    *   确保 `data` 目录存在或可写。

**目录结构:**

```text
/opt/rs-cmdb
  ├── rs-cmdb-server
  ├── dist/              <-- 静态前端文件
  │    ├── index.html
  │    └── ...
  ├── config/
  │    └── default.toml
  └── data/              <-- 数据库文件将在此处创建
```

**运行服务器:**

```bash
cd /opt/rs-cmdb

# 设置必需的环境变量
export CMDB_JWT_SECRET=$(openssl rand -base64 32)
export CMDB_ADMIN_PASSWORD="YourSecureP@ssword123"

# 运行服务器
./rs-cmdb-server
```

访问 UI: `http://localhost:8080`。

**首次登录:**

使用管理员用户名和您通过 `CMDB_ADMIN_PASSWORD` 设置的密码：
- 用户名: `admin`
- 密码: *(您选择的密码)*

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE)

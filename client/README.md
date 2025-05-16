# rs-cmdb 客户端

rs-cmdb 客户端用于收集和报告硬件信息。它支持两种工作模式：

1. **直接展示模式**：收集并直接显示硬件信息
2. **服务模式**：作为后台服务运行，定期向服务器推送硬件信息，并响应服务器的拉取请求

## 编译

```bash
# 开发构建
cargo build --bin client

# 生产构建
cargo build --bin client --release

# musl静态编译（适用于Alpine等环境）
cargo build --bin client --target x86_64-unknown-linux-musl --release
```

## 使用方法

### 直接展示模式

直接展示硬件信息，不进行上报：

```bash
# 显示所有硬件信息（默认行为）
./target/release/client

# 显示详细信息
./target/release/client --detail

# 仅显示CPU信息
./target/release/client cpu

# 仅显示内存信息（详细模式）
./target/release/client ram --detail

# 可选硬件类型：
# - os    操作系统信息
# - cpu   CPU信息
# - ram   内存信息
# - disk  磁盘信息
# - nic   网卡信息
# - gpu   GPU信息
# - all   全部信息
```

### 服务模式

作为服务运行，定期上报硬件信息：

```bash
# 使用默认配置启动服务
./target/release/client service

# 指定配置文件
./target/release/client service --config /etc/rs-cmdb/client.toml

# 自定义服务器URL和上报间隔
./target/release/client service --server http://cmdb-server:8080 --interval 600
```

## 配置文件

客户端支持通过配置文件配置，默认按以下优先级查找配置：

1. 命令行参数 `--config` 指定的路径
2. 当前目录的 `config/client.toml`
3. `/etc/rs-cmdb/client.toml`
4. `$HOME/.config/rs-cmdb/client.toml`

配置文件示例：

```toml
# 客户端标识（如果不指定则自动生成）
# client_id = "unique-client-id"

# 覆盖主机名（默认使用系统主机名）
# hostname = "custom-hostname"

# 服务器设置
[server]
# 服务器地址
url = "http://localhost:8080"
# 是否验证TLS证书（用于自签名证书）
verify_tls = true

# 报告设置
[report]
# 是否启用服务模式
service_mode = true
# 是否启用推送模式（主动向服务器推送数据）
push_enabled = true
# 推送间隔（秒）
push_interval = 300
# 是否启用拉取模式（响应服务器请求）
pull_enabled = true
# 收集的硬件组件
components = ["os", "cpu", "ram", "disk", "nic", "gpu"]

# 日志设置
[logging]
# 日志级别: debug, info, warn, error
level = "info"
# 日志文件路径（不指定则输出到控制台）
# file = "/var/log/rs-cmdb-client.log"
```

## 环境变量

所有配置项也可以通过环境变量设置，环境变量会覆盖配置文件中的值。环境变量格式为 `CMDB_CLIENT_<部分>_<设置>`，例如：

```bash
# 设置服务器URL
export CMDB_CLIENT_SERVER_URL="http://cmdb-server:8080"

# 设置推送间隔
export CMDB_CLIENT_REPORT_PUSH_INTERVAL=900

# 设置日志级别
export CMDB_CLIENT_LOGGING_LEVEL=debug
``` 
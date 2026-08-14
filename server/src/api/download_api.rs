use axum::{
    extract::{Extension, Path, Query},
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE, HOST},
    },
    response::{Json, Response},
};
use axum_macros::debug_handler;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info, instrument, warn};

use crate::config::ServerConfig;
use crate::service::client_service::ClientService;

#[derive(Deserialize, Debug)]
pub struct DownloadQuery {
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    arch: Option<String>,
}

static ALLOWED_PLATFORMS: Lazy<Vec<&str>> = Lazy::new(|| vec!["linux", "windows", "darwin"]);
static ALLOWED_ARCHS: Lazy<Vec<&str>> =
    Lazy::new(|| vec!["amd64", "x86_64", "arm64", "aarch64", "i386"]);

fn is_path_traversal(s: &str) -> bool {
    s.contains("..") || s.contains('/') || s.contains('\\')
}

fn validate_download_params(
    platform: &str,
    arch: &str,
    binary_name: &str,
) -> Result<(), &'static str> {
    if !ALLOWED_PLATFORMS.contains(&platform) {
        warn!("Invalid download platform: {}", platform);
        return Err("Invalid platform");
    }
    if !ALLOWED_ARCHS.contains(&arch) {
        warn!("Invalid download arch: {}", arch);
        return Err("Invalid architecture");
    }
    if is_path_traversal(binary_name) {
        warn!(
            "Invalid binary_name (path traversal detected): {}",
            binary_name
        );
        return Err("Invalid binary name");
    }
    Ok(())
}

#[derive(Serialize)]
pub struct ClientInfo {
    pub server_url: String,
    pub download_url: String,
    pub install_script: String,
    pub upgrade_script: String,
    pub ansible_example: String,
    pub systemd_service: String,
    pub config_template: String,
}

/// Get client download information
#[debug_handler]
#[instrument(skip(config, _client_service))]
pub async fn get_client_info(
    headers: HeaderMap,
    Query(params): Query<DownloadQuery>,
    Extension(config): Extension<Arc<ServerConfig>>,
    Extension(_client_service): Extension<Arc<ClientService>>,
) -> Result<Json<ClientInfo>, StatusCode> {
    let platform = params.platform.unwrap_or_else(|| "linux".to_string());
    let arch = params.arch.unwrap_or_else(|| "x86_64".to_string());
    let binary_name = match platform.as_str() {
        "linux" => "rs-cmdb-client",
        "windows" => "rs-cmdb-client.exe",
        "darwin" => "rs-cmdb-client",
        _ => "rs-cmdb-client",
    };

    // Determine server URL from headers or config
    let host = headers
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_string())
        .unwrap_or_else(|| format!("{}:{}", config.host, config.port));

    // Check for X-Forwarded-Proto to determine scheme
    let scheme = headers
        .get("X-Forwarded-Proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let server_url = format!("{}://{}", scheme, host);
    let download_url = format!(
        "{}/api/v1/download/client/{}/{}/{}",
        server_url, platform, arch, binary_name
    );

    let install_script = generate_install_script(&server_url, &platform, &arch);
    let upgrade_script = generate_upgrade_script(&server_url, &platform, &arch);
    let ansible_example = generate_ansible_example(&server_url);
    let systemd_service = generate_systemd_service(&server_url);
    let config_template = generate_config_template(&server_url);

    let response = ClientInfo {
        server_url: server_url.clone(),
        download_url,
        install_script,
        upgrade_script,
        ansible_example,
        systemd_service,
        config_template,
    };

    Ok(Json(response))
}

/// 下载客户端二进制文件
#[debug_handler]
#[instrument]
pub async fn download_client(
    Path((platform, arch, binary_name)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, &'static str)> {
    if let Err(msg) = validate_download_params(&platform, &arch, &binary_name) {
        return Err((StatusCode::BAD_REQUEST, msg));
    }
    let file_path = format!("binaries/{}/{}/{}", platform, arch, binary_name);
    info!("Downloading client binary from path: {}", file_path);

    match fs::read(&file_path).await {
        Ok(content) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                CONTENT_TYPE,
                "application/octet-stream"
                    .parse()
                    .expect("CONTENT_TYPE header value should be valid"),
            );
            headers.insert(
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", binary_name)
                    .parse()
                    .expect("CONTENT_DISPOSITION header value should be valid"),
            );

            info!("Client binary downloaded: {}", file_path);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(content.into())
                .expect("Response builder with known content should be valid"))
        }
        Err(e) => {
            error!("Failed to read client binary: {}", e);
            Err((StatusCode::NOT_FOUND, "Client binary not found"))
        }
    }
}

/// Get install script directly
#[debug_handler]
#[instrument(skip(config))]
pub async fn get_install_script(
    headers: HeaderMap,
    Extension(config): Extension<Arc<ServerConfig>>,
) -> Result<Response, StatusCode> {
    // Determine server URL from headers or config
    let host = headers
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_string())
        .unwrap_or_else(|| format!("{}:{}", config.host, config.port));

    // Check for X-Forwarded-Proto to determine scheme
    let scheme = headers
        .get("X-Forwarded-Proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let server_url = format!("{}://{}", scheme, host);
    let script = generate_install_script(&server_url, "linux", "x86_64");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/x-shellscript")
        .body(script.into())
        .expect("Response builder with known content should be valid"))
}

/// Get the Linux agent upgrade script directly.
#[debug_handler]
#[instrument(skip(config))]
pub async fn get_upgrade_script(
    headers: HeaderMap,
    Extension(config): Extension<Arc<ServerConfig>>,
) -> Result<Response, StatusCode> {
    let host = headers
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_string())
        .unwrap_or_else(|| format!("{}:{}", config.host, config.port));

    let scheme = headers
        .get("X-Forwarded-Proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let server_url = format!("{}://{}", scheme, host);
    let script = generate_upgrade_script(&server_url, "linux", "x86_64");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/x-shellscript")
        .body(script.into())
        .expect("Response builder with known content should be valid"))
}

fn shell_single_quote(value: &str) -> String {
    value.replace('\'', "'\\\"'\\\"'")
}

fn generate_upgrade_script(server_url: &str, platform: &str, _arch: &str) -> String {
    if platform != "linux" {
        return "# Agent upgrade is currently supported on Linux only.".to_string();
    }

    let server_url = shell_single_quote(server_url);
    format!(
        r#"#!/usr/bin/env bash
# RS-CMDB Client Upgrade Script
#
# This script replaces only the agent binary. It preserves client.toml,
# the client ID and the persisted agent token, and restarts the service only
# when it was already running.

set -Eeuo pipefail
umask 077

SERVER_URL='{server_url}'
INSTALL_PATH="/usr/local/bin/rs-cmdb-client"
SERVICE_NAME="rs-cmdb-client.service"
CONFIG_PATH="/etc/rs-cmdb/client.toml"
LEGACY_CONFIG_PATH="/root/.config/rs-cmdb/client.toml"
STATE_DIR="/var/lib/rs-cmdb"
LEGACY_STATE_DIR="/root/.config/rs-cmdb"
TEMP_PATH=""
BACKUP_PATH=""

run_root() {{
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    else
        sudo "$@"
    fi
}}

cleanup() {{
    if [ -n "${{TEMP_PATH}}" ]; then
        run_root rm -f "${{TEMP_PATH}}" 2>/dev/null || true
    fi
}}
trap cleanup EXIT

if ! command -v curl >/dev/null 2>&1; then
    echo "curl is required to download the agent binary." >&2
    exit 1
fi

read_client_id() {{
    run_root awk -F= '
        /^[[:space:]]*client_id[[:space:]]*=/ {{
            value=$2
            gsub(/^[[:space:]]*"|"[[:space:]]*$/, "", value)
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
            print value
            exit
        }}
    ' "$1" 2>/dev/null || true
}}

# Older installers could leave an empty ID in /etc while the running agent
# persisted its real identity under root's default config path. Recover that
# ID before replacing the binary so the upgraded agent updates the same CMDB
# record instead of registering a duplicate.
CURRENT_CLIENT_ID="$(read_client_id "$CONFIG_PATH")"
if [ -z "$CURRENT_CLIENT_ID" ] && run_root test -f "$LEGACY_CONFIG_PATH"; then
    LEGACY_CLIENT_ID="$(read_client_id "$LEGACY_CONFIG_PATH")"
    case "$LEGACY_CLIENT_ID" in
        ""|*[!A-Za-z0-9._:-]*)
            echo "No valid legacy client ID found; the upgraded agent will generate and persist one."
            ;;
        *)
            echo "Recovering existing client ID from $LEGACY_CONFIG_PATH..."
            run_root cp -p "$CONFIG_PATH" "${{CONFIG_PATH}}.pre-upgrade.bak"
            CONFIG_TEMP="$(run_root mktemp "${{CONFIG_PATH}}.identity.XXXXXX")"
            if run_root grep -q '^[[:space:]]*client_id[[:space:]]*=' "$CONFIG_PATH"; then
                run_root awk -v id="$LEGACY_CLIENT_ID" '
                    /^[[:space:]]*client_id[[:space:]]*=/ {{
                        print "client_id = \"" id "\""
                        next
                    }}
                    {{ print }}
                ' "$CONFIG_PATH" | run_root tee "$CONFIG_TEMP" >/dev/null
            else
                {{ printf 'client_id = "%s"\n' "$LEGACY_CLIENT_ID"; run_root cat "$CONFIG_PATH"; }} \
                    | run_root tee "$CONFIG_TEMP" >/dev/null
            fi
            run_root install -o root -g root -m 0644 "$CONFIG_TEMP" "$CONFIG_PATH"
            run_root rm -f "$CONFIG_TEMP"
            CURRENT_CLIENT_ID="$LEGACY_CLIENT_ID"
            ;;
    esac
fi

# Tokens are credentials, so they are deliberately kept out of the
# world-readable client.toml. Migrate the token that belongs to the preserved
# client ID into the system state directory before changing the binary. Keep a
# generic copy as well so rolling back to an older binary remains safe.
if [ -n "$CURRENT_CLIENT_ID" ]; then
    TOKEN_FILE_ID="$(printf '%s' "$CURRENT_CLIENT_ID" | tr -c 'A-Za-z0-9_-' '_')"
    TOKEN_SOURCE=""
    if [ -n "${{RS_CMDB_AGENT_TOKEN:-}}" ]; then
        echo "Installing administrator-recovered Agent Token..."
        run_root install -d -o root -g root -m 0700 "$STATE_DIR"
        printf '%s' "$RS_CMDB_AGENT_TOKEN" \
            | run_root tee "$STATE_DIR/agent_token_${{TOKEN_FILE_ID}}" >/dev/null
        run_root chmod 0600 "$STATE_DIR/agent_token_${{TOKEN_FILE_ID}}"
        TOKEN_SOURCE="$STATE_DIR/agent_token_${{TOKEN_FILE_ID}}"
        unset RS_CMDB_AGENT_TOKEN
    else
        for CANDIDATE in \
            "$STATE_DIR/agent_token_${{TOKEN_FILE_ID}}" \
            "$LEGACY_STATE_DIR/agent_token_${{TOKEN_FILE_ID}}" \
            "$STATE_DIR/agent_token" \
            "$LEGACY_STATE_DIR/agent_token" \
            "$STATE_DIR/agent_token_unknown" \
            "$LEGACY_STATE_DIR/agent_token_unknown"; do
            if run_root test -s "$CANDIDATE"; then
                TOKEN_SOURCE="$CANDIDATE"
                break
            fi
        done
    fi
    if [ -z "$TOKEN_SOURCE" ]; then
        echo "No persisted Agent Token was found for client $CURRENT_CLIENT_ID." >&2
        echo "The server will only allow migration when its existing record is tokenless and has the same valid hardware serial." >&2
        run_root install -d -o root -g root -m 0700 "$STATE_DIR"
    else
        echo "Migrating existing Agent Token to $STATE_DIR..."
        run_root install -d -o root -g root -m 0700 "$STATE_DIR"
        if [ "$TOKEN_SOURCE" != "$STATE_DIR/agent_token_${{TOKEN_FILE_ID}}" ]; then
            run_root install -o root -g root -m 0600 \
                "$TOKEN_SOURCE" "$STATE_DIR/agent_token_${{TOKEN_FILE_ID}}"
        fi
        if [ "$TOKEN_SOURCE" != "$STATE_DIR/agent_token" ]; then
            run_root install -o root -g root -m 0600 \
                "$TOKEN_SOURCE" "$STATE_DIR/agent_token"
        fi
    fi
else
    # This is an old installation without a durable identity. The upgraded
    # agent will generate and persist an ID, but it cannot reclaim an existing
    # server record without that record's Token.
    echo "Warning: no existing client ID was found; this host will register as a new identity." >&2
fi

case "$(uname -m)" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $(uname -m)" >&2
        exit 1
        ;;
esac

DOWNLOAD_URL="${{SERVER_URL}}/api/v1/download/client/linux/${{ARCH}}/rs-cmdb-client"
echo "Downloading RS-CMDB Client for $ARCH..."

run_root mkdir -p "$(dirname "$INSTALL_PATH")"
TEMP_PATH="$(run_root mktemp "${{INSTALL_PATH}}.new.XXXXXX")"
run_root curl --fail --silent --show-error --location --retry 3 \
    --output "$TEMP_PATH" "$DOWNLOAD_URL"
run_root test -s "$TEMP_PATH"
run_root chmod 0755 "$TEMP_PATH"

WAS_ACTIVE=0
HAS_SYSTEMD=0
if run_root test -f "$INSTALL_PATH"; then
    BACKUP_PATH="${{INSTALL_PATH}}.bak.$(date +%Y%m%d%H%M%S)"
    run_root cp -p "$INSTALL_PATH" "$BACKUP_PATH"
fi

if command -v systemctl >/dev/null 2>&1 && \
   run_root systemctl cat "$SERVICE_NAME" >/dev/null 2>&1; then
    HAS_SYSTEMD=1
    if run_root systemctl is-active --quiet "$SERVICE_NAME"; then
        WAS_ACTIVE=1
        echo "Stopping $SERVICE_NAME..."
        run_root systemctl stop "$SERVICE_NAME"
    fi
fi

echo "Installing the new agent binary..."
if ! run_root install -o root -g root -m 0755 "$TEMP_PATH" "$INSTALL_PATH"; then
    echo "Failed to install the new binary; restoring the previous binary." >&2
    if [ -n "$BACKUP_PATH" ] && run_root test -f "$BACKUP_PATH"; then
        run_root install -o root -g root -m 0755 "$BACKUP_PATH" "$INSTALL_PATH"
    fi
    if [ "$WAS_ACTIVE" -eq 1 ]; then
        run_root systemctl start "$SERVICE_NAME" || true
    fi
    exit 1
fi

if [ "$HAS_SYSTEMD" -eq 1 ]; then
    run_root systemctl daemon-reload
    if [ "$WAS_ACTIVE" -eq 1 ]; then
        echo "Starting $SERVICE_NAME..."
        if ! run_root systemctl start "$SERVICE_NAME"; then
            echo "The new agent failed to start; rolling back the previous binary." >&2
            if [ -n "$BACKUP_PATH" ] && run_root test -f "$BACKUP_PATH"; then
                run_root install -o root -g root -m 0755 "$BACKUP_PATH" "$INSTALL_PATH"
                run_root systemctl start "$SERVICE_NAME" || true
            fi
            exit 1
        fi
    else
        echo "The service was not running; binary upgraded without starting it."
    fi
else
    echo "systemd service not found; binary upgraded without restarting a service."
fi

echo "RS-CMDB Client upgraded successfully."
if [ -n "$BACKUP_PATH" ]; then
    echo "Previous binary backup: $BACKUP_PATH"
fi
"#,
        server_url = server_url
    )
}

fn generate_ansible_example(server_url: &str) -> String {
    let server_url = server_url
        .trim_end_matches('/')
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\r', '\n'], "");
    format!(
        r#"---
# Install or upgrade the RS-CMDB agent on Linux hosts.
# Existing client.toml and agent token files are preserved.

- name: Install or upgrade RS-CMDB agent
  hosts: rs_cmdb_agents
  become: true
  gather_facts: true
  vars:
    rs_cmdb_server_url: "{server_url}"
    rs_cmdb_agent_arch: >-
      {{{{ 'aarch64' if ansible_architecture in ['aarch64', 'arm64'] else 'x86_64' }}}}
    rs_cmdb_agent_url: "{{{{ rs_cmdb_server_url }}}}/api/v1/download/client/linux/{{{{ rs_cmdb_agent_arch }}}}/rs-cmdb-client"
    rs_cmdb_agent_path: /usr/local/bin/rs-cmdb-client
    rs_cmdb_config_path: /etc/rs-cmdb/client.toml
    rs_cmdb_service_name: rs-cmdb-client.service
    # For a host whose credential was lost, obtain this from the admin-only
    # recovery API and store it with Ansible Vault.
    # rs_cmdb_agent_recovery_token: "vault-encrypted-value"
    # Pin this to a trusted SHA-256 when distributing through CI.
    # rs_cmdb_agent_checksum: "sha256:..."

  tasks:
    - name: Ensure RS-CMDB configuration directory exists
      ansible.builtin.file:
        path: /etc/rs-cmdb
        state: directory
        owner: root
        group: root
        mode: "0755"

    - name: Ensure RS-CMDB credential state directory exists
      ansible.builtin.file:
        path: /var/lib/rs-cmdb
        state: directory
        owner: root
        group: root
        mode: "0700"

    - name: Install an administrator-recovered Agent Token when supplied
      ansible.builtin.copy:
        dest: /var/lib/rs-cmdb/agent_token
        content: "{{{{ rs_cmdb_agent_recovery_token }}}}"
        owner: root
        group: root
        mode: "0600"
      when:
        - rs_cmdb_agent_recovery_token is defined
        - rs_cmdb_agent_recovery_token | length > 0
      no_log: true

    - name: Create a default configuration on first install only
      ansible.builtin.copy:
        dest: "{{{{ rs_cmdb_config_path }}}}"
        force: false
        owner: root
        group: root
        mode: "0644"
        content: |
          # client_id is generated and persisted by the agent on first start.
          # hostname is auto-detected unless explicitly configured.

          [server]
          url = "{{{{ rs_cmdb_server_url }}}}/api/v1"
          verify_tls = true

          [report]
          service_mode = true
          push_enabled = true
          push_interval = 300
          pull_enabled = true
          components = ["sys", "os", "cpu", "ram", "disk", "nic", "gpu", "ipmi"]

          [logging]
          level = "info"

    - name: Read the installed agent checksum
      ansible.builtin.stat:
        path: "{{{{ rs_cmdb_agent_path }}}}"
        checksum_algorithm: sha1
      register: rs_cmdb_installed_agent

    - name: Download the current agent binary
      ansible.builtin.get_url:
        url: "{{{{ rs_cmdb_agent_url }}}}"
        dest: "{{{{ rs_cmdb_agent_path }}}}.download"
        owner: root
        group: root
        mode: "0755"
        force: true
        checksum: "{{{{ rs_cmdb_agent_checksum | default(omit) }}}}"
      register: rs_cmdb_downloaded_agent

    - name: Atomically install the new agent binary when it changed
      ansible.builtin.command:
        cmd: "mv -- {{{{ rs_cmdb_agent_path }}}}.download {{{{ rs_cmdb_agent_path }}}}"
      when:
        - rs_cmdb_downloaded_agent.changed
        - not rs_cmdb_installed_agent.stat.exists or rs_cmdb_installed_agent.stat.checksum != rs_cmdb_downloaded_agent.checksum_src
      notify: Restart RS-CMDB agent

    - name: Remove the downloaded binary when no upgrade was needed
      ansible.builtin.file:
        path: "{{{{ rs_cmdb_agent_path }}}}.download"
        state: absent

    - name: Install the systemd unit on first install only
      ansible.builtin.copy:
        dest: /etc/systemd/system/rs-cmdb-client.service
        force: false
        owner: root
        group: root
        mode: "0644"
        content: |
          [Unit]
          Description=RS-CMDB Client
          After=network.target

          [Service]
          Type=simple
          User=root
          ExecStart=/usr/local/bin/rs-cmdb-client --config /etc/rs-cmdb/client.toml service
          Restart=always
          RestartSec=10

          [Install]
          WantedBy=multi-user.target
      notify: Restart RS-CMDB agent

    - name: Ensure the RS-CMDB agent is enabled and running
      ansible.builtin.systemd:
        name: "{{{{ rs_cmdb_service_name }}}}"
        enabled: true
        state: started
        daemon_reload: true

  handlers:
    - name: Restart RS-CMDB agent
      ansible.builtin.systemd:
        name: "{{{{ rs_cmdb_service_name }}}}"
        state: restarted
        enabled: true
        daemon_reload: true
"#,
        server_url = server_url
    )
}

fn generate_install_script(server_url: &str, platform: &str, arch: &str) -> String {
    match platform {
        "linux" => format!(
            r#"#!/bin/bash
# RS-CMDB Client Install Script

set -e

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

CLIENT_URL="{}/api/v1/download/client/linux/$ARCH/rs-cmdb-client"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/rs-cmdb"
STATE_DIR="/var/lib/rs-cmdb"
SERVICE_FILE="/etc/systemd/system/rs-cmdb-client.service"

echo "Installing RS-CMDB Client..."
echo "Detected architecture: $ARCH"

# Create directories
sudo mkdir -p $INSTALL_DIR
sudo mkdir -p $CONFIG_DIR
sudo install -d -o root -g root -m 0700 $STATE_DIR

# Download client binary
echo "Downloading client binary..."
sudo wget -O $INSTALL_DIR/rs-cmdb-client "$CLIENT_URL"
sudo chmod +x $INSTALL_DIR/rs-cmdb-client

# Generate config
echo "Generating configuration..."
sudo $INSTALL_DIR/rs-cmdb-client generate-config --server {}/api/v1 --output $CONFIG_DIR/client.toml

# Create systemd service
echo "Creating systemd service..."
sudo tee $SERVICE_FILE > /dev/null << 'EOF'
[Unit]
Description=RS-CMDB Client
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/rs-cmdb-client --config /etc/rs-cmdb/client.toml service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable rs-cmdb-client
sudo systemctl start rs-cmdb-client

echo "RS-CMDB Client installed and started successfully!"
echo "Check status with: sudo systemctl status rs-cmdb-client"
"#,
            server_url, server_url
        ),
        "windows" => format!(
            r#"@echo off
REM RS-CMDB Client Install Script for Windows

echo Installing RS-CMDB Client...

REM Create directories
if not exist "C:\Program Files\rs-cmdb" mkdir "C:\Program Files\rs-cmdb"
if not exist "C:\ProgramData\rs-cmdb" mkdir "C:\ProgramData\rs-cmdb"

REM Download client binary (requires PowerShell)
echo Downloading client binary...
powershell -Command "Invoke-WebRequest -Uri '{}/api/v1/download/client/{}/{}/rs-cmdb-client.exe' -OutFile 'C:\Program Files\rs-cmdb\rs-cmdb-client.exe'"

REM Generate config
echo Generating configuration...
"C:\Program Files\rs-cmdb\rs-cmdb-client.exe" generate-config --server {} --output "C:\ProgramData\rs-cmdb\client.toml"

echo RS-CMDB Client installed successfully!
echo Run manually with: "C:\Program Files\rs-cmdb\rs-cmdb-client.exe" --config "C:\ProgramData\rs-cmdb\client.toml" service
"#,
            server_url, platform, arch, server_url
        ),
        _ => "# Unsupported platform".to_string(),
    }
}

fn generate_systemd_service(_server_url: &str) -> String {
    r#"[Unit]
Description=RS-CMDB Client
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/rs-cmdb-client --config /etc/rs-cmdb/client.toml service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target"#
        .to_string()
}

fn generate_config_template(server_url: &str) -> String {
    format!(
        r#"# RS-CMDB Client Configuration

# 客户端唯一标识符会在首次启动时自动生成并持久化。
# 如需覆盖自动检测到的主机名，可取消下一行注释：
# hostname = "custom-hostname"

[server]
# 服务器地址
url = "{}/api/v1"
# 是否验证TLS证书
verify_tls = true

[report]
# 是否启用服务模式
service_mode = true
# 是否启用推送模式
push_enabled = true
# 推送间隔（秒）
push_interval = 300
# 是否启用拉取模式
pull_enabled = true
# 收集的硬件组件
components = ["sys", "os", "cpu", "ram", "disk", "nic", "gpu", "ipmi"]

[logging]
# 日志级别
level = "info"
# 日志文件路径（可选）
file = "/var/log/rs-cmdb-client.log"
"#,
        server_url
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_path_traversal() {
        assert!(is_path_traversal("../../etc/passwd"));
        assert!(is_path_traversal("foo/bar"));
        assert!(is_path_traversal("foo\\bar"));
        assert!(is_path_traversal(".."));
        assert!(!is_path_traversal("rs-cmdb-client"));
        assert!(!is_path_traversal("rs-cmdb-client.exe"));
        assert!(!is_path_traversal("client-binary-v1.0"));
    }

    #[test]
    fn test_validate_download_params_valid() {
        assert!(validate_download_params("linux", "amd64", "rs-cmdb-client").is_ok());
        assert!(validate_download_params("windows", "x86_64", "rs-cmdb-client.exe").is_ok());
        assert!(validate_download_params("darwin", "arm64", "rs-cmdb-client").is_ok());
        assert!(validate_download_params("linux", "aarch64", "rs-cmdb-client").is_ok());
    }

    #[test]
    fn test_validate_download_params_invalid_platform() {
        assert!(validate_download_params("solaris", "amd64", "rs-cmdb-client").is_err());
        assert!(validate_download_params("", "amd64", "rs-cmdb-client").is_err());
    }

    #[test]
    fn test_validate_download_params_invalid_arch() {
        assert!(validate_download_params("linux", "mips", "rs-cmdb-client").is_err());
        assert!(validate_download_params("linux", "", "rs-cmdb-client").is_err());
    }

    #[test]
    fn test_validate_download_params_path_traversal() {
        assert!(validate_download_params("linux", "amd64", "../../etc/passwd").is_err());
        assert!(validate_download_params("linux", "amd64", "foo/bar").is_err());
        assert!(validate_download_params("linux", "amd64", "foo\\bar").is_err());
    }

    #[test]
    fn test_upgrade_script_preserves_agent_state_and_rolls_back() {
        let script = generate_upgrade_script("https://cmdb.example/o'h", "linux", "x86_64");

        assert!(script.starts_with("#!/usr/bin/env bash"));
        assert!(script.contains("/api/v1/download/client/linux/${ARCH}/rs-cmdb-client"));
        assert!(script.contains("client.toml"));
        assert!(script.contains("agent token"));
        assert!(script.contains("LEGACY_CONFIG_PATH=\"/root/.config/rs-cmdb/client.toml\""));
        assert!(script.contains("Recovering existing client ID"));
        assert!(script.contains("rolling back the previous binary"));
        assert!(script.contains("o'\\\"'\\\"'h"));
        assert!(!script.contains("generate-config"));
    }

    #[cfg(unix)]
    #[test]
    fn test_upgrade_script_has_valid_shell_syntax() {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let script = generate_upgrade_script("https://cmdb.example", "linux", "x86_64");
        let mut child = Command::new("bash")
            .arg("-n")
            .stdin(Stdio::piped())
            .spawn()
            .expect("bash should be available for script syntax validation");
        child
            .stdin
            .take()
            .expect("bash stdin should be available")
            .write_all(script.as_bytes())
            .expect("script should be written to bash");
        assert!(child.wait().expect("bash should exit").success());
    }

    #[test]
    fn test_ansible_example_is_install_and_upgrade_safe() {
        let example = generate_ansible_example("https://cmdb.example/");

        assert!(example.contains("rs_cmdb_server_url: \"https://cmdb.example\""));
        assert!(example.contains("ansible_architecture"));
        assert!(example.contains("force: false"));
        assert!(example.contains("rs_cmdb_agent_checksum"));
        assert!(example.contains("Atomically install the new agent binary"));
        assert!(example.contains("rs-cmdb-client.service"));
    }

    #[test]
    fn test_upgrade_script_rejects_non_linux_platforms() {
        assert_eq!(
            generate_upgrade_script("https://cmdb.example", "windows", "x86_64"),
            "# Agent upgrade is currently supported on Linux only."
        );
    }
}

#!/bin/bash
set -euo pipefail

# build the project as release 
cargo build --release
strip target/release/super_v

# Config
USERNAME="$(id -un)"
USERHOME="${HOME}"
SERVICE_NAME="super_v.service"
USER_DIR="${USERHOME}/.config/systemd/user"
USER_PATH="${USER_DIR}/${SERVICE_NAME}"
USER_LOG="${USERHOME}/superv.log"

# Stop and remove any system service to avoid conflicts (requires sudo)
echo "[*] removing system service (if present)..."
sudo rm /usr/local/bin/super_v || echo "super_v cli already removed"
sudo systemctl stop "${SERVICE_NAME}" 2>/dev/null || true
sudo systemctl disable "${SERVICE_NAME}" 2>/dev/null || true
sudo rm -f "/etc/systemd/system/${SERVICE_NAME}" 2>/dev/null || true
sudo systemctl daemon-reload

echo "[*] Installing service..."
sudo cp ./target/release/super_v /usr/local/bin/
sudo chmod +x /usr/local/bin/super_v

# Ensure user unit dir exists
echo "[*] creating user unit dir..."
mkdir -p "${USER_DIR}"

# Write user unit
echo "[*] writing user service to ${USER_PATH}..."
cat > "${USER_PATH}" <<EOF
[Unit]
Description=SuperV Clipboard Manager (user)
After=graphical-session.target

[Service]
Type=simple
Environment=RUST_BACKTRACE=1
Environment=RUST_LOG=info
Environment=DISPLAY=:0
Environment=XDG_RUNTIME_DIR=/run/user/1000
ExecStartPre=/usr/local/bin/super_v clean
ExecStart=/usr/local/bin/super_v start
Restart=on-failure
RestartSec=5
StandardOutput=append:%h/superv.log
StandardError=append:%h/superv.log

[Install]
WantedBy=default.target
EOF

# Ensure the log file exists and is writable by the user (no sudo required)
echo "[*] creating user-writable log at ${USER_LOG}..."
touch "${USER_LOG}"
chmod 600 "${USER_LOG}"
chown "${USERNAME}:${USERNAME}" "${USER_LOG}"

# Enable lingering so the service can run without an active login (requires sudo)
echo "[*] enabling linger for ${USERNAME} (requires sudo)..."
sudo loginctl enable-linger "${USERNAME}"

# Reload user daemon, enable and start the unit
echo "[*] reloading user systemd and starting service..."
systemctl --user daemon-reload
systemctl --user enable --now "${SERVICE_NAME}"

echo
echo "[*] status (user unit):"
systemctl --user status "${SERVICE_NAME}" --no-pager

echo
echo "[*] tailing ${USER_LOG} (press Ctrl-C to stop):"
tail -n 200 -f "${USER_LOG}"
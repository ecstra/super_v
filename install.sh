#!/bin/bash
set -euo pipefail

echo "[*] installing pre-requisites..."
sudo apt install build-essential pkg-config libgtk-4-dev libgraphene-1.0-dev libgdk-pixbuf2.0-dev libpango1.0-dev

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

# Step 1: One-time udev rule setup for ydotoold
echo "[*] setting up udev rule for /dev/uinput (one-time)..."
if [ ! -f /etc/udev/rules.d/99-ydotoold.rules ]; then
  sudo tee /etc/udev/rules.d/99-ydotoold.rules >/dev/null <<'EOF'
KERNEL=="uinput", MODE="0660", GROUP="input"
EOF
  sudo udevadm control --reload
  sudo udevadm trigger --action=change /dev/uinput || true
  if ! getent group input >/dev/null; then
    sudo groupadd input
  fi
  sudo usermod -aG input "$USERNAME"
  echo "[*] udev rule added and user added to input group. You may need to log out and log back in once."
else
  echo "[*] udev rule already exists, skipping."
fi

# Step 2: Remove any existing system-level service
echo "[*] removing system service (if present)..."
sudo rm /usr/local/bin/super_v || echo "super_v cli already removed"
sudo systemctl stop "${SERVICE_NAME}" 2>/dev/null || true
sudo systemctl disable "${SERVICE_NAME}" 2>/dev/null || true
sudo rm -f "/etc/systemd/system/${SERVICE_NAME}" 2>/dev/null || true
sudo systemctl daemon-reload

# Step 3: Install the binary
echo "[*] Installing binary..."
sudo cp ./target/release/super_v /usr/local/bin/
sudo chmod +x /usr/local/bin/super_v

# Step 4: Write user systemd service (avoid sudo in the unit)
echo "[*] creating user unit dir..."
mkdir -p "${USER_DIR}"

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

# clean before start
ExecStartPre=/usr/local/bin/super_v clean
# start ydotoold if not running (runs as user; udev/input group setup done by installer)
ExecStartPre=/bin/sh -c 'pgrep -x ydotoold >/dev/null || (command -v ydotoold >/dev/null && ydotoold & sleep 1 || true)'

ExecStart=/usr/local/bin/super_v start
Restart=on-failure
RestartSec=5
StandardOutput=append:%h/superv.log
StandardError=append:%h/superv.log

[Install]
WantedBy=default.target
EOF

# Step 5: Ensure log file
echo "[*] creating user-writable log at ${USER_LOG}..."
touch "${USER_LOG}"
sudo chmod 600 "${USER_LOG}"
sudo chown "${USERNAME}:${USERNAME}" "${USER_LOG}"

# Step 6: Enable linger and start service
echo "[*] enabling linger for ${USERNAME} (requires sudo)..."
sudo loginctl enable-linger "${USERNAME}"

echo "[*] reloading user systemd and starting service..."
systemctl --user daemon-reload
systemctl --user enable --now "${SERVICE_NAME}"

echo
echo "[*] status (user unit):"
systemctl --user status "${SERVICE_NAME}" --no-pager

# Step 7: One-time sudo chmod on the socket (wait a bit for the socket to appear)
echo "[*] attempting one-time sudo chmod on /tmp/.ydotool_socket (will prompt for password if needed)..."
for i in $(seq 1 10); do
  if [ -e /tmp/.ydotool_socket ]; then
    sudo chmod 666 /tmp/.ydotool_socket && echo "[*] /tmp/.ydotool_socket perms set" || echo "[!] chmod failed"
    break
  fi
  sleep 0.5
done || true

if [ ! -e /tmp/.ydotool_socket ]; then
  echo "[*] socket not found. It will be created next time ydotoold runs; run this script again or run 'sudo chmod 666 /tmp/.ydotool_socket' once the socket exists."
fi

echo
echo "[*] tailing ${USER_LOG} (press Ctrl-C to stop):"
tail -n 200 -f "${USER_LOG}"
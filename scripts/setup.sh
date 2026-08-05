#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status, 
# just to make sure we don't carry on if a spanner gets thrown in the works.
set -e

# 1. Check if cargo exists
if ! command -v cargo &> /dev/null; then
    echo "Error: 'cargo' could not be found. Please install Rust first!"
    exit 1
fi

echo "Cargo is installed!"

# Grab the current user and group so we can dynamically chuck them into the systemd file
CURRENT_USER=$(id -un)
CURRENT_GROUP=$(id -gn)

# 2. Check where we are currently executing from
if [ -f "Cargo.toml" ]; then
    echo "Running from the project root.,"
elif [ -f "../Cargo.toml" ] && [ "$(basename "$PWD")" = "scripts" ]; then
    echo "Running from the scripts folder- changing to the project's root directory..."
    cd ..
else
    echo "Cargo.toml can't be found - Please run this script from either the project root or the scripts folder."
    exit 1
fi

# 3. Build the release binary
echo "Building fr24d in release mode... Please note that this may take a couple of minutes on a Pi"
cargo build --release

# 4. Check for the existance of systemd
if [ ! -d "/run/systemd/system" ]; then
    echo "systemd isn't running on this machine."
    echo "This script relies on systemd to set up the background service"
    echo "Please look up the instructions on how to set up a service for your distro"
    exit 1
fi

echo "systemd is installed"

# 5. Copy the compiled binary to /usr/local/bin
echo "Copying the binary to /usr/local/bin..."
# Using sudo here in case your standard user doesn't have write permissions in /usr/local/bin
sudo cp target/release/fr24d /usr/local/bin/fr24d
sudo chmod +x /usr/local/bin/fr24d

# 6. Create the systemd service file
echo "Creating the systemd service file at /etc/systemd/system/fr24d.service..."

sudo tee /etc/systemd/system/fr24d.service > /dev/null <<EOF
[Unit]
Description=fr24d Background Service
After=network.target
Wants=network.target

[Service]
Type=simple

# Automatically updated to the user who ran this script
User=$CURRENT_USER
Group=$CURRENT_GROUP

# The absolute path to your compiled binary
ExecStart=/usr/local/bin/fr24d

# Restart it if it crashes
Restart=on-failure
RestartSec=10

# Logging: this sends standard output and errors straight to journalctl.
StandardOutput=journal
StandardError=journal

# Security: A few sensible restrictions to lock things down a bit.
ProtectSystem=full
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
EOF

# 7. Reload systemd, enable, start, and check the status
echo "Reloading systemd daemon..."
sudo systemctl daemon-reload

echo "Enabling the fr24d service to start on boot..."
sudo systemctl enable fr24d.service

echo "Starting the fr24d service..."
sudo systemctl start fr24d.service

echo "Done and dusted! Here is the current status of your service:"
sudo systemctl status fr24d.service --no-pager
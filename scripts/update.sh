#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

# 1. Check if we're in a git repository and if 'origin' exists
if ! git remote get-url origin > /dev/null 2>&1; then
    echo "Error: Git 'origin' does not exist. Cannot pull changes"
    exit 1
fi

# 2. Check where we are currently executing from
if [ -f "Cargo.toml" ]; then
    echo "Running from the project root.,"
elif [ -f "../Cargo.toml" ] && [ "$(basename "$PWD")" = "scripts" ]; then
    echo "Running from the scripts folder. Popping up to the root directory..."
    cd ..
else
    echo "I can't find Cargo.toml."
    echo "Please run this script from either the project root or the scripts folder."
    exit 1
fi

# 3. Pull the latest changes from the repository
echo "Pulling the latest changes from origin. Brilliant..."
git pull

# 4. Check if the systemd service actually exists
if [ ! -f /etc/systemd/system/fr24d.service ]; then
    echo "The fr24d service doesn't seem to exist at /etc/systemd/system/fr24d.service."
    echo "Make sure you've run the installation script first to get it all set up."
    exit 1
fi

echo "Service found. Cracking on with the build..."

# 5. Rebuild the release binary
echo "Rebuilding fr24d..."
cargo build --release

# 6. Copy the newly compiled binary over the old one
echo "Copying the updated binary to /usr/local/bin..."
sudo cp target/release/fr24d /usr/local/bin/fr24d
sudo chmod +x /usr/local/bin/fr24d

# 7. Restart the service so it picks up the new binary
echo "Restarting the fr24d service..."
sudo systemctl restart fr24d.service

echo "fr24-detector has been updated!"
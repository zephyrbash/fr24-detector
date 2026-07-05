# FlightRadar24 Status Detector

*Note: this project is NOT affiliated with FlightRadar24 in ANY way*

This is a script that periodically runs the `fr24feed-status` command to check if your radar is down before an email can be sent from FR24 to you. Whilst the grace periods are enough time to address any issues, sometimes problems can persist still even without you getting an email.

Also because I whipped this up whilst I was on holiday, and trust me, this made detecting network issues and loose connectors a lot easier.

## Dependencies

This program requires the following dependencies:
- Your fr24 feed
- curl

## Installation

There are some pre-compiled binaries available from the releases page, however installation and setting up the service is still manually required.

### Compiling from source

Compiling is recommended if you have Rust installed as it only takes a few seconds

1. Download Rust if you haven't already
2. Clone the repo
```bash
git clone https://github.com/zephyrbash/fr24-detector
cd fr24-detector
```
3. Compile the project
```bash
cargo build --release --verbose
```
4. **(Optional)** move your binary to the local bin folder
```bash
mv target/release/fr24d /usr/local/bin/fr24d
```
5. Create a service file like `/etc/systemd/system/fr24d.service` (unless you want to symlink) with the following content:
```toml
[Unit]
Description=fr24d Background Service
After=network.target
Wants=network.target

[Service]
Type=simple

# Change 'youruser' to the actual user account that should run the programme.
User=youruser
Group=yourgroup

# The absolute path to your compiled binary if you did not move the binary
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
```
6. Reload the service daemon and start the service
```bash
sudo systemctl daemon-reload
sudo systemctl enable fr24d.service
sudo systemctl start fr24d.service
sudo systemctl status fr24d.service
```
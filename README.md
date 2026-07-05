# FlightRadar24 Status Detector

*Note: this project is NOT affiliated with FlightRadar24 in ANY way*

This is a script that periodically runs the `fr24feed-status` command to check if your radar is down before an email can be detected and sent from FR24 to you. Whilst the grace periods are enough time to address any issues, sometimes problems can persist still even without you getting an email.

Also because I whipped this up whilst I was on holiday, and trust me, this made detecting network issues and loose connectors a lot easier

## Installation

There are some pre-compiled binaries available from the releases page, however installation and setting up the service is still manually required.

### Compiling from source

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
4. Create a service file like `fr24d.service` with the following content:
```
```
#!/bin/sh
# Build, install, restart — the ONLY way to ship changes to the live bar.
# (Restarting the service without copying the binary shows stale behavior.)
set -e
cd "$(dirname "$0")"
cargo build --release
sudo systemctl stop mtmr
sudo cp target/release/mtmr /usr/bin/mtmr
sudo systemctl start mtmr
sleep 3
systemctl is-active mtmr.service

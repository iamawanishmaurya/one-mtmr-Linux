#!/bin/bash
# One-time: enable passwordless sudo for Astra (wheel group)
set -e
# safe-write the sudoers drop-in with correct perms
echo 'Astra ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/astra-nopasswd
chmod 440 /etc/sudoers.d/astra-nopasswd
visudo -c
echo "== passwordless sudo enabled for Astra =="
# also finish the pending react-drm path fix
sed -i 's|/home/Astra/react-drm|/home/Astra/opencode/react-drm|g' /etc/systemd/system/react-drm.service
systemctl daemon-reload
systemctl restart react-drm
sleep 6
systemctl is-active react-drm.service
echo "2" > /sys/class/backlight/appletb_backlight/brightness
echo "== react-drm restarted at new path, bar should be live =="

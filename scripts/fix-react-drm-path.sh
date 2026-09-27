#!/bin/bash
# Update react-drm.service to the new install path and restart
sed -i 's|/home/Astra/react-drm|/home/Astra/opencode/react-drm|g' /etc/systemd/system/react-drm.service
systemctl daemon-reload
systemctl restart react-drm
sleep 6
systemctl is-active react-drm.service
journalctl -u react-drm.service -n 3 --no-pager | grep -E "DRM display ready|touch device ready" | tail -1
echo "2" > /sys/class/backlight/appletb_backlight/brightness
echo "DONE - bar should be live"

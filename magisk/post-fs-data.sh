#!/system/bin/sh
# Fix permissions on psyche Goodix FOD, DRM disp_param, and panel backlight nodes
[ -e /dev/goodix_fp ] && chown system:system /dev/goodix_fp && chmod 0660 /dev/goodix_fp
[ -e /sys/class/drm/card0-DSI-1/disp_param ] && chown system:system /sys/class/drm/card0-DSI-1/disp_param && chmod 0660 /sys/class/drm/card0-DSI-1/disp_param
[ -e /sys/class/backlight/panel0-backlight/brightness ] && chown system:system /sys/class/backlight/panel0-backlight/brightness && chmod 0664 /sys/class/backlight/panel0-backlight/brightness

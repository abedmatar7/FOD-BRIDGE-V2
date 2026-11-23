#!/system/bin/sh
MODDIR="${0%/*}"

# Wait for boot completion
while [ "$(getprop sys.boot_completed)" != "1" ]; do
    sleep 1
done

# Ensure Goodix vendor prop is set and restart HIDL 2.1 FP HAL if it initialized as 'none'
if [ "$(getprop persist.vendor.sys.fp.vendor)" != "goodix_fod" ]; then
    setprop persist.vendor.sys.fp.vendor goodix_fod
fi

# Stop umi_aod_bridge by default so it doesn't fight psyche_aodd over panel0-backlight
if [ "$(getprop persist.sys.psyche.keep_umi_aod)" != "1" ]; then
    stop umi_aod_bridge 2>/dev/null
    killall umi_aod_bridge 2>/dev/null
fi

# Start psyche_aodd daemon if binary is present
if [ -x "$MODDIR/system/bin/psyche_aodd" ]; then
    "$MODDIR/system/bin/psyche_aodd" >/dev/null 2>&1 &
fi

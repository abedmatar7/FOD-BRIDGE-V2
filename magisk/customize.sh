#!/system/bin/sh
ui_print "- Installing NEESS ColorOS 17 FOD & AOD Fix for Xiaomi 12X (psyche)"
set_perm_recursive "$MODPATH" 0 0 0755 0644
[ -f "$MODPATH/system/bin/psyche_aodd" ] && set_perm "$MODPATH/system/bin/psyche_aodd" 0 2000 0755
set_perm "$MODPATH/post-fs-data.sh" 0 0 0755
set_perm "$MODPATH/service.sh" 0 0 0755

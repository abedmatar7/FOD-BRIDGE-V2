# NEESS ColorOS 17 FOD & AOD Fix — Xiaomi 12X (`psyche`) Adaptation

Adapted from [NEESCHAL-3/NEESS-COS-AOD-FOD-FIX](https://github.com/NEESCHAL-3/NEESS-COS-AOD-FOD-FIX) (originally targeting POCO X7 Pro / `rodin` / MT6899) for **Xiaomi 12X (`psyche`, SM8250-AC)** running **ColorOS 17 (Android 17 / SDK 37)**.

## What Was Adapted for `psyche`
1. **AOD & FOD HBM Daemon (`aod/` -> `psyche_aodd`)**:
   - Retargeted backlight control from `rodin`'s `/sys/devices/virtual/mi_display/disp_feature/disp-DSI-0/backlight` to `psyche`'s `/sys/class/backlight/panel0-backlight/brightness`.
   - Added SM8250 DRM FOD HBM toggle via `/sys/class/drm/card0-DSI-1/disp_param` (`0x20000` on / `0xE0000` off, toggleable via `persist.sys.psyche.fod_disp_param=0`).
   - Removed the `NEES4` Ed25519 lock and MediaTek `sensors.mt6899.so` path check.
2. **Goodix FOD Properties & Permissions (`magisk/system.prop`, `magisk/post-fs-data.sh`, `fod/zz_psyche_fp_compat.rc`)**:
   - Sets `persist.vendor.sys.fp.vendor=goodix_fod` (fixing the `none` state), `ro.oplus.aod.fod.support=true`, `persist.sys.fp.fod.screenoff=true`, `persist.vendor.sys.fp.fod.lhbmoffafterresult=1`, and `psyche`'s `433,2056` / `540::2163` / `214` FOD geometry.
   - Fixes permissions on `/dev/goodix_fp` and `/sys/class/drm/card0-DSI-1/disp_param` (`0660 system:system`).
   - Stops `umi_aod_bridge` in `service.sh` by default (override with `persist.sys.psyche.keep_umi_aod=1`) so it doesn't fight `psyche_aodd`.
3. **SELinux (`sepolicy/vendor_sepolicy_30.0.cil.append` & `magisk/sepolicy.rule`)**:
   - Updated policy version suffix from `_202404` to `psyche`'s `plat_sepolicy_vers` (`30.0` -> `_30_0`).
4. **Why `librodin_fp_compat.so` & `sensors.mt6899.so` Were Dropped**:
   - `psyche` runs Xiaomi HIDL 2.1 (`android.hardware.biometrics.fingerprint@2.1-service`), not an AIDL `mfp-daemon`, so a `libbinder_ndk` `DT_NEEDED` shim cannot attach to it.
   - `sensors.mt6899.so` is MediaTek-only; `psyche` uses Qualcomm SSC multihal sensors.

## How to Build
1. Push this directory to a GitHub repository and run the **Build NEESS ColorOS 17 Psyche FOD & AOD Magisk Module** workflow.
2. Extract the downloaded GitHub Actions artifact and flash `NEESS-COS17-Psyche-FOD-AOD-Fix-Magisk.zip` in Magisk/KernelSU.

## Note on AIDL Fingerprint HAL (If HIDL 2.1 Still Won't Enroll/Auth)
If ColorOS 17's `FingerprintService` refuses to enroll or authenticate through Xiaomi's HIDL 2.1 HAL even with `persist.vendor.sys.fp.vendor=goodix_fod`, pair this module with an AIDL `IFingerprint/default` bridge (`hoshikv/mfp-daemon-rs` + `hoshikv/MiOplusFingerprint-rs`).

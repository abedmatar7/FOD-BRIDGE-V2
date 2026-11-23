mod panel_writes;
mod power_state;

use panel_writes::PanelWrites;
use power_state::PowerState;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

// Xiaomi 12X (psyche, SM8250-AC, 4.19 kernel) verified sysfs nodes
const BL_PATH: &str = "/sys/class/backlight/panel0-backlight/brightness";
const DISP_PARAM_PATH: &str = "/sys/class/drm/card0-DSI-1/disp_param";

// Default backlight levels (overridable via persist.sys.psyche.aod_bl / fp_bl)
const DEFAULT_AOD_BL: u32 = 28;
const DEFAULT_FP_BL: u32 = 100;

// Xiaomi SM8250 DRM disp_param values for FOD HBM control
const DISP_PARAM_FOD_HBM_ON: &str = "0x20000\n";
const DISP_PARAM_FOD_HBM_OFF: &str = "0xE0000\n";

static WRITES: PanelWrites = PanelWrites::new();

fn setprop(key: &str, val: &str) {
    let _ = Command::new("/system/bin/setprop")
        .args([key, val])
        .status();
}

fn getprop(key: &str) -> String {
    Command::new("/system/bin/getprop")
        .arg(key)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn getprop_u32(key: &str, fallback: u32) -> u32 {
    getprop(key).parse::<u32>().unwrap_or(fallback)
}

fn disp_param_enabled() -> bool {
    let v = getprop("persist.sys.psyche.fod_disp_param");
    v.is_empty() || v == "1" || v.eq_ignore_ascii_case("true")
}

fn set_fod_hbm(gen: u64, enable: bool) {
    if !disp_param_enabled() || !Path::new(DISP_PARAM_PATH).exists() {
        return;
    }
    let val = if enable {
        DISP_PARAM_FOD_HBM_ON
    } else {
        DISP_PARAM_FOD_HBM_OFF
    };
    let _ = WRITES.apply(gen, DISP_PARAM_PATH, val);
}

fn wait_for_backlight_node() {
    for _ in 0..60 {
        if Path::new(BL_PATH).exists() {
            return;
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    setprop("ro.nees.fod.compat", "1");
    setprop("persist.sys.psyche.aod_keep_doze", "1");

    wait_for_backlight_node();

    let aod_bl = getprop_u32("persist.sys.psyche.aod_bl", DEFAULT_AOD_BL);
    let fp_bl = getprop_u32("persist.sys.psyche.fp_bl", DEFAULT_FP_BL);

    let mut current_state = PowerState::Unknown;

    loop {
        let mut child = match Command::new("/system/bin/logcat")
            .args(["-b", "all", "-v", "brief", "-T", "1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line_res in reader.lines() {
                let line = match line_res {
                    Ok(l) => l,
                    Err(_) => break,
                };

                // 1. Track PowerState transitions (Awake / Dozing / Asleep)
                if let Some(new_state) = PowerState::parse_from_log_line(&line) {
                    if new_state != current_state {
                        current_state = new_state;
                        let gen = WRITES.invalidate();
                        match current_state {
                            PowerState::Dozing => {
                                let bl_str = format!("{}\n", aod_bl);
                                let _ = WRITES.apply(gen, BL_PATH, &bl_str);
                            }
                            PowerState::Awake | PowerState::Asleep => {
                                // Ensure FOD HBM is cleared when leaving doze/auth
                                set_fod_hbm(gen, false);
                            }
                            PowerState::Unknown => {}
                        }
                    }
                }

                // 2. FOD touch / LHBM on events in ColorOS 17
                if line.contains("showFingerprintIcon")
                    || line.contains("onTouchDown")
                    || line.contains("OPLUS_TOUCH_DOWN")
                    || line.contains("fp_hbm_on")
                {
                    let gen = WRITES.invalidate();
                    if current_state == PowerState::Dozing {
                        let bl_str = format!("{}\n", fp_bl);
                        let _ = WRITES.apply(gen, BL_PATH, &bl_str);
                    }
                    set_fod_hbm(gen, true);
                }

                // 3. Instant LHBM turn-off on auth result / hide icon / cancel
                if line.contains("hideFingerprintIcon")
                    || line.contains("onTouchUp")
                    || line.contains("OPLUS_TOUCH_UP")
                    || line.contains("onAuthenticated")
                    || line.contains("fp_hbm_off")
                {
                    let gen = WRITES.invalidate();
                    set_fod_hbm(gen, false);
                    if current_state == PowerState::Dozing {
                        let bl_str = format!("{}\n", aod_bl);
                        let _ = WRITES.apply(gen, BL_PATH, &bl_str);
                    }
                }
            }
        }

        let _ = child.wait();
        thread::sleep(Duration::from_secs(1));
    }
}

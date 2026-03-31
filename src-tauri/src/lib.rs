use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use tauri::{Emitter, Manager};

static MOBILE_GYRO_ACTIVE: AtomicBool = AtomicBool::new(false);

type SimState = Arc<Mutex<HashMap<String, f64>>>;

#[derive(Serialize, Deserialize, Clone)]
struct AppSettings {
    mode: String,           // "strict" or "smart"
    auto_check: String,     // "on" or "off"
    check_feedback: String, // "on" or "off"
    haptic: String,         // "on" or "off"
    sound: String,          // "on" or "off"
    check_method: String,   // "tap" or "swipe"
    auto_advance: String,   // "on" or "off"
    data_mode: String,      // "live" or "manual"
    web_theme: String,      // "dark" or "light"
    active_checklist: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            mode: "smart".into(),
            auto_check: "on".into(),
            check_feedback: "on".into(),
            haptic: "on".into(),
            sound: "on".into(),
            check_method: "tap".into(),
            auto_advance: "on".into(),
            data_mode: "live".into(),
            web_theme: "dark".into(),
            active_checklist: "cessna-172".into(),
        }
    }
}

type SettingsState = Arc<Mutex<AppSettings>>;
// Wrapper types so Tauri can distinguish them
struct CaptureData(String);
struct OverlayData(String);
type CaptureState = Arc<Mutex<CaptureData>>;
type OverlayState = Arc<Mutex<OverlayData>>;
type ChecklistProgress = Arc<Mutex<HashMap<String, bool>>>; // item_id → checked

#[derive(Serialize, Deserialize, Clone)]
struct GaugeData {
    airspeed: f64,
    altitude: f64,
    heading: f64,
    pitch: f64,
    roll: f64,
    vs: f64,
    rpm: f64,
    turn_rate: f64,
    slip: f64,
    baro: f64,
    nav1_obs: f64,
    nav1_cdi: f64,
    nav1_gs: f64,
    nav1_to_from: String,
    nav2_obs: f64,
    nav2_cdi: f64,
    // Electrical
    master_battery: f64,
    master_alt: f64,
    fuel_pump: f64,
    beacon: f64,
    landing_light: f64,
    taxi_light: f64,
    nav_light: f64,
    strobe: f64,
    pitot_heat: f64,
    avionics_master: f64,
    throttle: f64,
    mixture: f64,
    carb_heat: f64,
    flap_ratio: f64,
    elevator_trim: f64,
    fuel_selector: f64,
    magneto_key: f64,
    connected: bool,
}

#[tauri::command]
fn get_gauge_data(state: tauri::State<SimState>) -> GaugeData {
    let s = state.lock().unwrap();
    GaugeData {
        airspeed: *s.get("AIRSPEED").unwrap_or(&0.0),
        altitude: *s.get("ALTITUDE").unwrap_or(&0.0),
        heading: *s.get("HEADING").unwrap_or(&0.0),
        pitch: *s.get("PITCH").unwrap_or(&0.0),
        roll: *s.get("ROLL").unwrap_or(&0.0),
        vs: *s.get("VS").unwrap_or(&0.0),
        rpm: *s.get("RPM").unwrap_or(&0.0),
        turn_rate: *s.get("TURN_RATE").unwrap_or(&0.0),
        slip: *s.get("SLIP").unwrap_or(&0.0),
        baro: *s.get("BARO").unwrap_or(&29.92),
        nav1_obs: *s.get("NAV1_OBS").unwrap_or(&0.0),
        nav1_cdi: *s.get("NAV1_CDI").unwrap_or(&0.0),
        nav1_gs: *s.get("NAV1_GS").unwrap_or(&0.0),
        nav1_to_from: if *s.get("NAV1_TOFROM").unwrap_or(&0.0) > 0.5 { "TO".into() } else { "FROM".into() },
        nav2_obs: *s.get("NAV2_OBS").unwrap_or(&0.0),
        nav2_cdi: *s.get("NAV2_CDI").unwrap_or(&0.0),
        master_battery: *s.get("MASTER_BATTERY").unwrap_or(&0.0),
        master_alt: *s.get("MASTER_ALT").unwrap_or(&0.0),
        fuel_pump: *s.get("FUEL_PUMP").unwrap_or(&0.0),
        beacon: *s.get("BEACON").unwrap_or(&0.0),
        landing_light: *s.get("LANDING_LIGHT").unwrap_or(&0.0),
        taxi_light: *s.get("TAXI_LIGHT").unwrap_or(&0.0),
        nav_light: *s.get("NAV_LIGHT").unwrap_or(&0.0),
        strobe: *s.get("STROBE").unwrap_or(&0.0),
        pitot_heat: *s.get("PITOT_HEAT").unwrap_or(&0.0),
        avionics_master: *s.get("AVIONICS_MASTER").unwrap_or(&0.0),
        throttle: *s.get("THROTTLE").unwrap_or(&0.0),
        mixture: *s.get("MIXTURE").unwrap_or(&0.0),
        carb_heat: *s.get("CARB_HEAT").unwrap_or(&0.0),
        flap_ratio: *s.get("FLAPS").unwrap_or(&0.0),
        elevator_trim: *s.get("TRIM").unwrap_or(&0.0),
        fuel_selector: *s.get("FUEL_SEL").unwrap_or(&0.0),
        magneto_key: *s.get("MAGNETO").unwrap_or(&0.0),
        connected: *s.get("_CONNECTED").unwrap_or(&0.0) > 0.5,
    }
}

#[tauri::command]
fn send_command(cmd: String, value: f64) -> Result<(), String> {
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;

    // Use DREF set (direct value) instead of CMND toggle
    let dataref = match cmd.as_str() {
        "MASTER_BATTERY" => "sim/cockpit/electrical/battery_on",
        "MASTER_ALT" => "sim/cockpit/electrical/generator_on[0]",
        "FUEL_PUMP" => "sim/cockpit/engine/fuel_pump_on[0]",
        "BEACON" => "sim/cockpit/electrical/beacon_lights_on",
        "LANDING_LIGHT" => "sim/cockpit/electrical/landing_lights_on",
        "TAXI_LIGHT" => "sim/cockpit/electrical/taxi_light_on",
        "NAV_LIGHT" => "sim/cockpit/electrical/nav_lights_on",
        "STROBE" => "sim/cockpit/electrical/strobe_lights_on",
        "PITOT_HEAT" => "sim/cockpit/switches/pitot_heat_on",
        "AVIONICS_MASTER" => "sim/cockpit2/switches/avionics_power_on",
        "MAGNETO" => {
            if value >= 3.5 {
                // START: set ignition_key[0] = 4 continuously for 3 sec
                for _ in 0..30 {
                    let mut pkt = Vec::with_capacity(509);
                    pkt.extend_from_slice(b"DREF\0");
                    pkt.extend_from_slice(&4.0f32.to_le_bytes());
                    pkt.extend_from_slice(b"sim/cockpit2/engine/actuators/ignition_key[0]");
                    pkt.resize(509, 0);
                    sock.send_to(&pkt, "127.0.0.1:49000").map_err(|e| e.to_string())?;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                // Release back to BOTH (3)
                let mut pkt = Vec::with_capacity(509);
                pkt.extend_from_slice(b"DREF\0");
                pkt.extend_from_slice(&3.0f32.to_le_bytes());
                pkt.extend_from_slice(b"sim/cockpit2/engine/actuators/ignition_key[0]");
                pkt.resize(509, 0);
                sock.send_to(&pkt, "127.0.0.1:49000").map_err(|e| e.to_string())?;
                return Ok(());
            }
            // Normal magneto positions use key commands
            let mag_cmd = match value as i32 {
                0 => "sim/magnetos/magnetos_off_1",
                1 => "sim/magnetos/magnetos_right_1",
                2 => "sim/magnetos/magnetos_left_1",
                3 => "sim/magnetos/magnetos_both_1",
                _ => "sim/magnetos/magnetos_both_1",
            };
            let mut pkt = Vec::with_capacity(509);
            pkt.extend_from_slice(b"CMND\0");
            pkt.extend_from_slice(mag_cmd.as_bytes());
            pkt.resize(509, 0);
            sock.send_to(&pkt, "127.0.0.1:49000").map_err(|e| e.to_string())?;
            return Ok(());
        },
        "FUEL_SEL" => {
            // X-Plane 11 Cessna 172: 0=OFF, 1=LEFT, 3=RIGHT, 4=BOTH
            let fuel_val = match value as i32 {
                0 => 0.0f32,  // OFF
                1 => 1.0f32,  // LEFT
                2 => 4.0f32,  // BOTH
                3 => 3.0f32,  // RIGHT
                _ => 4.0f32,
            };
            // Key: must use [0] index!
            let mut pkt = Vec::with_capacity(509);
            pkt.extend_from_slice(b"DREF\0");
            pkt.extend_from_slice(&fuel_val.to_le_bytes());
            pkt.extend_from_slice(b"sim/cockpit2/fuel/fuel_tank_selector[0]");
            pkt.resize(509, 0);
            sock.send_to(&pkt, "127.0.0.1:49000").map_err(|e| e.to_string())?;
            return Ok(());
        },
        // These work as DREF (continuous values)
        "THROTTLE" | "MIXTURE" | "CARB_HEAT" | "FLAPS" | "TRIM"
        | "AILERON" | "ELEVATOR" | "RUDDER" | "PARKING_BRAKE" => {
            let dataref = match cmd.as_str() {
                "THROTTLE" => "sim/cockpit2/engine/actuators/throttle_ratio[0]",
                "MIXTURE" => "sim/cockpit2/engine/actuators/mixture_ratio[0]",
                "CARB_HEAT" => "sim/cockpit2/engine/actuators/carb_heat_ratio[0]",
                "FLAPS" => "sim/cockpit2/controls/flap_ratio",
                "TRIM" => "sim/cockpit2/controls/elevator_trim",
                "AILERON" => "sim/cockpit2/controls/yoke_roll_ratio",
                "ELEVATOR" => "sim/cockpit2/controls/yoke_pitch_ratio",
                "RUDDER" => "sim/cockpit2/controls/rudder_ratio",
                "PARKING_BRAKE" => "sim/cockpit2/controls/parking_brake_ratio",
                _ => unreachable!(),
            };
            let mut pkt = Vec::with_capacity(509);
            pkt.extend_from_slice(b"DREF\0");
            pkt.extend_from_slice(&(value as f32).to_le_bytes());
            pkt.extend_from_slice(dataref.as_bytes());
            pkt.resize(509, 0);
            sock.send_to(&pkt, "127.0.0.1:49000").map_err(|e| e.to_string())?;
            return Ok(());
        },
        "PAUSE" => {
            // X-Plane toggle pause via CMND
            let mut pkt = Vec::with_capacity(509);
            pkt.extend_from_slice(b"CMND\0");
            pkt.extend_from_slice(b"sim/operation/pause_toggle");
            pkt.resize(509, 0);
            sock.send_to(&pkt, "127.0.0.1:49000").map_err(|e| e.to_string())?;
            return Ok(());
        },
        _ => return Err(format!("Unknown command: {}", cmd)),
    };

    // Send DREF packet (set value directly)
    let mut pkt = Vec::with_capacity(509);
    pkt.extend_from_slice(b"DREF\0");
    pkt.extend_from_slice(&(value as f32).to_le_bytes());
    pkt.extend_from_slice(dataref.as_bytes());
    pkt.resize(509, 0);
    sock.send_to(&pkt, "127.0.0.1:49000").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_local_ip() -> String {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".into())
}

#[tauri::command]
fn get_settings(settings: tauri::State<SettingsState>) -> AppSettings {
    settings.lock().unwrap().clone()
}

#[tauri::command]
fn set_settings(settings: tauri::State<SettingsState>, new_settings: AppSettings) {
    *settings.lock().unwrap() = new_settings;
}

#[tauri::command]
fn capture_screen_region(x: i32, y: i32, w: i32, h: i32) -> Result<String, String> {
    use winapi::um::winuser::GetDC;
    use winapi::um::wingdi::{CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, GetDIBits, DeleteDC, DeleteObject, SRCCOPY, BITMAPINFOHEADER, BI_RGB, BITMAPINFO, DIB_RGB_COLORS};

    if w <= 0 || h <= 0 { return Err("Invalid region".into()); }

    unsafe {
        let hdc_screen = GetDC(std::ptr::null_mut()); // entire screen
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm = CreateCompatibleBitmap(hdc_screen, w, h);
        SelectObject(hdc_mem, hbm as _);

        BitBlt(hdc_mem, 0, 0, w, h, hdc_screen, x, y, SRCCOPY);

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w, biHeight: -h, biPlanes: 1, biBitCount: 24,
                biCompression: BI_RGB, biSizeImage: 0,
                biXPelsPerMeter: 0, biYPelsPerMeter: 0, biClrUsed: 0, biClrImportant: 0,
            },
            bmiColors: [std::mem::zeroed()],
        };

        let row_size = ((w * 3 + 3) / 4 * 4) as usize;
        let mut pixels = vec![0u8; row_size * h as usize];
        GetDIBits(hdc_mem, hbm, 0, h as u32, pixels.as_mut_ptr() as _, &mut bmi, DIB_RGB_COLORS);

        let file_size = 54 + pixels.len();
        let mut bmp = Vec::with_capacity(file_size);
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
        bmp.extend_from_slice(&0u32.to_le_bytes());
        bmp.extend_from_slice(&54u32.to_le_bytes());
        bmp.extend_from_slice(&40u32.to_le_bytes());
        bmp.extend_from_slice(&w.to_le_bytes());
        bmp.extend_from_slice(&(-h).to_le_bytes());
        bmp.extend_from_slice(&1u16.to_le_bytes());
        bmp.extend_from_slice(&24u16.to_le_bytes());
        bmp.extend_from_slice(&0u32.to_le_bytes());
        bmp.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
        bmp.extend_from_slice(&[0u8; 16]);
        bmp.extend_from_slice(&pixels);

        DeleteObject(hbm as _);
        DeleteDC(hdc_mem);
        winapi::um::winuser::ReleaseDC(std::ptr::null_mut(), hdc_screen);

        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp);
        Ok(format!("data:image/bmp;base64,{}", b64))
    }
}

#[tauri::command]
fn store_capture(capture: tauri::State<CaptureState>, data: String) {
    capture.lock().unwrap().0 = data;
}

#[tauri::command]
fn store_overlay(overlay_state: tauri::State<OverlayState>, data: String) {
    overlay_state.lock().unwrap().0 = data;
}

#[tauri::command]
fn list_presets() -> Vec<String> {
    let dirs = vec![
        std::path::PathBuf::from("../public/presets"),
        std::path::PathBuf::from("public/presets"),
    ];
    let mut presets: Vec<String> = Vec::new();
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if !presets.contains(&name.to_string()) {
                            presets.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    presets
}

#[tauri::command]
fn load_preset(name: String) -> Result<String, String> {
    let candidates = vec![
        std::path::PathBuf::from(format!("../public/presets/{}.json", name)),
        std::path::PathBuf::from(format!("public/presets/{}.json", name)),
    ];
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Ok(content);
        }
    }
    Err(format!("Preset '{}' not found", name))
}

#[tauri::command]
fn save_preset(name: String, data: String) -> Result<(), String> {
    // Validate JSON
    serde_json::from_str::<serde_json::Value>(&data)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    let dirs = vec![
        std::path::PathBuf::from("../public/presets"),
        std::path::PathBuf::from("public/presets"),
    ];
    for dir in &dirs {
        if dir.exists() {
            let path = dir.join(format!("{}.json", name));
            return std::fs::write(&path, &data)
                .map_err(|e| format!("Failed to write: {}", e));
        }
    }
    Err("No presets directory found".into())
}

#[tauri::command]
async fn check_for_updates() -> Result<serde_json::Value, String> {
    let current = env!("CARGO_PKG_VERSION");

    // Fetch latest release from GitHub
    let url = "https://api.github.com/repos/CockpitFlow/cockpitflow/releases/latest";

    let client = std::thread::spawn(move || -> Result<String, String> {
        let resp = ureq_minimal_get(url)?;
        Ok(resp)
    });

    match client.join() {
        Ok(Ok(body)) => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                let latest = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v');
                let update_available = version_newer(latest, current);
                let download_url = json["html_url"].as_str().unwrap_or("").to_string();
                let changelog = json["body"].as_str().unwrap_or("").to_string();

                // Find .msi or .exe asset for auto-update
                // Prefer .exe (NSIS, in-place upgrade) over .msi (requires uninstall)
                let installer_url = json["assets"].as_array()
                    .and_then(|assets| {
                        // First try .exe
                        assets.iter().find_map(|a| {
                            let name = a["name"].as_str().unwrap_or("");
                            if name.ends_with("-setup.exe") { a["browser_download_url"].as_str().map(|s| s.to_string()) } else { None }
                        }).or_else(|| {
                            // Fallback to .msi
                            assets.iter().find_map(|a| {
                                let name = a["name"].as_str().unwrap_or("");
                                if name.ends_with(".msi") { a["browser_download_url"].as_str().map(|s| s.to_string()) } else { None }
                            })
                        })
                    })
                    .unwrap_or_default();

                Ok(serde_json::json!({
                    "current": current,
                    "latest": latest,
                    "update_available": update_available,
                    "download_url": download_url,
                    "installer_url": installer_url,
                    "changelog": changelog,
                }))
            } else {
                Ok(serde_json::json!({
                    "current": current,
                    "latest": current,
                    "update_available": false,
                }))
            }
        }
        _ => Ok(serde_json::json!({
            "current": current,
            "latest": current,
            "update_available": false,
        }))
    }
}

#[tauri::command]
async fn download_and_install_update(url: String) -> Result<String, String> {
    if url.is_empty() { return Err("No installer URL".into()); }

    let temp_dir = std::env::temp_dir();
    let filename = url.split('/').last().unwrap_or("CockpitFlow-update.exe");
    let filepath = temp_dir.join(filename);
    let filepath_str = filepath.to_string_lossy().to_string();

    // Download silently using curl (built-in Windows 10/11, no window)
    let download = std::process::Command::new("curl")
        .args(["-L", "-s", "-o", &filepath_str, &url])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Download failed: {}", e))?;

    if !download.status.success() {
        return Err("Download failed".into());
    }

    // Launch NSIS installer (overwrites in-place, no uninstall needed)
    std::process::Command::new(&filepath_str)
        .args(["/S"]) // Silent install
        .spawn()
        .map_err(|e| format!("Install failed: {}", e))?;

    Ok("Installing...".into())
}

/// Simple HTTP GET without external deps
fn ureq_minimal_get(url: &str) -> Result<String, String> {
    let url = url.strip_prefix("https://").ok_or("Not HTTPS")?;
    let (host, path) = url.split_once('/').ok_or("Bad URL")?;

    // Connect with TLS via native-tls or rustls — but we don't have those deps.
    // Simpler: shell out to curl/powershell
    let output = std::process::Command::new("powershell")
        .args(["-Command", &format!(
            "(Invoke-WebRequest -Uri 'https://{}' -UseBasicParsing -Headers @{{'User-Agent'='CockpitFlow'}}).Content",
            format!("{}/{}", host, path)
        )])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    } else {
        Err("HTTP request failed".into())
    }
}

/// Compare semver: is `latest` newer than `current`?
fn version_newer(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|s| s.parse().ok()).collect()
    };
    let l = parse(latest);
    let c = parse(current);
    for i in 0..3 {
        let lv = l.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if lv > cv { return true; }
        if lv < cv { return false; }
    }
    false
}

fn module_base_dirs() -> Vec<std::path::PathBuf> {
    let cwd = std::env::current_dir().unwrap_or_default();
    let exe_dir = std::env::current_exe().ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    vec![
        std::path::PathBuf::from("../public/modules"),
        std::path::PathBuf::from("../dist/modules"),
        std::path::PathBuf::from("public/modules"),
        std::path::PathBuf::from("dist/modules"),
        cwd.join("public/modules"),
        cwd.join("dist/modules"),
        cwd.join("src-tauri/../public/modules"),
        exe_dir.join("../../public/modules"),
        exe_dir.join("../../../public/modules"),
    ]
}

#[tauri::command]
fn list_modules() -> Vec<serde_json::Value> {
    let dirs = module_base_dirs();
    let mut modules = Vec::new();
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest = path.join("module.json");
                    if let Ok(content) = std::fs::read_to_string(&manifest) {
                        if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                            // Add presets list
                            let presets_dir = path.join("presets");
                            let mut presets: Vec<String> = Vec::new();
                            if let Ok(pents) = std::fs::read_dir(&presets_dir) {
                                for pe in pents.flatten() {
                                    if pe.path().extension().map_or(false, |e| e == "json") {
                                        if let Some(name) = pe.path().file_stem().and_then(|s| s.to_str()) {
                                            presets.push(name.to_string());
                                        }
                                    }
                                }
                            }
                            parsed["presets"] = serde_json::json!(presets);
                            modules.push(parsed);
                        }
                    }
                }
            }
        }
        if !modules.is_empty() { break; }
    }
    modules
}

#[tauri::command]
fn update_module(module_id: String, field: String, value: serde_json::Value) -> Result<(), String> {
    let paths: Vec<std::path::PathBuf> = module_base_dirs().iter()
        .map(|d| d.join(&module_id).join("module.json"))
        .collect();
    let mut updated = false;
    let mut new_content = String::new();
    // Read from first existing file
    for path in &paths {
        if path.exists() && new_content.is_empty() {
            let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            let mut parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            parsed[&field] = value.clone();
            new_content = serde_json::to_string_pretty(&parsed).unwrap();
        }
    }
    if new_content.is_empty() {
        return Err(format!("Module '{}' not found", module_id));
    }
    // Write to ALL existing paths
    for path in &paths {
        if path.exists() {
            let _ = std::fs::write(path, &new_content);
            updated = true;
        }
    }
    if updated { Ok(()) } else { Err(format!("Could not write module '{}'", module_id)) }
}

#[tauri::command]
fn load_module_preset(module_id: String, preset_id: String) -> Result<String, String> {
    let candidates: Vec<std::path::PathBuf> = module_base_dirs().iter()
        .map(|d| d.join(&module_id).join("presets").join(format!("{}.json", preset_id)))
        .collect();
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Ok(content);
        }
    }
    Err(format!("Preset '{}' not found in module '{}'", preset_id, module_id))
}

#[tauri::command]
fn save_module_preset(module_id: String, preset_id: String, data: String) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(&data).map_err(|e| format!("Invalid JSON: {}", e))?;
    let dirs: Vec<std::path::PathBuf> = module_base_dirs().iter()
        .map(|d| d.join(&module_id).join("presets"))
        .collect();
    for dir in &dirs {
        if dir.exists() {
            let path = dir.join(format!("{}.json", preset_id));
            return std::fs::write(&path, &data).map_err(|e| e.to_string());
        }
    }
    Err("Presets directory not found".into())
}

#[tauri::command]
fn delete_module_preset(module_id: String, preset_id: String) -> Result<(), String> {
    let candidates: Vec<std::path::PathBuf> = module_base_dirs().iter()
        .map(|d| d.join(&module_id).join("presets").join(format!("{}.json", preset_id)))
        .collect();
    for path in &candidates {
        if path.exists() {
            return std::fs::remove_file(path).map_err(|e| e.to_string());
        }
    }
    Err("Preset not found".into())
}

fn handle_request(
    request: &mut tiny_http::Request,
    sim_state: &SimState,
    settings: &SettingsState,
    capture: &CaptureState,
    overlay: &OverlayState,
    checklist_progress: &ChecklistProgress,
    dist_dir: &std::path::Path,
) -> tiny_http::ResponseBox {
    let full_url = request.url().to_string();
    let url = full_url.split('?').next().unwrap_or(&full_url).to_string();
    match url.as_str() {
        "/api/sim-state" => {
            let s = sim_state.lock().unwrap();
            let connected = *s.get("_CONNECTED").unwrap_or(&0.0) > 0.5;
            let values: HashMap<String, f64> = s.iter()
                .filter(|(k, _)| !k.starts_with('_'))
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            let json = serde_json::json!({ "connected": connected, "values": values });
            tiny_http::Response::from_string(json.to_string())
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/settings" => {
            let s = settings.lock().unwrap();
            let json = serde_json::to_string(&*s).unwrap_or_default();
            tiny_http::Response::from_string(json)
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/app-config" => {
            // Dynamic: scan modules/ directory for module.json manifests
            let module_dirs = vec![
                dist_dir.join("modules"),
                dist_dir.join("../public/modules"),
                std::path::PathBuf::from("../public/modules"),
                std::path::PathBuf::from("public/modules"),
            ];
            let mut modules: Vec<serde_json::Value> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for mdir in &module_dirs {
                if let Ok(entries) = std::fs::read_dir(mdir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let manifest = path.join("module.json");
                            if manifest.exists() {
                                if let Ok(content) = std::fs::read_to_string(&manifest) {
                                    if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                                        let id = parsed.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                                        if !id.is_empty() && enabled && seen.insert(id.clone()) {
                                            // List presets for this module
                                            let presets_dir = path.join("presets");
                                            let mut presets: Vec<String> = Vec::new();
                                            if let Ok(pents) = std::fs::read_dir(&presets_dir) {
                                                for pe in pents.flatten() {
                                                    if pe.path().extension().map_or(false, |e| e == "json") {
                                                        if let Some(name) = pe.path().file_stem().and_then(|s| s.to_str()) {
                                                            presets.push(name.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                            parsed["presets"] = serde_json::json!(presets);
                                            // Build renderer URL
                                            parsed["renderer_url"] = serde_json::json!(format!("/modules/{}/renderer.html", id));
                                            modules.push(parsed);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !modules.is_empty() { break; }
            }
            let json = serde_json::json!({
                "app": "CockpitFlow",
                "version": "1.0.0",
                "modules": modules
            });
            tiny_http::Response::from_string(json.to_string())
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/checklists" => {
            let dirs = vec![
                dist_dir.join("checklists"),
                dist_dir.join("../public/checklists"),
                dist_dir.join("../checklists"),
                std::path::PathBuf::from("../public/checklists"),
                std::path::PathBuf::from("checklists"),
            ];
            let mut lists: Vec<serde_json::Value> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for dir in &dirs {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "json") {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                if seen.insert(name.to_string()) {
                                    // Read name and metadata from JSON
                                    let mut info = serde_json::json!({ "id": name });
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                                            if let Some(n) = parsed.get("name").and_then(|v| v.as_str()) { info["name"] = serde_json::json!(n); }
                                            if let Some(a) = parsed.get("author").and_then(|v| v.as_str()) { info["author"] = serde_json::json!(a); }
                                            if let Some(c) = parsed.get("category").and_then(|v| v.as_str()) { info["category"] = serde_json::json!(c); }
                                            if let Some(phases) = parsed.get("phases").and_then(|v| v.as_array()) {
                                                let items: usize = phases.iter().filter_map(|p| p.get("items").and_then(|i| i.as_array()).map(|a| a.len())).sum();
                                                info["phases"] = serde_json::json!(phases.len());
                                                info["items"] = serde_json::json!(items);
                                            }
                                        }
                                    }
                                    lists.push(info);
                                }
                            }
                        }
                    }
                }
            }
            tiny_http::Response::from_string(serde_json::json!(lists).to_string())
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        _ if url.starts_with("/api/checklist/") => {
            let name = url.trim_start_matches("/api/checklist/");
            if *request.method() == tiny_http::Method::Get {
                let candidates = vec![
                    dist_dir.join(format!("checklists/{}.json", name)),
                    dist_dir.join(format!("../public/checklists/{}.json", name)),
                    dist_dir.join(format!("../checklists/{}.json", name)),
                    std::path::PathBuf::from(format!("../public/checklists/{}.json", name)),
                    std::path::PathBuf::from(format!("checklists/{}.json", name)),
                ];
                match candidates.iter().find_map(|p| std::fs::read(p).ok()) {
                    Some(data) => tiny_http::Response::from_data(data)
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                        .boxed(),
                    None => tiny_http::Response::from_string(format!("{{\"error\":\"Checklist '{}' not found\"}}", name))
                        .with_status_code(404)
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .boxed(),
                }
            } else if *request.method() == tiny_http::Method::Post {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).ok();
                if serde_json::from_str::<serde_json::Value>(&body).is_ok() {
                    let save_dirs = vec![
                        dist_dir.join("checklists"),
                        dist_dir.join("../public/checklists"),
                        dist_dir.join("../checklists"),
                        std::path::PathBuf::from("../public/checklists"),
                        std::path::PathBuf::from("checklists"),
                    ];
                    let mut saved = false;
                    for dir in &save_dirs {
                        if dir.exists() {
                            let path = dir.join(format!("{}.json", name));
                            if std::fs::write(&path, &body).is_ok() {
                                saved = true;
                                eprintln!("[CHECKLIST] Saved '{}' to {:?}", name, path);
                                break;
                            }
                        }
                    }
                    tiny_http::Response::from_string(if saved { "{\"ok\":true}" } else { "{\"error\":\"Could not save\"}" })
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                        .boxed()
                } else {
                    tiny_http::Response::from_string("{\"error\":\"Invalid JSON\"}")
                        .with_status_code(400)
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .boxed()
                }
            } else {
                tiny_http::Response::from_string("{\"error\":\"Method not allowed\"}")
                    .with_status_code(405)
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .boxed()
            }
        },
        // Generic module preset: /api/module/{module_id}/preset/{preset_id}
        _ if url.starts_with("/api/module/") && url.contains("/preset/") => {
            let parts: Vec<&str> = url.trim_start_matches("/api/module/").splitn(3, '/').collect();
            if parts.len() >= 3 && parts[1] == "preset" {
                let module_id = parts[0];
                let preset_id = parts[2];
                let candidates = vec![
                    dist_dir.join(format!("modules/{}/presets/{}.json", module_id, preset_id)),
                    dist_dir.join(format!("../public/modules/{}/presets/{}.json", module_id, preset_id)),
                    std::path::PathBuf::from(format!("../public/modules/{}/presets/{}.json", module_id, preset_id)),
                    std::path::PathBuf::from(format!("public/modules/{}/presets/{}.json", module_id, preset_id)),
                ];
                if *request.method() == tiny_http::Method::Get {
                    match candidates.iter().find_map(|p| std::fs::read(p).ok()) {
                        Some(data) => tiny_http::Response::from_data(data)
                            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                            .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                            .boxed(),
                        None => tiny_http::Response::from_string("{\"error\":\"Not found\"}")
                            .with_status_code(404)
                            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                            .boxed(),
                    }
                } else if *request.method() == tiny_http::Method::Post {
                    let mut body = String::new();
                    request.as_reader().read_to_string(&mut body).ok();
                    if serde_json::from_str::<serde_json::Value>(&body).is_ok() {
                        let save_dirs = vec![
                            dist_dir.join(format!("modules/{}/presets", module_id)),
                            dist_dir.join(format!("../public/modules/{}/presets", module_id)),
                            std::path::PathBuf::from(format!("../public/modules/{}/presets", module_id)),
                            std::path::PathBuf::from(format!("public/modules/{}/presets", module_id)),
                        ];
                        let mut saved = false;
                        for dir in &save_dirs {
                            if dir.exists() {
                                let path = dir.join(format!("{}.json", preset_id));
                                if std::fs::write(&path, &body).is_ok() { saved = true; break; }
                            }
                        }
                        tiny_http::Response::from_string(if saved { "{\"ok\":true}" } else { "{\"error\":\"Could not save\"}" })
                            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                            .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                            .boxed()
                    } else {
                        tiny_http::Response::from_string("{\"error\":\"Invalid JSON\"}")
                            .with_status_code(400)
                            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                            .boxed()
                    }
                } else {
                    tiny_http::Response::from_string("{\"error\":\"Method not allowed\"}")
                        .with_status_code(405)
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .boxed()
                }
            } else {
                tiny_http::Response::from_string("{\"error\":\"Bad request\"}")
                    .with_status_code(400)
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .boxed()
            }
        },
        "/api/checklist-progress" => {
            if *request.method() == tiny_http::Method::Get {
                let progress = checklist_progress.lock().unwrap();
                let json = serde_json::json!(&*progress);
                tiny_http::Response::from_string(json.to_string())
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                    .boxed()
            } else if *request.method() == tiny_http::Method::Post {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).ok();
                if let Ok(data) = serde_json::from_str::<HashMap<String, bool>>(&body) {
                    let mut progress = checklist_progress.lock().unwrap();
                    *progress = data;
                }
                tiny_http::Response::from_string("{\"ok\":true}")
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                    .boxed()
            } else {
                tiny_http::Response::from_string("{}")
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .boxed()
            }
        },
        "/api/checklist-check" => {
            // Single item check/uncheck: POST {"id":"master-switch","checked":true}
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).ok();
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                let id = data["id"].as_str().unwrap_or("").to_string();
                let is_checked = data["checked"].as_bool().unwrap_or(false);
                let mut progress = checklist_progress.lock().unwrap();
                if is_checked {
                    progress.insert(id, true);
                } else {
                    progress.remove(&id);
                }
            }
            tiny_http::Response::from_string("{\"ok\":true}")
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/checklist-reset" => {
            checklist_progress.lock().unwrap().clear();
            tiny_http::Response::from_string("{\"ok\":true}")
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/active-checklist" => {
            let s = settings.lock().unwrap();
            let json = serde_json::json!(s.active_checklist);
            tiny_http::Response::from_string(json.to_string())
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/checklist" => {
            // Short URL: reads active preset from module config, serves renderer
            let module_dirs = module_base_dirs();
            let mut preset = "cessna-172".to_string();
            for dir in &module_dirs {
                let manifest = dir.join("checklist/module.json");
                if let Ok(content) = std::fs::read_to_string(&manifest) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(p) = parsed.get("default_preset").and_then(|v| v.as_str()) {
                            preset = p.to_string();
                            break;
                        }
                    }
                }
            }
            // Serve the renderer HTML with preset injected
            let renderer_paths = vec![
                dist_dir.join("modules/checklist/renderer.html"),
                dist_dir.join("../public/modules/checklist/renderer.html"),
                std::path::PathBuf::from("../public/modules/checklist/renderer.html"),
                std::path::PathBuf::from("public/modules/checklist/renderer.html"),
            ];
            let html = renderer_paths.iter()
                .find_map(|p| std::fs::read_to_string(p).ok())
                .unwrap_or_else(|| "<h1>Checklist not found</h1>".to_string());
            // Inject preset param via a small script
            let injected = html.replace(
                "</head>",
                &format!("<script>if(!new URLSearchParams(location.search).get('preset'))history.replaceState(null,'','/checklist?preset={}')</script></head>", preset)
            );
            tiny_http::Response::from_string(injected)
                .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/cockpit-basic" | "/cockpit-basic.html" | "/cockpit" | "/cockpit.html" => {
            let paths = vec![
                dist_dir.join("cockpit.html"),
                dist_dir.join("../public/cockpit.html"),
                std::path::PathBuf::from("../public/cockpit.html"),
                std::path::PathBuf::from("public/cockpit.html"),
            ];
            let html = paths.iter().find_map(|p| std::fs::read_to_string(p).ok()).unwrap_or_else(|| "<h1>cockpit.html not found</h1>".to_string());
            tiny_http::Response::from_string(html)
                .with_header("Content-Type: text/html; charset=utf-8".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/panel" | "/panel.html" => {
            let candidates = vec![
                dist_dir.join("panel.html"),
                dist_dir.join("../public/panel.html"),
                std::path::PathBuf::from("../public/panel.html"),
                std::path::PathBuf::from("public/panel.html"),
            ];
            let content = candidates.iter()
                .find_map(|p| std::fs::read(p).ok())
                .unwrap_or_else(|| b"<h1>panel.html not found</h1>".to_vec());
            tiny_http::Response::from_data(content)
                .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/presets" => {
            // List available presets from presets/ directory
            let presets_dirs = vec![
                dist_dir.join("presets"),
                dist_dir.join("../public/presets"),
                std::path::PathBuf::from("../public/presets"),
                std::path::PathBuf::from("public/presets"),
            ];
            let mut presets: Vec<String> = Vec::new();
            for dir in &presets_dirs {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "json") {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                if !presets.contains(&name.to_string()) {
                                    presets.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            let json = serde_json::json!(presets);
            tiny_http::Response::from_string(json.to_string())
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        _ if url.starts_with("/api/preset/") => {
            let name = url.trim_start_matches("/api/preset/");
            if *request.method() == tiny_http::Method::Get {
                // Load preset
                let candidates = vec![
                    dist_dir.join(format!("presets/{}.json", name)),
                    dist_dir.join(format!("../public/presets/{}.json", name)),
                    std::path::PathBuf::from(format!("../public/presets/{}.json", name)),
                    std::path::PathBuf::from(format!("public/presets/{}.json", name)),
                ];
                let content = candidates.iter()
                    .find_map(|p| std::fs::read(p).ok());
                match content {
                    Some(data) => tiny_http::Response::from_data(data)
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                        .boxed(),
                    None => tiny_http::Response::from_string(format!("{{\"error\":\"Preset '{}' not found\"}}", name))
                        .with_status_code(404)
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                        .boxed(),
                }
            } else if *request.method() == tiny_http::Method::Post {
                // Save preset
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).ok();
                // Validate JSON
                if serde_json::from_str::<serde_json::Value>(&body).is_ok() {
                    let save_dirs = vec![
                        dist_dir.join("presets"),
                        dist_dir.join("../public/presets"),
                        std::path::PathBuf::from("../public/presets"),
                        std::path::PathBuf::from("public/presets"),
                    ];
                    let mut saved = false;
                    for dir in &save_dirs {
                        if dir.exists() {
                            let path = dir.join(format!("{}.json", name));
                            if std::fs::write(&path, &body).is_ok() {
                                saved = true;
                                eprintln!("[PRESET] Saved '{}' to {:?}", name, path);
                                break;
                            }
                        }
                    }
                    if saved {
                        tiny_http::Response::from_string("{\"ok\":true}")
                            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                            .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                            .boxed()
                    } else {
                        tiny_http::Response::from_string("{\"error\":\"Could not save preset\"}")
                            .with_status_code(500)
                            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                            .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                            .boxed()
                    }
                } else {
                    tiny_http::Response::from_string("{\"error\":\"Invalid JSON\"}")
                        .with_status_code(400)
                        .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                        .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                        .boxed()
                }
            } else {
                tiny_http::Response::from_string("{\"error\":\"Method not allowed\"}")
                    .with_status_code(405)
                    .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                    .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                    .boxed()
            }
        },
        "/api/mobile-gyro" => {
            if request.method() == &tiny_http::Method::Post {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).ok();
                if body.contains("center") {
                    // Signal recenter by toggling off briefly
                    MOBILE_GYRO_ACTIVE.store(false, Ordering::Relaxed);
                    std::thread::spawn(|| { std::thread::sleep(Duration::from_millis(100)); MOBILE_GYRO_ACTIVE.store(true, Ordering::Relaxed); });
                } else {
                    let active = body.contains("true");
                    MOBILE_GYRO_ACTIVE.store(active, Ordering::Relaxed);
                }
            }
            let active = MOBILE_GYRO_ACTIVE.load(Ordering::Relaxed);
            tiny_http::Response::from_string(format!("{{\"active\":{}}}", active))
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/command" => {
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).ok();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                let cmd = val["cmd"].as_str().unwrap_or("").to_string();
                let value = val["value"].as_f64().unwrap_or(0.0);
                let _ = send_command(cmd, value);
            }
            tiny_http::Response::from_string("{\"ok\":true}")
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/overlay" => {
            let ov = overlay.lock().unwrap().0.clone();
            tiny_http::Response::from_string(if ov.is_empty() { "[]".into() } else { ov })
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/capture" => {
            let img = capture.lock().unwrap().0.clone();
            let json = serde_json::json!({ "image": img });
            tiny_http::Response::from_string(json.to_string())
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        "/api/mode" => {
            let s = settings.lock().unwrap();
            let json = serde_json::json!(s.mode);
            tiny_http::Response::from_string(json.to_string())
                .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                .with_header("Access-Control-Allow-Origin: *".parse::<tiny_http::Header>().unwrap())
                .boxed()
        },
        _ => {
            let clean_url = url.split('?').next().unwrap_or(&url);
            let file_path = if clean_url == "/" { "/index.html".to_string() } else { clean_url.to_string() };
            let stripped = file_path.trim_start_matches('/');
            // Try dist dir first, then public/ fallbacks
            let candidates = vec![
                dist_dir.join(stripped),
                dist_dir.join("..").join("public").join(stripped),
                std::path::PathBuf::from("../public").join(stripped),
                std::path::PathBuf::from("public").join(stripped),
            ];
            let found = candidates.iter().find(|p| p.exists() && p.is_file());
            if let Some(full_path) = found {
                let content = std::fs::read(full_path).unwrap_or_default();
                let mime = if full_path.extension().map_or(false, |e| e == "html") { "text/html" }
                    else if full_path.extension().map_or(false, |e| e == "js") { "application/javascript" }
                    else if full_path.extension().map_or(false, |e| e == "css") { "text/css" }
                    else if full_path.extension().map_or(false, |e| e == "svg") { "image/svg+xml" }
                    else if full_path.extension().map_or(false, |e| e == "json") { "application/json" }
                    else { "application/octet-stream" };
                tiny_http::Response::from_data(content)
                    .with_header(format!("Content-Type: {}", mime).parse::<tiny_http::Header>().unwrap())
                    .boxed()
            } else if url.contains('.') {
                // File-like URL but not found — return 404, don't serve SPA
                tiny_http::Response::from_string("Not found")
                    .with_status_code(404)
                    .boxed()
            } else {
                // SPA route (no extension) — serve index.html
                let index = dist_dir.join("index.html");
                let content = std::fs::read(&index).unwrap_or_default();
                tiny_http::Response::from_data(content)
                    .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap())
                    .boxed()
            }
        },
    }
}

fn generate_self_signed_cert() -> (Vec<u8>, Vec<u8>) {
    use rcgen::{CertifiedKey, generate_simple_self_signed};

    // Include LAN IPs as SANs so browsers accept it
    let mut sans = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    if let Ok(ip) = local_ip_address::local_ip() {
        sans.push(ip.to_string());
    }
    // Also try all local IPs
    if let Ok(ifaces) = local_ip_address::list_afinet_netifas() {
        for (_, ip) in &ifaces {
            let s = ip.to_string();
            if !sans.contains(&s) {
                sans.push(s);
            }
        }
    }

    eprintln!("[HTTPS] Generating self-signed cert for SANs: {:?}", sans);
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(sans)
        .expect("Failed to generate self-signed cert");
    (cert.pem().into_bytes(), key_pair.serialize_pem().into_bytes())
}

fn serve_loop(
    server: tiny_http::Server,
    sim_state: SimState,
    settings: SettingsState,
    capture: CaptureState,
    overlay: OverlayState,
    checklist_progress: ChecklistProgress,
    dist_dir: std::path::PathBuf,
) {
    for mut request in server.incoming_requests() {
        let response = handle_request(
            &mut request, &sim_state, &settings, &capture, &overlay, &checklist_progress, &dist_dir,
        );
        let _ = request.respond(response);
    }
}

fn start_http_server(sim_state: SimState, settings: SettingsState, capture: CaptureState, overlay: OverlayState, dist_dir: std::path::PathBuf) {
    let cl_progress: ChecklistProgress = Arc::new(Mutex::new(HashMap::new()));

    // HTTP server — configurable port (default 8080)
    let http_port = std::env::var("CF_PORT").unwrap_or_else(|_| "8080".to_string());
    let (sim1, set1, cap1, ov1, cl1, dir1) = (sim_state.clone(), settings.clone(), capture.clone(), overlay.clone(), cl_progress.clone(), dist_dir.clone());
    let port1 = http_port.clone();
    std::thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", port1);
        let server = match tiny_http::Server::http(&addr) {
            Ok(s) => s,
            Err(e) => { eprintln!("[HTTP] Failed to start on {}: {}", addr, e); return; }
        };
        eprintln!("[HTTP] Serving on port {}", port1);
        serve_loop(server, sim1, set1, cap1, ov1, cl1, dir1);
    });

    // HTTPS on port 8443 (for gyroscope access from mobile devices)
    let cl2 = cl_progress.clone();
    std::thread::spawn(move || {
        let (cert_pem, key_pem) = generate_self_signed_cert();
        let ssl_config = tiny_http::SslConfig {
            certificate: cert_pem,
            private_key: key_pem,
        };
        let server = match tiny_http::Server::https("0.0.0.0:8443", ssl_config) {
            Ok(s) => s,
            Err(e) => { eprintln!("[HTTPS] Failed to start: {}", e); return; }
        };
        eprintln!("[HTTPS] Serving on port 8443 (accept cert warning on phone for gyro)");
        serve_loop(server, sim_state, settings, capture, overlay, cl2, dist_dir);
    });
}

// Window capture for pop-out panels
#[tauri::command]
fn list_sim_windows() -> Vec<(String, i64)> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::um::winuser::{EnumWindows, GetWindowTextW, GetWindowTextLengthW, IsWindowVisible};
    use winapi::shared::windef::HWND;
    use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};

    let mut windows: Vec<(String, i64)> = Vec::new();
    let windows_ptr = &mut windows as *mut Vec<(String, i64)> as LPARAM;

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if unsafe { IsWindowVisible(hwnd) } == 0 {
            return TRUE;
        }
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        if len == 0 { return TRUE; }
        let mut buf = vec![0u16; (len + 1) as usize];
        unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        let title = OsString::from_wide(&buf[..len as usize]).to_string_lossy().to_string();
        // Show all visible windows with a title
        let vec = unsafe { &mut *(lparam as *mut Vec<(String, i64)>) };
        vec.push((title, hwnd as i64));
        TRUE
    }

    unsafe { EnumWindows(Some(enum_callback), windows_ptr); }
    windows
}

#[tauri::command]
fn capture_window(hwnd_val: i64) -> Result<String, String> {
    use winapi::um::winuser::{GetClientRect, GetDC, ReleaseDC};
    use winapi::um::wingdi::{CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, GetDIBits, DeleteDC, DeleteObject, SRCCOPY, BITMAPINFOHEADER, BI_RGB, BITMAPINFO, DIB_RGB_COLORS};
    use winapi::shared::windef::{HWND, RECT, HDC, HBITMAP};

    let hwnd = hwnd_val as HWND;

    unsafe {
        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetClientRect(hwnd, &mut rect);
        let w = (rect.right - rect.left) as i32;
        let h = (rect.bottom - rect.top) as i32;
        if w <= 0 || h <= 0 { return Err("Window has no size".into()); }

        // Limit size for performance
        let max_w = 480;
        let max_h = 320;
        let scale = f64::min(max_w as f64 / w as f64, max_h as f64 / h as f64).min(1.0);
        let out_w = (w as f64 * scale) as i32;
        let out_h = (h as f64 * scale) as i32;

        let hdc_src: HDC = GetDC(hwnd);
        let hdc_mem: HDC = CreateCompatibleDC(hdc_src);
        let hbm: HBITMAP = CreateCompatibleBitmap(hdc_src, out_w, out_h);
        SelectObject(hdc_mem, hbm as _);

        // Use StretchBlt for scaling
        winapi::um::wingdi::SetStretchBltMode(hdc_mem, winapi::um::wingdi::HALFTONE as i32);
        winapi::um::wingdi::StretchBlt(hdc_mem, 0, 0, out_w, out_h, hdc_src, 0, 0, w, h, SRCCOPY);

        // Get bitmap data
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: out_w,
                biHeight: -out_h, // top-down
                biPlanes: 1,
                biBitCount: 24,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [std::mem::zeroed()],
        };

        let row_size = ((out_w * 3 + 3) / 4 * 4) as usize;
        let mut pixels = vec![0u8; row_size * out_h as usize];
        GetDIBits(hdc_mem, hbm, 0, out_h as u32, pixels.as_mut_ptr() as _, &mut bmi, DIB_RGB_COLORS);

        // Convert BGR to simple BMP then base64
        // Build BMP file in memory
        let file_size = 54 + pixels.len();
        let mut bmp = Vec::with_capacity(file_size);
        // BMP header
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
        bmp.extend_from_slice(&0u32.to_le_bytes()); // reserved
        bmp.extend_from_slice(&54u32.to_le_bytes()); // offset
        // DIB header
        bmp.extend_from_slice(&40u32.to_le_bytes()); // header size
        bmp.extend_from_slice(&out_w.to_le_bytes());
        bmp.extend_from_slice(&(-out_h).to_le_bytes()); // top-down
        bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
        bmp.extend_from_slice(&24u16.to_le_bytes()); // bpp
        bmp.extend_from_slice(&0u32.to_le_bytes()); // compression
        bmp.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
        bmp.extend_from_slice(&[0u8; 16]); // rest of header
        bmp.extend_from_slice(&pixels);

        // Cleanup
        DeleteObject(hbm as _);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_src);

        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp);
        Ok(format!("data:image/bmp;base64,{}", b64))
    }
}

#[allow(unused_assignments)]
fn start_xplane(sim_state: SimState, app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let datarefs: Vec<(&str, &str)> = vec![
            ("AIRSPEED", "sim/cockpit2/gauges/indicators/airspeed_kts_pilot"),
            ("ALTITUDE", "sim/cockpit2/gauges/indicators/altitude_ft_pilot"),
            ("HEADING", "sim/cockpit2/gauges/indicators/heading_vacuum_deg_mag_pilot"),
            ("PITCH", "sim/cockpit2/gauges/indicators/pitch_vacuum_deg_pilot"),
            ("ROLL", "sim/cockpit2/gauges/indicators/roll_vacuum_deg_pilot"),
            ("VS", "sim/cockpit2/gauges/indicators/vvi_fpm_pilot"),
            ("RPM", "sim/cockpit2/engine/indicators/engine_speed_rpm[0]"),
            ("TURN_RATE", "sim/cockpit2/gauges/indicators/turn_rate_heading_deg_pilot"),
            ("SLIP", "sim/cockpit2/gauges/indicators/slip_deg"),
            ("BARO", "sim/cockpit2/gauges/actuators/barometer_setting_in_hg_pilot"),
            ("NAV1_OBS", "sim/cockpit/radios/nav1_obs_degm"),
            ("NAV1_CDI", "sim/cockpit/radios/nav1_hdef_dot"),
            ("NAV1_GS", "sim/cockpit/radios/nav1_vdef_dot"),
            ("NAV1_TOFROM", "sim/cockpit/radios/nav1_fromto"),
            ("NAV2_OBS", "sim/cockpit/radios/nav2_obs_degm"),
            ("NAV2_CDI", "sim/cockpit/radios/nav2_hdef_dot"),
            ("MASTER_BATTERY", "sim/cockpit/electrical/battery_on"),
            ("MASTER_ALT", "sim/cockpit/electrical/generator_on[0]"),
            ("FUEL_PUMP", "sim/cockpit/engine/fuel_pump_on[0]"),
            ("BEACON", "sim/cockpit/electrical/beacon_lights_on"),
            ("LANDING_LIGHT", "sim/cockpit/electrical/landing_lights_on"),
            ("TAXI_LIGHT", "sim/cockpit/electrical/taxi_light_on"),
            ("NAV_LIGHT", "sim/cockpit/electrical/nav_lights_on"),
            ("STROBE", "sim/cockpit/electrical/strobe_lights_on"),
            ("PITOT_HEAT", "sim/cockpit/switches/pitot_heat_on"),
            ("AVIONICS_MASTER", "sim/cockpit2/switches/avionics_power_on"),
            ("THROTTLE", "sim/cockpit2/engine/actuators/throttle_ratio[0]"),
            ("MIXTURE", "sim/cockpit2/engine/actuators/mixture_ratio[0]"),
            ("CARB_HEAT", "sim/cockpit2/engine/actuators/carb_heat_ratio[0]"),
            ("FLAPS", "sim/cockpit2/controls/flap_ratio"),
            ("TRIM", "sim/cockpit2/controls/elevator_trim"),
            ("FUEL_SEL", "sim/cockpit2/fuel/fuel_tank_selector"),
            ("MAGNETO", "sim/cockpit2/engine/actuators/ignition_key[0]"),
        ];

        loop {
            let sock = match UdpSocket::bind("0.0.0.0:49002") {
                Ok(s) => s,
                Err(_) => { std::thread::sleep(Duration::from_secs(3)); continue; }
            };
            sock.set_read_timeout(Some(Duration::from_secs(5))).ok();

            // Subscribe to all datarefs at 20Hz
            for (i, (_, dataref)) in datarefs.iter().enumerate() {
                let mut pkt = Vec::with_capacity(413);
                pkt.extend_from_slice(b"RREF\0");
                pkt.extend_from_slice(&20u32.to_le_bytes()); // freq
                pkt.extend_from_slice(&(i as u32).to_le_bytes()); // index
                let dr = dataref.as_bytes();
                pkt.extend_from_slice(dr);
                pkt.resize(413, 0);
                if sock.send_to(&pkt, "127.0.0.1:49000").is_err() {
                    break;
                }
            }

            let mut buf = [0u8; 4096];
            let mut connected = false;

            loop {
                match sock.recv_from(&mut buf) {
                    Ok((n, _)) if n >= 9 && &buf[0..4] == b"RREF" => {
                        if !connected {
                            connected = true;
                            let mut s = sim_state.lock().unwrap();
                            s.insert("_CONNECTED".into(), 1.0);
                            let _ = app.emit("sim-connected", true);
                        }
                        let mut off = 5;
                        while off + 8 <= n {
                            let idx = u32::from_le_bytes([buf[off], buf[off+1], buf[off+2], buf[off+3]]) as usize;
                            let val = f32::from_le_bytes([buf[off+4], buf[off+5], buf[off+6], buf[off+7]]) as f64;
                            off += 8;
                            if idx < datarefs.len() {
                                let key = datarefs[idx].0;
                                let mut s = sim_state.lock().unwrap();
                                s.insert(key.into(), val);
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {
                        if connected {
                            connected = false;
                            let mut s = sim_state.lock().unwrap();
                            s.insert("_CONNECTED".into(), 0.0);
                            let _ = app.emit("sim-connected", false);
                        }
                        break; // reconnect
                    }
                }
            }

            // Unsubscribe
            for (i, _) in datarefs.iter().enumerate() {
                let mut pkt = Vec::with_capacity(413);
                pkt.extend_from_slice(b"RREF\0");
                pkt.extend_from_slice(&0u32.to_le_bytes());
                pkt.extend_from_slice(&(i as u32).to_le_bytes());
                pkt.resize(413, 0);
                let _ = sock.send_to(&pkt, "127.0.0.1:49000");
            }

            std::thread::sleep(Duration::from_secs(3));
        }
    });
}

fn start_arduino_reader(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            // Find COM7 (Arduino Nano)
            let port = serialport::new("COM7", 115200)
                .timeout(Duration::from_millis(100))
                .open();

            let mut serial = match port {
                Ok(s) => {
                    eprintln!("[ARDUINO] Connected to COM7! Waiting for boot...");
                    std::thread::sleep(Duration::from_secs(3)); // Wait for Arduino reset
                    eprintln!("[ARDUINO] Ready, reading frames...");
                    let _ = app.emit("arduino-status", "connected");
                    s
                }
                Err(_) => { std::thread::sleep(Duration::from_secs(10)); continue; }
            };

            let mut buf = [0u8; 256];
            let mut parser: Vec<u8> = Vec::new();
            let d3_pressed = Arc::new(Mutex::new(false));
            let d3_clone = d3_pressed.clone();
            let xp_sock = UdpSocket::bind("0.0.0.0:0").ok();
            let mut frame_count: u64 = 0;

            // Starter hold thread - keeps sending key=4 while D3 pressed
            std::thread::spawn(move || {
                let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
                loop {
                    let pressed = *d3_clone.lock().unwrap();
                    if pressed {
                        let mut pkt = Vec::with_capacity(509);
                        pkt.extend_from_slice(b"DREF\0");
                        pkt.extend_from_slice(&4.0f32.to_le_bytes());
                        pkt.extend_from_slice(b"sim/cockpit2/engine/actuators/ignition_key[0]");
                        pkt.resize(509, 0);
                        let _ = sock.send_to(&pkt, "127.0.0.1:49000");
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            });

            loop {
                match serial.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        if frame_count == 0 { eprintln!("[ARDUINO] First data: {} bytes", n); }
                        parser.extend_from_slice(&buf[..n]);

                        while parser.len() >= 6 {
                            if parser[0] != 0xAA && parser[0] != 0xCC {
                                parser.remove(0);
                                continue;
                            }

                            let hdr = parser[0];
                            let _dev = parser[1];
                            let pin = parser[2];
                            let vl = parser[3];
                            let vh = parser[4];
                            let ck = parser[5];
                            let value = (vl as u16) | ((vh as u16) << 8);
                            let expected = hdr ^ _dev ^ pin ^ vl ^ vh;

                            if ck != expected {
                                parser.remove(0);
                                continue;
                            }

                            parser.drain(..6);
                            frame_count += 1;
                            if frame_count % 500 == 0 {
                                eprintln!("[ARDUINO] {} frames, last: hdr={:#x} pin={} val={}", frame_count, hdr, pin, value);
                            }

                            // Emit ALL digital pin events to frontend
                            if hdr == 0xAA && (value == 0 || value == 1) {
                                eprintln!("[ARDUINO] Pin D{}={}", pin, value);
                                let _ = app.emit("arduino-pin", format!("D{}={}", pin, value));
                            }

                            // D3 = pin 3, press = 1, release = 0
                            if hdr == 0xAA && pin == 3 {
                                if let Some(ref sock) = xp_sock {
                                    let currently_pressed = *d3_pressed.lock().unwrap();
                                    if value == 1 && !currently_pressed {
                                        *d3_pressed.lock().unwrap() = true;
                                        // Enable override + set key to 4
                                        let mut pkt = Vec::with_capacity(509);
                                        pkt.extend_from_slice(b"DREF\0");
                                        pkt.extend_from_slice(&1.0f32.to_le_bytes());
                                        pkt.extend_from_slice(b"sim/operation/override/override_starters");
                                        pkt.resize(509, 0);
                                        let _ = sock.send_to(&pkt, "127.0.0.1:49000");

                                        let mut pkt2 = Vec::with_capacity(509);
                                        pkt2.extend_from_slice(b"DREF\0");
                                        pkt2.extend_from_slice(&4.0f32.to_le_bytes());
                                        pkt2.extend_from_slice(b"sim/cockpit2/engine/actuators/ignition_key[0]");
                                        pkt2.resize(509, 0);
                                        let _ = sock.send_to(&pkt2, "127.0.0.1:49000");
                                        log::info!("D3 PRESS -> START (override + key=4)");
                                    } else if value == 0 && currently_pressed {
                                        *d3_pressed.lock().unwrap() = false;
                                        // Set key back to 3 (BOTH) + disable override
                                        let mut pkt = Vec::with_capacity(509);
                                        pkt.extend_from_slice(b"DREF\0");
                                        pkt.extend_from_slice(&3.0f32.to_le_bytes());
                                        pkt.extend_from_slice(b"sim/cockpit2/engine/actuators/ignition_key[0]");
                                        pkt.resize(509, 0);
                                        let _ = sock.send_to(&pkt, "127.0.0.1:49000");

                                        let mut pkt2 = Vec::with_capacity(509);
                                        pkt2.extend_from_slice(b"DREF\0");
                                        pkt2.extend_from_slice(&0.0f32.to_le_bytes());
                                        pkt2.extend_from_slice(b"sim/operation/override/override_starters");
                                        pkt2.resize(509, 0);
                                        let _ = sock.send_to(&pkt2, "127.0.0.1:49000");
                                        log::info!("D3 RELEASE -> BOTH (key=3, override off)");
                                    }
                                }
                            }
                        }
                    }
                    Ok(0) => { eprintln!("[ARDUINO] read 0 bytes"); }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => { eprintln!("[ARDUINO] read error: {}", e); break; }
                }
            }
            log::info!("Arduino disconnected, reconnecting...");
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let sim_state: SimState = Arc::new(Mutex::new(HashMap::new()));
    let settings: SettingsState = Arc::new(Mutex::new(AppSettings::default()));
    let capture: CaptureState = Arc::new(Mutex::new(CaptureData(String::new())));
    let overlay: OverlayState = Arc::new(Mutex::new(OverlayData(String::new())));
    let sim_clone = sim_state.clone();
    let sim_http = sim_state.clone();
    let settings_http = settings.clone();
    let capture_http = capture.clone();
    let overlay_http = overlay.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(sim_state)
        .manage(settings)
        .manage(capture)
        .manage(overlay)
        .invoke_handler(tauri::generate_handler![get_gauge_data, send_command, get_local_ip, get_settings, set_settings, list_sim_windows, capture_window, capture_screen_region, store_capture, store_overlay, list_presets, load_preset, save_preset, list_modules, update_module, load_module_preset, save_module_preset, delete_module_preset, check_for_updates, download_and_install_update])
        .setup(move |app| {
            start_xplane(sim_clone, app.handle().clone());
            start_arduino_reader(app.handle().clone());

            // Start HTTP server for LAN checklist
            let dist = app.path().resource_dir().unwrap_or_default().join("dist");
            // In dev, use the public dir
            let dist_dir = if dist.exists() { dist } else {
                std::env::current_dir().unwrap_or_default().join("../dist")
            };
            start_http_server(sim_http, settings_http, capture_http, overlay_http, dist_dir);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

//! Development-only Windows layout extraction. No network access.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::{json, Map, Value};
    use std::{fs, process::Command};
    #[link(name = "user32")]
    extern "system" {
        fn LoadKeyboardLayoutW(id: *const u16, flags: u32) -> isize;
        fn MapVirtualKeyExW(code: u32, kind: u32, layout: isize) -> u32;
        fn ToUnicodeEx(
            vk: u32,
            scan: u32,
            state: *const u8,
            out: *mut u16,
            len: i32,
            flags: u32,
            layout: isize,
        ) -> i32;
        fn UnloadKeyboardLayout(layout: isize) -> i32;
    }
    let registry = Command::new("reg")
        .args([
            "query",
            r"HKLM\SYSTEM\CurrentControlSet\Control\Keyboard Layouts",
            "/s",
            "/f",
            "KBDKNI.DLL",
            "/d",
            "/e",
        ])
        .output()?;
    let registry = String::from_utf8_lossy(&registry.stdout);
    let nida = registry
        .lines()
        .map(str::trim)
        .find(|s| s.starts_with("HKEY_"))
        .and_then(|s| s.rsplit('\\').next())
        .ok_or("Microsoft NiDA layout not installed")?;
    let rows: &[&[(&str, u32)]] = &[
        &[
            ("Backquote", 0x29),
            ("Digit1", 2),
            ("Digit2", 3),
            ("Digit3", 4),
            ("Digit4", 5),
            ("Digit5", 6),
            ("Digit6", 7),
            ("Digit7", 8),
            ("Digit8", 9),
            ("Digit9", 10),
            ("Digit0", 11),
            ("Minus", 12),
            ("Equal", 13),
        ],
        &[
            ("KeyQ", 16),
            ("KeyW", 17),
            ("KeyE", 18),
            ("KeyR", 19),
            ("KeyT", 20),
            ("KeyY", 21),
            ("KeyU", 22),
            ("KeyI", 23),
            ("KeyO", 24),
            ("KeyP", 25),
            ("BracketLeft", 26),
            ("BracketRight", 27),
            ("Backslash", 43),
        ],
        &[
            ("KeyA", 30),
            ("KeyS", 31),
            ("KeyD", 32),
            ("KeyF", 33),
            ("KeyG", 34),
            ("KeyH", 35),
            ("KeyJ", 36),
            ("KeyK", 37),
            ("KeyL", 38),
            ("Semicolon", 39),
            ("Quote", 40),
        ],
        &[
            ("KeyZ", 44),
            ("KeyX", 45),
            ("KeyC", 46),
            ("KeyV", 47),
            ("KeyB", 48),
            ("KeyN", 49),
            ("KeyM", 50),
            ("Comma", 51),
            ("Period", 52),
            ("Slash", 53),
            ("IntlBackslash", 86),
        ],
        &[("Space", 57)],
    ];
    fs::create_dir_all("content/layouts")?;
    for (id, klid, km, en) in [
        ("nida", nida, "នីដា (NiDA)", "Khmer NiDA"),
        ("khmer_legacy", "00000453", "ខ្មែរ (ចាស់)", "Khmer legacy"),
    ] {
        let wide: Vec<u16> = klid.encode_utf16().chain(Some(0)).collect();
        // No KLF_ACTIVATE: do not switch the user's active input language.
        let handle = unsafe { LoadKeyboardLayoutW(wide.as_ptr(), 0) };
        if handle == 0 {
            return Err(format!("cannot load {klid}").into());
        }
        let mut keys = Map::new();
        for (row, entries) in rows.iter().enumerate() {
            for (column, (code, scan)) in entries.iter().enumerate() {
                let finger = if *code == "IntlBackslash" {
                    "L5"
                } else if row == 4 {
                    "R1"
                } else if row == 0 {
                    [
                        "L5", "L5", "L4", "L3", "L2", "L2", "R2", "R2", "R3", "R4", "R5", "R5",
                        "R5",
                    ][column]
                } else {
                    let split = if row == 0 { 6 } else { 5 };
                    if column < split {
                        ["L5", "L4", "L3", "L2", "L2", "L2"][column.min(5)]
                    } else {
                        ["R2", "R2", "R3", "R4", "R5", "R5", "R5", "R5"][(column - split).min(7)]
                    }
                };
                let mut key = json!({"finger":finger,"row":row,"scan_code":scan});
                for (layer, shift, altgr) in [
                    ("base", false, false),
                    ("shift", true, false),
                    ("altgr", false, true),
                    ("shift_altgr", true, true),
                ] {
                    let mut state = [0u8; 256];
                    if shift {
                        state[0x10] = 0x80;
                        state[0xa0] = 0x80;
                    }
                    if altgr {
                        state[0x11] = 0x80;
                        state[0x12] = 0x80;
                        state[0xa2] = 0x80;
                        state[0xa5] = 0x80;
                    }
                    let vk = unsafe { MapVirtualKeyExW(*scan, 3, handle) };
                    let mut out = [0u16; 16];
                    let n = unsafe {
                        ToUnicodeEx(vk, *scan, state.as_ptr(), out.as_mut_ptr(), 16, 4, handle)
                    };
                    if n < 0 {
                        return Err(format!("dead key {id} {code} {layer}; unsupported").into());
                    }
                    let value = if n > 0 {
                        let text = String::from_utf16(&out[..n as usize])?;
                        if text.chars().all(|c| !c.is_control()) {
                            Value::String(text)
                        } else {
                            Value::Null
                        }
                    } else {
                        Value::Null
                    };
                    key[layer] = value;
                }
                keys.insert(code.to_string(), key);
            }
        }
        unsafe {
            UnloadKeyboardLayout(handle);
        }
        let value = json!({"id":id,"name":{"km":km,"en":en},"verified":true,"source":{"kind":"Windows ToUnicodeEx","klid":klid,"flags":4},"keys":keys});
        fs::write(
            format!("content/layouts/{id}.json"),
            serde_json::to_string_pretty(&value)? + "\n",
        )?;
        println!("Extracted {id} ({klid})");
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    panic!("Layout extraction requires Windows; use the committed JSON elsewhere.");
}

use webview2_com::Microsoft::Web::WebView2::Win32::{
    GetAvailableCoreWebView2BrowserVersionString, ICoreWebView2Settings3, ICoreWebView2Settings4,
};
use windows::{
    core::{w, Interface, PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE},
        System::{Com::CoTaskMemFree, Threading::CreateMutexW},
        UI::WindowsAndMessaging::{
            FindWindowW, MessageBoxW, SetForegroundWindow, ShowWindow, MB_ICONERROR, MB_OK,
            SW_RESTORE,
        },
    },
};
pub struct Instance(HANDLE);
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
fn native_message(key: &str) -> Vec<u16> {
    let messages: serde_json::Value =
        serde_json::from_str(include_str!("../../src/lib/i18n/native.json"))
            .expect("bundled native messages");
    format!(
        "{}\n\n{}",
        messages[key]["km"].as_str().unwrap_or(""),
        messages[key]["en"].as_str().unwrap_or("")
    )
    .encode_utf16()
    .chain([0])
    .collect()
}
fn supported_version(value: &str) -> bool {
    value
        .split('.')
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .is_some_and(|major| major >= 105)
}
pub fn startup() -> Option<Instance> {
    unsafe {
        let handle = match CreateMutexW(None, false, w!("Local\\SvaType.Desktop.Instance")) {
            Ok(h) => h,
            Err(_) => {
                MessageBoxW(
                    None,
                    PCWSTR(native_message("startup_failed").as_ptr()),
                    w!("SvaType"),
                    MB_OK | MB_ICONERROR,
                );
                return None;
            }
        };
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if let Ok(window) = FindWindowW(None, w!("SvaType")) {
                let _ = ShowWindow(window, SW_RESTORE);
                let _ = SetForegroundWindow(window);
            }
            let _ = CloseHandle(handle);
            return None;
        }
        let instance = Instance(handle);
        let mut version = PWSTR::null();
        let found =
            GetAvailableCoreWebView2BrowserVersionString(PCWSTR::null(), &mut version).is_ok();
        let available = found
            && !version.is_null()
            && version
                .to_string()
                .is_ok_and(|value| supported_version(&value));
        if !version.is_null() {
            CoTaskMemFree(Some(version.0.cast()));
        }
        if !available {
            MessageBoxW(
                None,
                PCWSTR(native_message("webview_missing").as_ptr()),
                w!("SvaType"),
                MB_OK | MB_ICONERROR,
            );
            return None;
        }
        Some(instance)
    }
}
pub fn lockdown(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    window.with_webview(|webview| unsafe {
        let result = (|| -> windows::core::Result<()> {
            let settings = webview.controller().CoreWebView2()?.Settings()?;
            settings.SetAreDefaultContextMenusEnabled(false)?;
            settings.SetAreDefaultScriptDialogsEnabled(false)?;
            settings.SetIsStatusBarEnabled(false)?;
            settings.SetAreDevToolsEnabled(cfg!(debug_assertions))?;
            settings
                .cast::<ICoreWebView2Settings3>()?
                .SetAreBrowserAcceleratorKeysEnabled(false)?;
            let autofill = settings.cast::<ICoreWebView2Settings4>()?;
            autofill.SetIsPasswordAutosaveEnabled(false)?;
            autofill.SetIsGeneralAutofillEnabled(false)?;
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("WebView settings: {error}");
        }
    })
}

#[cfg(test)]
mod tests {
    use super::supported_version;
    #[test]
    fn requires_the_frontend_browser_baseline() {
        assert!(!supported_version("104.0.1293.0"));
        assert!(supported_version("105.0.0.0"));
        assert!(supported_version("150.0.3657.0 beta"));
        assert!(!supported_version("unknown"));
    }
}

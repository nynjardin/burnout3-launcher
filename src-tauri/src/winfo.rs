//! Windows through its API, without starting any other program: a file's
//! version (its FileVersion resource), a short description of the PC
//! (Windows, processor, memory, graphics cards) from the registry, and
//! opening a folder or a page (ShellExecute; the `open` crate went through
//! PowerShell, which antivirus programs watch).

use std::path::Path;

/// The FileVersion of an exe ("0.8.0-lan.1"), or None (no version resource).
#[cfg(windows)]
pub fn file_version(path: &Path) -> Option<String> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};

    let file: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let size = unsafe { GetFileVersionInfoSizeW(file.as_ptr(), null_mut()) };
    if size == 0 {
        return None;
    }
    let mut block = vec![0u8; size as usize];
    if unsafe { GetFileVersionInfoW(file.as_ptr(), 0, size, block.as_mut_ptr() as *mut c_void) } == 0 {
        return None;
    }
    let query = |what: &str| -> Option<(*const u16, u32)> {
        let what = wide(what);
        let mut at: *mut c_void = null_mut();
        let mut len = 0u32;
        let found = unsafe { VerQueryValueW(block.as_ptr() as *const c_void, what.as_ptr(), &mut at, &mut len) };
        (found != 0 && !at.is_null() && len > 0).then_some((at as *const u16, len))
    };
    // The file's own language first, then the usual ones.
    let mut languages = Vec::new();
    if let Some((at, len)) = query("\\VarFileInfo\\Translation") {
        if len >= 4 {
            let (language, codepage) = unsafe { (*at, *at.add(1)) };
            languages.push(format!("{language:04x}{codepage:04x}"));
        }
    }
    languages.extend(["040904b0".to_string(), "040904e4".to_string()]);
    for language in languages {
        if let Some((at, len)) = query(&format!("\\StringFileInfo\\{language}\\FileVersion")) {
            let text = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(at, len as usize) });
            let text = text.trim_end_matches('\0').trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub fn file_version(_path: &Path) -> Option<String> {
    None
}

#[cfg(windows)]
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// A text value under HKEY_LOCAL_MACHINE (read only).
#[cfg(windows)]
fn registry(key: &str, value: &str) -> Option<String> {
    use std::ptr::null_mut;
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
    let (key, value) = (wide(key), wide(value));
    let mut bytes = 0u32;
    let first = unsafe {
        RegGetValueW(HKEY_LOCAL_MACHINE, key.as_ptr(), value.as_ptr(), RRF_RT_REG_SZ, null_mut(), null_mut(), &mut bytes)
    };
    if first != 0 || bytes < 2 {
        return None;
    }
    let mut text = vec![0u16; bytes as usize / 2];
    let second = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            text.as_mut_ptr() as *mut std::ffi::c_void,
            &mut bytes,
        )
    };
    if second != 0 {
        return None;
    }
    let text = String::from_utf16_lossy(&text);
    let text = text.trim_end_matches('\0').trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Windows, the processor, the memory, the graphics cards and their drivers.
#[cfg(windows)]
pub fn machine() -> String {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut lines = Vec::new();

    let nt = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let build: u32 = registry(nt, "CurrentBuild").and_then(|b| b.parse().ok()).unwrap_or(0);
    let mut product = registry(nt, "ProductName").unwrap_or_else(|| "Windows".into());
    // Windows 11 still says "Windows 10" there; its builds start at 22000.
    if build >= 22000 {
        product = product.replace("Windows 10", "Windows 11");
    }
    let release = registry(nt, "DisplayVersion").map(|r| format!(" {r}")).unwrap_or_default();
    lines.push(format!("Windows : {product}{release} (build {build})"));

    if let Some(cpu) = registry("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0", "ProcessorNameString") {
        lines.push(format!("Processeur : {cpu}"));
    }

    let mut memory: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    memory.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut memory) } != 0 {
        lines.push(format!("Mémoire : {:.1} Go", memory.ullTotalPhys as f64 / (1u64 << 30) as f64));
    }

    // The display adapters' class: one numbered subkey per driver installed.
    let class = "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";
    for n in 0..16 {
        let key = format!("{class}\\{n:04}");
        if let Some(name) = registry(&key, "DriverDesc") {
            let driver = registry(&key, "DriverVersion").unwrap_or_else(|| "?".into());
            lines.push(format!("Carte graphique : {name} (pilote {driver})"));
        }
    }
    lines.join("\n")
}

#[cfg(not(windows))]
pub fn machine() -> String {
    String::from("Machine : inconnue")
}

/// Opens a folder in the Explorer, or a page in the default browser.
#[cfg(windows)]
pub fn open(target: &std::ffi::OsStr) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let target: Vec<u16> = target.encode_wide().chain(Some(0)).collect();
    let verb = wide("open");
    let done = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), target.as_ptr(), null(), null(), SW_SHOWNORMAL) };
    // Above 32: done (the HINSTANCE it returns is a code, for history's sake).
    if done as isize > 32 {
        Ok(())
    } else {
        Err(format!("Windows n'a pas pu l'ouvrir (code {}).", done as isize))
    }
}

#[cfg(not(windows))]
pub fn open(_target: &std::ffi::OsStr) -> Result<(), String> {
    Err("Pas encore fait hors de Windows.".into())
}

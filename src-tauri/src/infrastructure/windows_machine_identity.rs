use crate::application::license::MachineIdentityProvider;

pub struct SystemMachineIdentity;

impl MachineIdentityProvider for SystemMachineIdentity {
    fn machine_guid(&self) -> Result<String, ()> {
        read_machine_guid()
    }
}

#[cfg(windows)]
fn read_machine_guid() -> Result<String, ()> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE,
        KEY_QUERY_VALUE, REG_SZ,
    };
    const SUBKEY: &str = "SOFTWARE\\Microsoft\\Cryptography";
    const VALUE: &str = "MachineGuid";
    let subkey: Vec<u16> = OsStr::new(SUBKEY).encode_wide().chain(Some(0)).collect();
    let name: Vec<u16> = OsStr::new(VALUE).encode_wide().chain(Some(0)).collect();
    let mut key: HKEY = ptr::null_mut();
    if unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) } != 0 { return Err(()); }
    let mut kind = 0;
    let mut size = 0;
    let query = unsafe { RegQueryValueExW(key, name.as_ptr(), ptr::null(), &mut kind, ptr::null_mut(), &mut size) };
    if query != 0 || kind != REG_SZ || size < 2 || size > 1024 { unsafe { RegCloseKey(key); } return Err(()); }
    let mut value = vec![0_u16; (size as usize).div_ceil(2)];
    let query = unsafe { RegQueryValueExW(key, name.as_ptr(), ptr::null(), &mut kind, value.as_mut_ptr().cast(), &mut size) };
    unsafe { RegCloseKey(key); }
    if query != 0 || kind != REG_SZ { return Err(()); }
    let end = value.iter().position(|unit| *unit == 0).ok_or(())?;
    if end == 0 || value[end + 1..].iter().any(|unit| *unit != 0) { return Err(()); }
    String::from_utf16(&value[..end]).map_err(|_| ())
}

#[cfg(not(windows))]
fn read_machine_guid() -> Result<String, ()> { Err(()) }

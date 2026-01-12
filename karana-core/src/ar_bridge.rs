use std::os::raw::c_char;
use std::ffi::{CStr, CString};

// Mock Manifest struct for now since we don't have the full definition handy in this context
// In a real scenario, this would be imported from the core types
pub struct Manifest {
    pub text: String,
}

#[unsafe(no_mangle)]
pub extern "C" fn render_ar_whisper(text: *const c_char) {
    let c_str = unsafe { CStr::from_ptr(text) };
    let msg = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "Invalid UTF-8 string",
    };
    
    // In a real scenario, this would call into Unity via another FFI or IPC
    // For simulation, we log to stdout which the simulator can pick up
    println!("AR_WHISPER_SIM: {}", msg);
}

#[unsafe(no_mangle)]
pub extern "C" fn pin_tab_csharp(url: *const c_char, x: f32, y: f32, z: f32) {
    let c_str = unsafe { CStr::from_ptr(url) };
    let url_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "Invalid URL",
    };
    
    println!("AR_TAB_PIN_SIM: url={} pos=({}, {}, {})", url_str, x, y, z);
}

pub fn manifest_to_unity(manifest: &Manifest) {
    let text = CString::new(manifest.text.clone()).unwrap_or_default();
    unsafe { render_ar_whisper(text.as_ptr()) };
}

pub fn spawn_ar_tab(url: &str, x: f32, y: f32, z: f32) {
    let c_url = CString::new(url).unwrap_or_default();
    unsafe { pin_tab_csharp(c_url.as_ptr(), x, y, z) };
}

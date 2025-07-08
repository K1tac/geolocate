// src/lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::net::IpAddr;
use std::str::FromStr;
use reqwest::blocking::get;
use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
struct IpWhoResponse {
    success: bool,
    continent: Option<String>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    district: Option<String>,
}

#[no_mangle]
pub extern "C" fn geolocate(ip_ptr: *const c_char) -> *const c_char {
    if ip_ptr.is_null() {
        return ptr::null();
    }

    let c_str = unsafe { CStr::from_ptr(ip_ptr) };
    let ip_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Invalid UTF-8").unwrap().into_raw(),
    };

    if IpAddr::from_str(ip_str).is_err() {
        return CString::new("Invalid IP Address").unwrap().into_raw();
    }

    let url = format!("https://ipwho.is/{}", ip_str);
    let resp = match get(Url::parse(&url).unwrap()) {
        Ok(r) => r,
        Err(_) => return CString::new("Request failed").unwrap().into_raw(),
    };

    let data: IpWhoResponse = match resp.json() {
        Ok(d) => d,
        Err(_) => return CString::new("Failed to parse JSON").unwrap().into_raw(),
    };

    if !data.success {
        return CString::new("Lookup failed").unwrap().into_raw();
    }

    let result = format!(
        "Region: {}\nCountry: {}\nProvince/State: {}\nCity/Town: {}\nCounty: {}",
        data.continent.unwrap_or("Unknown".into()),
        data.country.unwrap_or("Unknown".into()),
        data.region.unwrap_or("Unknown".into()),
        data.city.unwrap_or("Unknown".into()),
        data.district.unwrap_or("Unknown".into())
    );

    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

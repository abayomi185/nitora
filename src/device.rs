use anyhow::{bail, Result};

pub const SUPPORTED_DEVICES: &[&str] = &[
    "MacBookPro18,1",
    "MacBookPro18,2",
    "MacBookPro18,3",
    "MacBookPro18,4",
    "Mac14,6",
    "Mac14,10",
    "Mac14,5",
    "Mac14,9",
    "Mac15,7",
    "Mac15,9",
    "Mac15,11",
    "Mac15,6",
    "Mac15,8",
    "Mac15,10",
    "Mac15,3",
    "Mac16,1",
    "Mac16,6",
    "Mac16,8",
    "Mac16,7",
    "Mac16,5",
    "Mac17,2",
    "Mac17,6",
    "Mac17,8",
    "Mac17,7",
    "Mac17,9",
];

pub const SDR_600_NITS_DEVICES: &[&str] = &[
    "Mac15,3", "Mac15,6", "Mac15,7", "Mac15,8", "Mac15,9", "Mac15,10", "Mac15,11", "Mac16,1",
    "Mac16,6", "Mac16,8", "Mac16,7", "Mac16,5", "Mac17,2", "Mac17,6", "Mac17,8", "Mac17,7",
    "Mac17,9",
];

pub fn get_model_identifier() -> Result<String> {
    use std::ffi::CString;

    use objc2_core_foundation::{CFData, CFDictionary, CFRetained, CFString};
    use objc2_io_kit::{
        kIOMainPortDefault, IOObjectRelease, IORegistryEntryCreateCFProperty,
        IOServiceGetMatchingService, IOServiceMatching,
    };

    let service_class = CString::new("IOPlatformExpertDevice")
        .expect("CString::new failed for IOPlatformExpertDevice");

    let matching = unsafe { IOServiceMatching(service_class.as_ptr()) };
    if matching.is_none() {
        bail!("IOServiceMatching returned NULL");
    }

    let matching_dict: Option<CFRetained<CFDictionary>> =
        matching.map(|m| unsafe { CFRetained::cast_unchecked(m) });

    let service = unsafe { IOServiceGetMatchingService(kIOMainPortDefault, matching_dict) };

    if service == 0 {
        bail!("IOServiceGetMatchingService: IOPlatformExpertDevice not found");
    }

    let key = CFString::from_str("model");
    let prop = unsafe { IORegistryEntryCreateCFProperty(service, Some(&key), None, 0) };

    IOObjectRelease(service);

    let prop = prop.ok_or_else(|| {
        anyhow::anyhow!("IORegistryEntryCreateCFProperty: 'model' property not found")
    })?;

    let data: CFRetained<CFData> = prop
        .downcast::<CFData>()
        .map_err(|_| anyhow::anyhow!("IORegistryEntryCreateCFProperty: 'model' is not CFData"))?;

    let bytes = data.to_vec();
    let model = String::from_utf8(bytes)
        .map_err(|e| anyhow::anyhow!("model identifier is not valid UTF-8: {e}"))?;
    let model = model.trim_matches('\0').to_string();

    if model.is_empty() {
        bail!("model identifier is empty");
    }

    Ok(model)
}

pub fn is_device_supported(model: &str) -> bool {
    SUPPORTED_DEVICES.contains(&model)
}

pub fn get_device_max_brightness(model: &str) -> f32 {
    if SDR_600_NITS_DEVICES.contains(&model) {
        1.535
    } else {
        1.59
    }
}

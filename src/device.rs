use anyhow::{bail, Result};

/// Retrieves the Mac model identifier (e.g. "Mac16,1") via IOKit.
/// Used only for diagnostic/status output, not for capability gating.
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
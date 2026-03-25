use anyhow::{bail, Result};

use core_graphics2::direct_display::{
    CGDisplayGammaTableCapacity, CGDisplayRestoreColorSyncSettings, CGGetDisplayTransferByTable,
    CGSetDisplayTransferByTable,
};

use core_graphics2::error::CGError;

#[derive(Debug, Clone)]
pub struct GammaTable {
    pub red: Vec<f32>,
    pub green: Vec<f32>,
    pub blue: Vec<f32>,
}

pub fn capture_gamma_table(display_id: u32) -> Result<GammaTable> {
    let capacity = unsafe { CGDisplayGammaTableCapacity(display_id) };
    if capacity == 0 {
        bail!("CGDisplayGammaTableCapacity returned 0 for display {display_id}");
    }

    let mut red = vec![0f32; capacity as usize];
    let mut green = vec![0f32; capacity as usize];
    let mut blue = vec![0f32; capacity as usize];
    let mut sample_count: u32 = 0;

    let result = unsafe {
        CGGetDisplayTransferByTable(
            display_id,
            capacity,
            red.as_mut_ptr(),
            green.as_mut_ptr(),
            blue.as_mut_ptr(),
            &mut sample_count,
        )
    };

    if result != CGError::Success {
        bail!("CGGetDisplayTransferByTable failed for display {display_id}: {result:?}");
    }

    red.truncate(sample_count as usize);
    green.truncate(sample_count as usize);
    blue.truncate(sample_count as usize);

    Ok(GammaTable { red, green, blue })
}

pub fn apply_gamma_factor(display_id: u32, table: &GammaTable, factor: f32) -> Result<()> {
    let red: Vec<f32> = table.red.iter().map(|v| v * factor).collect();
    let green: Vec<f32> = table.green.iter().map(|v| v * factor).collect();
    let blue: Vec<f32> = table.blue.iter().map(|v| v * factor).collect();

    let table_size = red.len() as u32;

    let result = unsafe {
        CGSetDisplayTransferByTable(
            display_id,
            table_size,
            red.as_ptr(),
            green.as_ptr(),
            blue.as_ptr(),
        )
    };

    if result != CGError::Success {
        bail!("CGSetDisplayTransferByTable failed for display {display_id}: {result:?}");
    }

    Ok(())
}

pub fn restore_color_sync() -> Result<()> {
    unsafe { CGDisplayRestoreColorSyncSettings() };
    Ok(())
}

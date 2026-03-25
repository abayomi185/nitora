use core_graphics2::display::get_active_display_list;

use anyhow::{Result, anyhow};

pub struct DisplayInfo {
    #[allow(dead_code)]
    pub id: u32,
    #[allow(dead_code)]
    pub built_in: bool,
    #[allow(dead_code)]
    pub name: String,
}

pub fn active_displays() -> Result<Vec<DisplayInfo>> {
    let displays =
        get_active_display_list(32).ok_or_else(|| anyhow!("CGGetActiveDisplayList failed"))?;

    let infos = displays
        .into_iter()
        .map(|d| {
            let built_in = d.is_built_in();
            let name = if built_in {
                "Built-in Display".to_owned()
            } else if d.is_main() {
                format!("Display {} (Main)", d.id)
            } else {
                format!("Display {}", d.id)
            };
            DisplayInfo {
                id: d.id,
                built_in,
                name,
            }
        })
        .collect();

    Ok(infos)
}

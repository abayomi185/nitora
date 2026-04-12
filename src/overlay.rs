use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSScreen, NSScreenSaverWindowLevel, NSView, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_core_graphics::{kCGColorSpaceExtendedLinearSRGB, CGColorSpace};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_metal::{MTLClearColor, MTLLoadAction};
use objc2_metal::{
    MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice,
    MTLDrawable, MTLPixelFormat, MTLRenderPassDescriptor, MTLStoreAction,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};

/// Fraction of available EDR headroom used for the maximum gamma boost.
/// A value of 0.75 means at brightness=100% we use 75% of the display's
/// extra dynamic range, leaving margin to avoid clipping artifacts.
const MAX_HEADROOM_FRACTION: f64 = 0.75;

#[derive(Debug, Clone)]
pub struct TargetScreen {
    pub display_id: u32,
    pub name: String,
    pub frame: NSRect,
    /// The display's full EDR headroom from
    /// `maximumPotentialExtendedDynamicRangeColorComponentValue`.
    /// Values > 1.0 indicate extended dynamic range capability.
    pub edr_headroom: f64,
}

impl TargetScreen {
    /// Maximum safe gamma multiplier for this display, derived from its
    /// hardware EDR headroom. Scales linearly within the headroom using
    /// `MAX_HEADROOM_FRACTION` as a conservative ceiling.
    pub fn max_gamma_factor(&self) -> f64 {
        (1.0 + (self.edr_headroom - 1.0) * MAX_HEADROOM_FRACTION).max(1.0)
    }

    /// Compute the gamma table multiplier for a brightness percentage (0–100).
    /// At 0% the factor is 1.0 (no boost); at 100% it equals `max_gamma_factor()`.
    pub fn gamma_factor_for_brightness(&self, brightness: u8) -> f32 {
        let max = self.max_gamma_factor();
        (1.0 + (max - 1.0) * f64::from(brightness) / 100.0) as f32
    }
}

struct OverlayWindow {
    window: Retained<NSWindow>,
    _layer: Retained<CAMetalLayer>,
}

pub struct OverlayController {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    windows: HashMap<u32, OverlayWindow>,
}

impl OverlayController {
    pub fn new() -> Result<Self> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| anyhow!("Metal is unavailable on this machine"))?;
        let command_queue = device
            .newCommandQueue()
            .ok_or_else(|| anyhow!("failed to create Metal command queue"))?;

        Ok(Self {
            device,
            command_queue,
            windows: HashMap::new(),
        })
    }

    pub fn activate(&mut self, screens: &[TargetScreen]) -> Result<()> {
        self.disable()?;

        for screen in screens {
            let overlay_window = self.create_window(screen)?;
            self.windows.insert(screen.display_id, overlay_window);
        }

        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        for overlay_window in self.windows.values() {
            overlay_window.window.close();
        }
        self.windows.clear();
        Ok(())
    }

    fn create_window(&self, screen: &TargetScreen) -> Result<OverlayWindow> {
        let mtm =
            MainThreadMarker::new().ok_or_else(|| anyhow!("must run overlay on main thread"))?;
        // Place the 1×1 overlay at the screen’s coordinate origin.
        let frame = NSRect::new(
            NSPoint::new(screen.frame.origin.x, screen.frame.origin.y),
            NSSize::new(1.0, 1.0),
        );

        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                frame,
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::Buffered,
                false,
            )
        };

        unsafe { window.setReleasedWhenClosed(false) };
        window.setOpaque(false);
        let clear = NSColor::clearColor();
        window.setBackgroundColor(Some(&clear));
        window.setIgnoresMouseEvents(true);
        window.setCanHide(false);
        window.setMovableByWindowBackground(true);
        window.setLevel(NSScreenSaverWindowLevel);
        window.setCollectionBehavior(
            NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle
                | NSWindowCollectionBehavior::CanJoinAllSpaces,
        );
        window.setFrame_display(frame, true);

        let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
        view.setWantsLayer(true);

        let layer = CAMetalLayer::new();
        layer.setDevice(Some(&self.device));
        layer.setPixelFormat(MTLPixelFormat::RGBA16Float);
        layer.setFramebufferOnly(false);
        layer.setWantsExtendedDynamicRangeContent(true);
        layer.setDrawableSize(NSSize::new(1.0, 1.0));
        layer.setOpaque(false);

        let extended_linear_srgb = unsafe { kCGColorSpaceExtendedLinearSRGB };
        if let Some(color_space) = CGColorSpace::with_name(Some(extended_linear_srgb)) {
            layer.setColorspace(Some(&color_space));
        }

        view.setLayer(Some(&layer));
        window.setContentView(Some(&view));
        window.orderFrontRegardless();

        render_layer(&self.command_queue, &layer, screen.edr_headroom)
            .with_context(|| format!("failed rendering EDR overlay for {}", screen.name))?;

        Ok(OverlayWindow {
            window,
            _layer: layer,
        })
    }
}

fn render_layer(
    command_queue: &ProtocolObject<dyn MTLCommandQueue>,
    layer: &CAMetalLayer,
    edr_headroom: f64,
) -> Result<()> {
    autoreleasepool(|_| {
        let drawable = layer
            .nextDrawable()
            .ok_or_else(|| anyhow!("CAMetalLayer returned no drawable"))?;
        let texture = drawable.texture();

        let descriptor = MTLRenderPassDescriptor::renderPassDescriptor();
        let attachments = descriptor.colorAttachments();
        let attachment = unsafe { attachments.objectAtIndexedSubscript(0) };
        attachment.setTexture(Some(&texture));
        attachment.setLoadAction(MTLLoadAction::Clear);
        attachment.setStoreAction(MTLStoreAction::Store);
        attachment.setClearColor(MTLClearColor {
            red: edr_headroom,
            green: edr_headroom,
            blue: edr_headroom,
            alpha: 1.0,
        });

        let command_buffer = command_queue
            .commandBuffer()
            .ok_or_else(|| anyhow!("failed to create Metal command buffer"))?;

        let encoder = command_buffer
            .renderCommandEncoderWithDescriptor(&descriptor)
            .ok_or_else(|| anyhow!("failed to create Metal render encoder"))?;
        encoder.endEncoding();

        let drawable_ref: &ProtocolObject<dyn MTLDrawable> = drawable.as_ref();
        command_buffer.presentDrawable(drawable_ref);
        command_buffer.commit();
        layer.display();

        Ok(())
    })
}

/// Enumerate screens that support extended dynamic range.
/// Any display (built-in or external) with EDR headroom > 1.0 is eligible.
/// This avoids maintaining a hardcoded list of device model identifiers.
pub fn collect_target_screens() -> Vec<TargetScreen> {
    let Some(mtm) = MainThreadMarker::new() else {
        return Vec::new();
    };

    let screens = NSScreen::screens(mtm);
    let mut targets = Vec::new();

    for screen in screens.iter() {
        let display_id = screen.CGDirectDisplayID();
        let potential_edr = screen.maximumPotentialExtendedDynamicRangeColorComponentValue();

        if potential_edr > 1.0 {
            let name: String = screen.localizedName().to_string();
            targets.push(TargetScreen {
                display_id,
                name,
                frame: screen.frame(),
                edr_headroom: potential_edr,
            });
        }
    }

    targets
}

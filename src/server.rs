use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread;

use anyhow::{anyhow, Context, Result};

use objc2::rc::autoreleasepool;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSEventMask};
use objc2_foundation::{NSDate, NSDefaultRunLoopMode};

use crate::gamma::GammaTable;
use crate::ipc::{Request, Response};

const BACKEND_NAME: &str = "appkit-metal+coregraphics";

struct CommandEnvelope {
    request: Request,
    responder: SyncSender<Response>,
}

struct ServiceState {
    enabled: bool,
    brightness: u8,
    backend: BackendState,
}

struct BackendState {
    overlay: crate::overlay::OverlayController,
    gamma_tables: HashMap<u32, GammaTable>,
    /// Retained for status/diagnostic output only.
    model_identifier: Option<String>,
}

impl ServiceState {
    fn new(brightness: u8) -> Result<Self> {
        Ok(Self {
            enabled: false,
            brightness,
            backend: BackendState::new()?,
        })
    }

    fn handle_request(&mut self, request: Request) -> Response {
        match request {
            Request::Enable => match self.backend.apply(true, self.brightness) {
                Ok(target_count) => {
                    self.enabled = true;
                    response(format!("Enabled on {target_count} display(s)"), self, false)
                }
                Err(error) => {
                    self.enabled = false;
                    response(format!("Enable failed: {error:#}"), self, true)
                }
            },
            Request::Disable => match self.backend.apply(false, self.brightness) {
                Ok(_) => {
                    self.enabled = false;
                    response("Disabled".to_owned(), self, false)
                }
                Err(error) => response(format!("Disable failed: {error:#}"), self, true),
            },
            Request::Toggle => {
                if self.enabled {
                    match self.backend.apply(false, self.brightness) {
                        Ok(_) => {
                            self.enabled = false;
                            response("Disabled".to_owned(), self, false)
                        }
                        Err(error) => response(format!("Disable failed: {error:#}"), self, true),
                    }
                } else {
                    match self.backend.apply(true, self.brightness) {
                        Ok(target_count) => {
                            self.enabled = true;
                            response(format!("Enabled on {target_count} display(s)"), self, false)
                        }
                        Err(error) => response(format!("Enable failed: {error:#}"), self, true),
                    }
                }
            }
            Request::Set { value } => {
                self.brightness = value;
                if self.enabled {
                    match self.backend.apply(true, self.brightness) {
                        Ok(target_count) => response(
                            format!("Brightness set to {value}% on {target_count} display(s)"),
                            self,
                            false,
                        ),
                        Err(error) => {
                            response(format!("Brightness update failed: {error:#}"), self, true)
                        }
                    }
                } else {
                    response(format!("Brightness set to {value}%"), self, false)
                }
            }
            Request::Status => {
                let model = self
                    .backend
                    .model_identifier
                    .as_deref()
                    .unwrap_or("unknown");
                let eligible_count = self.backend.target_count();
                let active_count = crate::display::active_displays()
                    .map(|displays| displays.len())
                    .unwrap_or(0);
                response(
                    format!(
                        "Service ready. model={model}, active_displays={active_count}, eligible_displays={eligible_count}"
                    ),
                    self,
                    true,
                )
            }
        }
    }
}

impl BackendState {
    fn new() -> Result<Self> {
        let model_identifier = crate::device::get_model_identifier().ok();

        Ok(Self {
            overlay: crate::overlay::OverlayController::new()?,
            gamma_tables: HashMap::new(),
            model_identifier,
        })
    }

    fn apply(&mut self, enabled: bool, brightness: u8) -> Result<usize> {
        if !enabled {
            self.overlay.disable()?;
            crate::gamma::restore_color_sync()?;
            self.gamma_tables.clear();
            return Ok(0);
        }

        let targets = crate::overlay::collect_target_screens();
        if targets.is_empty() {
            self.overlay.disable()?;
            crate::gamma::restore_color_sync()?;
            self.gamma_tables.clear();
            return Err(anyhow!("no eligible EDR/XDR displays detected"));
        }

        self.overlay.activate(&targets)?;
        crate::gamma::restore_color_sync()?;
        self.gamma_tables.clear();

        // Each display gets its own gamma factor derived from its EDR headroom.
        for target in &targets {
            let factor = target.gamma_factor_for_brightness(brightness);
            let table = crate::gamma::capture_gamma_table(target.display_id)
                .with_context(|| format!("failed capturing gamma table for {}", target.name))?;
            crate::gamma::apply_gamma_factor(target.display_id, &table, factor)
                .with_context(|| format!("failed applying gamma factor to {}", target.name))?;
            self.gamma_tables.insert(target.display_id, table);
        }

        Ok(targets.len())
    }

    fn target_count(&self) -> usize {
        crate::overlay::collect_target_screens().len()
    }
}

pub fn run(auto_activate: bool, brightness: u8) -> Result<()> {
    let mtm = MainThreadMarker::new().ok_or_else(|| anyhow!("must run on the main thread"))?;
    let app = NSApplication::sharedApplication(mtm);
    if !app.setActivationPolicy(NSApplicationActivationPolicy::Accessory) {
        return Err(anyhow!("failed to set NSApplication activation policy"));
    }
    app.finishLaunching();

    let (command_tx, command_rx) = mpsc::channel::<CommandEnvelope>();
    let _ipc_thread = spawn_ipc_thread(command_tx)?;
    let mut state = ServiceState::new(brightness)?;

    if auto_activate {
        if let Err(e) = state.backend.apply(true, state.brightness) {
            eprintln!("Auto-enable failed: {e:#}");
        } else {
            state.enabled = true;
        }
    }

    loop {
        drain_commands(&command_rx, &mut state);
        pump_appkit_once(&app);
    }
}

fn spawn_ipc_thread(command_tx: Sender<CommandEnvelope>) -> Result<thread::JoinHandle<()>> {
    let listener = crate::ipc::create_listener()?;

    println!(
        "Nitora service listening on {}",
        crate::ipc::socket_path().display()
    );

    Ok(thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(error) = handle_connection(stream, &command_tx) {
                        eprintln!("IPC error: {error:#}");
                    }
                }
                Err(error) => {
                    eprintln!("Socket accept error: {error}");
                }
            }
        }
    }))
}

fn handle_connection(stream: UnixStream, command_tx: &Sender<CommandEnvelope>) -> Result<()> {
    let request = crate::ipc::read_json_line::<Request>(stream.try_clone()?)?;
    let (response_tx, response_rx) = mpsc::sync_channel(1);

    command_tx
        .send(CommandEnvelope {
            request,
            responder: response_tx,
        })
        .context("service loop is unavailable")?;

    let response = response_rx
        .recv()
        .context("service loop dropped response channel")?;

    let mut stream = stream;
    crate::ipc::write_json_line(&mut stream, &response)?;
    Ok(())
}

fn drain_commands(command_rx: &Receiver<CommandEnvelope>, state: &mut ServiceState) {
    while let Ok(envelope) = command_rx.try_recv() {
        let response = state.handle_request(envelope.request);
        let _ = envelope.responder.send(response);
    }
}

fn pump_appkit_once(app: &NSApplication) {
    autoreleasepool(|_| {
        let until = NSDate::dateWithTimeIntervalSinceNow(0.02);
        let run_loop_mode = unsafe { NSDefaultRunLoopMode };
        if let Some(event) = app.nextEventMatchingMask_untilDate_inMode_dequeue(
            NSEventMask::Any,
            Some(&until),
            run_loop_mode,
            true,
        ) {
            app.sendEvent(&event);
        }
        app.updateWindows();
    });
}

fn response(message: String, state: &ServiceState, show_state: bool) -> Response {
    Response {
        message,
        enabled: state.enabled,
        brightness: state.brightness,
        backend: BACKEND_NAME.to_owned(),
        show_state,
    }
}

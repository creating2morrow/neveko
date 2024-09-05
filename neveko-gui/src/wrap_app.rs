#[cfg(feature = "glow")]
use eframe::glow;
use neveko_core::*;

use std::sync::mpsc::{
    Receiver,
    Sender,
};

// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Anchor {
    App,
    Home,
    AddressBook,
    MailBox,
    Market,
    Settings,
    Wallet,
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<Anchor> for egui::WidgetText {
    fn from(value: Anchor) -> Self {
        Self::RichText(egui::RichText::new(value.to_string()))
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::App
    }
}

// ----------------------------------------------------------------------------

/// The state
pub struct State {
    // security state
    app_init_lock: bool,
    is_cred_set: bool,
    is_checking_cred: bool,
    is_screen_locked: bool,
    is_screen_locking: bool,
    // app state
    home: crate::apps::HomeApp,
    address_book: crate::apps::AddressBookApp,
    lock_screen: crate::apps::LockScreenApp,
    lock_timer: u64,
    login: crate::login::LoginApp,
    mailbox: crate::apps::MailBoxApp,
    market: crate::apps::MarketApp,
    selected_anchor: Anchor,
    settings: crate::apps::SettingsApp,
    wallet: crate::apps::WalletApp,
    // async notifications
    is_screen_locked_tx: Sender<bool>,
    is_screen_locked_rx: Receiver<bool>,
    is_cred_set_tx: Sender<bool>,
    is_cred_set_rx: Receiver<bool>,
    lock_timer_tx: Sender<bool>,
    lock_timer_rx: Receiver<bool>,
    // end async notifications
}

impl Default for State {
    fn default() -> Self {
        let (is_screen_locked_tx, is_screen_locked_rx) = std::sync::mpsc::channel();
        let (lock_timer_tx, lock_timer_rx) = std::sync::mpsc::channel();
        let (is_cred_set_tx, is_cred_set_rx) = std::sync::mpsc::channel();
        Self {
            home: Default::default(),
            address_book: Default::default(),
            app_init_lock: true,
            lock_screen: Default::default(),
            is_cred_set: false,
            is_checking_cred: true,
            is_screen_locked: false,
            is_screen_locking: false,
            lock_timer: 0,
            lock_timer_rx,
            lock_timer_tx,
            login: Default::default(),
            mailbox: Default::default(),
            market: Default::default(),
            selected_anchor: Anchor::Home,
            settings: Default::default(),
            wallet: Default::default(),
            // async notifications
            is_screen_locked_tx,
            is_screen_locked_rx,
            is_cred_set_tx,
            is_cred_set_rx,
        }
    }
}

/// Wraps many apps into one.
pub struct WrapApp {
    state: State,
}

impl WrapApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut slf = Self {
            state: State::default(),
        };
        slf
    }

    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
        let vec = vec![
            (
                "Home",
                Anchor::Home,
                &mut self.state.home as &mut dyn eframe::App,
            ),
            (
                "Address Book",
                Anchor::AddressBook,
                &mut self.state.address_book as &mut dyn eframe::App,
            ),
            (
                "Mailbox",
                Anchor::MailBox,
                &mut self.state.mailbox as &mut dyn eframe::App,
            ),
            (
                "Market",
                Anchor::Market,
                &mut self.state.market as &mut dyn eframe::App,
            ),
            (
                "Wallet",
                Anchor::Wallet,
                &mut self.state.wallet as &mut dyn eframe::App,
            ),
            (
                "Settings",
                Anchor::Settings,
                &mut self.state.settings as &mut dyn eframe::App,
            ),
        ];
        vec.into_iter()
    }
}

impl eframe::App for WrapApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // NOTE: just moving the mouse wont reset the lock screen timer
        if ctx.is_using_pointer() {
            self.state.lock_timer = 0;
        }
        if let Ok(cred_set) = self.state.is_cred_set_rx.try_recv() {
            self.state.is_cred_set = cred_set;
        }
        if let Ok(lock) = self.state.is_screen_locked_rx.try_recv() {
            self.state.is_screen_locked = lock;
            if lock {
                let lock_screen = &mut self.state.lock_screen;
                if self.state.lock_timer >= crate::LOCK_SCREEN_TIMEOUT_SECS {
                    lock_screen.set_lock();
                }
                self.state.is_screen_locking = false;
            }
        }
        if let Ok(lock_timer) = self.state.lock_timer_rx.try_recv() {
            if lock_timer {
                self.state.lock_timer += 1
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            frame.set_fullscreen(!frame.info().window_info.fullscreen);
        }

        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            egui::trace!(ui);
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.bar_contents(ui, frame);
            });
        });
        // credential set logic
        if self.state.is_checking_cred {
            self.check_credential_key(self.state.is_cred_set_tx.clone(), ctx.clone());
            self.state.is_checking_cred = false;
        }
        // screen lock logic
        let app_initializing = self.state.app_init_lock;
        if app_initializing {
            send_inc_lock_timer_req(self.state.lock_timer_tx.clone(), ctx.clone());
        }
        if (!self.state.is_screen_locking && self.state.is_cred_set) || app_initializing {
            self.send_lock_refresh(
                self.state.is_screen_locked_tx.clone(),
                ctx.clone(),
                app_initializing,
            );
            self.state.is_screen_locking = true;
        }
        self.show_selected_app(ctx, frame);

        // On web, the browser controls `pixels_per_point`.
        if !frame.is_web() {
            egui::gui_zoom::zoom_with_keyboard_shortcuts(ctx, frame.info().native_pixels_per_point);
        }
    }

    #[cfg(feature = "glow")]
    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        // sanity check that wallet password is no longer in user environment
        std::env::set_var(neveko_core::MONERO_WALLET_PASSWORD, "");
        let is_bg = std::env::var(crate::NEVEKO_DEV_BACKGROUND).unwrap_or(String::from("0"));
        if is_bg != String::from("1") {
            utils::kill_child_processes(false);
        }
    }
}

impl WrapApp {
    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        // initial cred check, is there a better way to do this?
        if !self.state.is_cred_set {
            let s = db::DatabaseEnvironment::open(&utils::get_release_env().value()).unwrap();
            let r = db::DatabaseEnvironment::read(&s.env, &s.handle, crate::CREDENTIAL_KEY);
            if !r.is_empty() {
                self.state.is_cred_set = true;
                self.state.is_checking_cred = false;
            }
        }
        let selected_anchor = self.state.selected_anchor;
        let lock_screen = &mut self.state.lock_screen;
        let is_screen_locked = lock_screen.get_lock_status();
        let lock_app = lock_screen as &mut dyn eframe::App;
        let login_screen = &mut self.state.login as &mut dyn eframe::App;
        if self.state.is_cred_set {
            if self.state.is_screen_locked && is_screen_locked || self.state.app_init_lock {
                lock_app.update(ctx, frame);
                self.state.app_init_lock = false;
            } else {
                for (_name, anchor, app) in self.apps_iter_mut() {
                    if anchor == selected_anchor || ctx.memory(|mem| mem.everything_is_visible()) {
                        app.update(ctx, frame);
                    }
                }
            }
        } else {
            login_screen.update(ctx, frame);
        }
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        let mut selected_anchor = self.state.selected_anchor;
        for (name, anchor, _app) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
                if frame.is_web() {
                    ui.output_mut(|o| o.open_url(format!("#{}", anchor)));
                }
            }
        }
        self.state.selected_anchor = selected_anchor;

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::warn_if_debug_build(ui);
        });
    }

    // refresh rate for the home screen
    fn send_lock_refresh(&mut self, tx: Sender<bool>, ctx: egui::Context, init: bool) {
        tokio::spawn(async move {
            log::debug!("locking screen");
            if !init {
                tokio::time::sleep(std::time::Duration::from_secs(
                    crate::LOCK_SCREEN_TIMEOUT_SECS,
                ))
                .await;
            }
            let _ = tx.send(true);
            ctx.request_repaint();
        });
    }
    /// Validate that a credential was set by the user;
    fn check_credential_key(&mut self, tx: Sender<bool>, ctx: egui::Context) {
        tokio::spawn(async move {
            loop {
                log::debug!("check for cred");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                let s = db::DatabaseEnvironment::open(&utils::get_release_env().value()).unwrap();
                let r = db::DatabaseEnvironment::read(&s.env, &s.handle, crate::CREDENTIAL_KEY);
                if r.is_empty() {
                    log::debug!("credential not found");
                    let _ = tx.send(false);
                    ctx.request_repaint();
                } else {
                    let _ = tx.send(true);
                    ctx.request_repaint();
                    break;
                }
            }
        });
    }
}

/// When the pointer goes 'active' (i.e. pushing a button, dragging
///
/// a slider, etc) reset it. Otherwise this function runs forever
///
/// incrementing by one every second. Once this timer matches the
///
/// `LOCK_SCREEN_TIMEOUT_SECS` constant the lock screen will trigger.
fn send_inc_lock_timer_req(tx: Sender<bool>, ctx: egui::Context) {
    tokio::spawn(async move {
        log::debug!("starting the lock screen timer");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let _ = tx.send(true);
            ctx.request_repaint();
        }
    });
}

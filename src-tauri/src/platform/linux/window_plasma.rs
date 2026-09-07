//! Aktives Fenster unter KDE Plasma auf Wayland.
//!
//! Wayland hat kein Standardprotokoll, über das ein normaler Client das
//! fokussierte Fenster samt PID erfährt: `wlr-foreign-toplevel-management`
//! kennt keine PID, und `xdg-foreign` betrifft nur eigene Fenster. KWin bietet
//! dafür `org_kde_plasma_window_management` an – die Windows-Entsprechung von
//! `GetForegroundWindow` + `GetWindowThreadProcessId`.
//!
//! Das Protokoll ist laut eigener Beschreibung ein Implementierungsdetail der
//! Desktop-Umgebung und kann sich zwischen Plasma-Versionen ändern. Deshalb
//! wird hier defensiv gebunden: schlägt das fehl, meldet [`is_available`]
//! `false` und der Aufrufer wählt ein anderes Backend.
//!
//! Ein Hintergrund-Thread hält den Zustand aller Fenster aktuell und schreibt
//! das jeweils aktive in einen geteilten Cache. [`active_window`] liest nur
//! diesen Cache, kostet also keinen Roundtrip.

use crate::platform::ActiveWindow;
use log::{debug, warn};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::thread;
use wayland_client::backend::ObjectId;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols_plasma::plasma_window_management::client::org_kde_plasma_window::{
    Event as WindowEvent, OrgKdePlasmaWindow,
};
use wayland_protocols_plasma::plasma_window_management::client::org_kde_plasma_window_management::{
    Event as ManagementEvent, OrgKdePlasmaWindowManagement,
};

/// `active`-Bit aus `org_kde_plasma_window_management.state`.
const STATE_ACTIVE: u32 = 0x1;
/// Ab dieser Version gibt es `window_with_uuid`/`get_window_by_uuid`.
const UUID_SINCE: u32 = 13;
/// Ab dieser Version besitzt `org_kde_plasma_window` einen Destruktor.
const DESTROY_SINCE: u32 = 4;
/// Höchste Protokollversion, die wir kennen.
const MAX_VERSION: u32 = 18;

fn cache() -> &'static Mutex<Option<ActiveWindow>> {
    static CACHE: OnceLock<Mutex<Option<ActiveWindow>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

#[derive(Default)]
struct WindowInfo {
    pid: Option<u32>,
    app_id: Option<String>,
    active: bool,
}

struct Tracker {
    windows: HashMap<ObjectId, WindowInfo>,
    active: Option<ObjectId>,
    cache: &'static Mutex<Option<ActiveWindow>>,
}

impl Tracker {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            active: None,
            cache: cache(),
        }
    }

    /// Schreibt das aktuell aktive Fenster in den geteilten Cache.
    fn publish(&self) {
        let active = self
            .active
            .as_ref()
            .and_then(|id| self.windows.get(id))
            .map(|info| ActiveWindow {
                pid: info.pid,
                app_id: info.app_id.clone(),
            })
            .filter(|active| !active.is_empty());

        if let Ok(mut guard) = self.cache.lock() {
            *guard = active;
        }
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for Tracker {
    fn event(
        _tracker: &mut Self,
        _registry: &WlRegistry,
        _event: <WlRegistry as Proxy>::Event,
        _data: &GlobalListContents,
        _connection: &Connection,
        _handle: &QueueHandle<Self>,
    ) {
        // Globals interessieren nur beim Start, das übernimmt registry_queue_init.
    }
}

impl Dispatch<OrgKdePlasmaWindowManagement, ()> for Tracker {
    fn event(
        tracker: &mut Self,
        management: &OrgKdePlasmaWindowManagement,
        event: ManagementEvent,
        _data: &(),
        _connection: &Connection,
        handle: &QueueHandle<Self>,
    ) {
        // KWin schickt je nach ausgehandelter Version entweder `window` oder
        // `window_with_uuid` – nie beides.
        let window = match event {
            ManagementEvent::WindowWithUuid { uuid, .. } => {
                Some(management.get_window_by_uuid(uuid, handle, ()))
            }
            ManagementEvent::Window { id } if management.version() < UUID_SINCE => {
                Some(management.get_window(id, handle, ()))
            }
            _ => None,
        };

        if let Some(window) = window {
            tracker.windows.insert(window.id(), WindowInfo::default());
        }
    }
}

impl Dispatch<OrgKdePlasmaWindow, ()> for Tracker {
    fn event(
        tracker: &mut Self,
        window: &OrgKdePlasmaWindow,
        event: WindowEvent,
        _data: &(),
        _connection: &Connection,
        _handle: &QueueHandle<Self>,
    ) {
        let id = window.id();

        match event {
            WindowEvent::PidChanged { pid } => {
                tracker.windows.entry(id).or_default().pid = (pid != 0).then_some(pid);
            }
            WindowEvent::AppIdChanged { app_id } => {
                tracker.windows.entry(id).or_default().app_id =
                    (!app_id.is_empty()).then_some(app_id);
            }
            WindowEvent::StateChanged { flags } => {
                let active = flags & STATE_ACTIVE != 0;
                tracker.windows.entry(id.clone()).or_default().active = active;

                if active {
                    tracker.active = Some(id);
                } else if tracker.active.as_ref() == Some(&id) {
                    tracker.active = None;
                }
            }
            WindowEvent::Unmapped => {
                tracker.windows.remove(&id);
                if tracker.active.as_ref() == Some(&id) {
                    tracker.active = None;
                }
                if window.version() >= DESTROY_SINCE {
                    window.destroy();
                }
            }
            _ => return,
        }

        tracker.publish();
    }
}

/// Baut die Verbindung auf und startet den Beobachter-Thread.
///
/// Der Bind läuft absichtlich synchron: nur so weiß der Aufrufer, ob der
/// Compositor das Protokoll überhaupt anbietet.
pub fn is_available() -> bool {
    static STARTED: OnceLock<bool> = OnceLock::new();
    *STARTED.get_or_init(start)
}

fn start() -> bool {
    let Ok(connection) = Connection::connect_to_env() else {
        debug!("Keine Wayland-Verbindung möglich");
        return false;
    };

    let Ok((globals, mut queue)) = registry_queue_init::<Tracker>(&connection) else {
        debug!("Wayland-Registry konnte nicht gelesen werden");
        return false;
    };

    let management = match globals.bind::<OrgKdePlasmaWindowManagement, _, _>(
        &queue.handle(),
        1..=MAX_VERSION,
        (),
    ) {
        Ok(management) => management,
        Err(error) => {
            debug!(
                "org_kde_plasma_window_management nicht verfügbar: {}",
                error
            );
            return false;
        }
    };

    let mut tracker = Tracker::new();

    // Ein Roundtrip liefert die bereits offenen Fenster und deckt auf, wenn der
    // Compositor den Bind mit einem Protokollfehler ablehnt.
    if let Err(error) = queue.roundtrip(&mut tracker) {
        debug!("Plasma-Window-Management abgelehnt: {}", error);
        return false;
    }
    // Zweiter Roundtrip: die Fensterobjekte aus dem ersten senden ihre
    // pid/app_id/state-Events erst jetzt.
    let _ = queue.roundtrip(&mut tracker);

    thread::Builder::new()
        .name("novadeck-plasma-windows".to_owned())
        .spawn(move || {
            // Beide Objekte müssen am Leben bleiben, sonst bricht die
            // Verbindung ab bzw. der Server hört auf, Events zu senden.
            let _connection = connection;
            let _management = management;

            while queue.blocking_dispatch(&mut tracker).is_ok() {}

            warn!("Beobachtung der Plasma-Fenster beendet");
            if let Ok(mut guard) = cache().lock() {
                *guard = None;
            }
        })
        .is_ok()
}

pub fn active_window() -> Option<ActiveWindow> {
    cache().lock().ok()?.clone()
}

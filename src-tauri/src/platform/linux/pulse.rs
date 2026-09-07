//! Verbindung zum PulseAudio-Protokoll – bedient sowohl PipeWire (über
//! `pipewire-pulse`) als auch klassisches PulseAudio.
//!
//! `libpulse` ist nicht `Send`: Mainloop und Context enthalten rohe Zeiger und
//! werden über `Rc<RefCell<…>>` gehalten. Deshalb lebt die Verbindung in einem
//! eigenen Worker-Thread, der Anfragen über einen Kanal entgegennimmt. Alle
//! öffentlichen Funktionen hier sind blockierend, aber von jedem Thread aus
//! aufrufbar.
//!
//! Snapshots werden zwischengespeichert: ein Subscribe-Callback markiert den
//! Cache als veraltet, sodass das 200-ms-Polling in [`crate::monitor`] im
//! Ruhezustand keine Introspect-Roundtrips auslöst.

use crate::platform::{PlatformError, Result};
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::subscribe::InterestMaskSet;
use libpulse_binding::context::{Context, FlagSet, State};
use libpulse_binding::mainloop::threaded::Mainloop;
use libpulse_binding::operation::{Operation, State as OperationState};
use libpulse_binding::proplist::{properties, Proplist};
use libpulse_binding::volume::{ChannelVolumes, Volume};
use log::{info, warn};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

/// Wie lange ein Aufrufer auf die Antwort des Worker-Threads wartet.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
/// Zeitfenster für den Verbindungsaufbau zum Server.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
/// Frühestens nach dieser Pause wird ein erneuter Verbindungsversuch gestartet.
const RECONNECT_INTERVAL: Duration = Duration::from_secs(2);
/// Auch ohne Subscribe-Event wird der Cache nach dieser Zeit erneuert.
const CACHE_MAX_AGE: Duration = Duration::from_secs(1);

// --- Snapshot-Typen (bewusst `Send`, damit sie über den Kanal passen) ---

#[derive(Debug, Clone)]
pub struct SinkSnapshot {
    /// Technischer Sink-Name, dient als Geräte-ID *und* zum Ansprechen des
    /// Geräts – siehe [`set_sink_volume`].
    pub name: String,
    /// Anzeigename für das Frontend.
    pub description: String,
    pub volume: ChannelVolumes,
    pub muted: bool,
}

#[derive(Debug, Clone)]
pub struct SinkInputSnapshot {
    pub index: u32,
    pub pid: Option<u32>,
    pub binary: Option<String>,
    pub app_name: Option<String>,
    pub media_name: Option<String>,
    pub volume: ChannelVolumes,
    pub muted: bool,
    pub volume_writable: bool,
}

impl SinkInputSnapshot {
    /// Entspricht dem WASAPI-Session-Identifier: ein zusammengesetzter,
    /// klein geschriebener String, gegen den Slot-Namen per Teilstring geprüft
    /// werden.
    pub fn identifier(&self) -> String {
        [&self.binary, &self.app_name, &self.media_name]
            .into_iter()
            .flatten()
            .filter(|value| !value.is_empty())
            .map(|value| value.to_lowercase())
            .collect::<Vec<_>>()
            .join("|")
    }

    /// Prüft einen bereits normalisierten Namens-Hinweis gegen Binary und
    /// Anwendungsnamen. Der Medienname (z. B. ein Songtitel) bleibt hier außen
    /// vor, damit ein Hinweis wie „spotify“ nicht zufällig in einem Titel trifft.
    pub fn matches_hint(&self, hint: &str) -> bool {
        if hint.is_empty() {
            return false;
        }
        [&self.binary, &self.app_name].iter().any(|value| {
            value
                .as_deref()
                .map(str::to_lowercase)
                .is_some_and(|value| value.contains(hint) || hint.contains(&value))
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub default_sink_name: Option<String>,
    pub sinks: Vec<SinkSnapshot>,
    pub inputs: Vec<SinkInputSnapshot>,
}

impl Snapshot {
    pub fn default_sink(&self) -> Option<&SinkSnapshot> {
        let name = self.default_sink_name.as_deref()?;
        self.sinks.iter().find(|sink| sink.name == name)
    }
}

// --- Lautstärke-Umrechnung ---

/// PulseAudio rechnet in `pa_volume_t`; 100 % entspricht [`Volume::NORMAL`].
/// Das ist dieselbe Skala, die `pactl` und `wpctl` als Prozent anzeigen.
pub fn volume_to_percent(volume: &ChannelVolumes) -> i32 {
    let normal = Volume::NORMAL.0 as f64;
    ((volume.avg().0 as f64 / normal) * 100.0).round() as i32
}

/// Skaliert alle Kanäle so, dass der lauteste `percent` erreicht. Damit bleibt
/// eine eingestellte Balance zwischen den Kanälen erhalten.
pub fn volume_with_percent(volume: &ChannelVolumes, percent: i32) -> ChannelVolumes {
    let target = Volume(
        ((Volume::NORMAL.0 as f64) * (percent.clamp(0, 100) as f64) / 100.0).round() as u32,
    );

    let mut scaled = *volume;
    if scaled.len() == 0 {
        scaled.set(2, target);
        return scaled;
    }
    scaled.scale(target);
    scaled
}

// --- Worker-Protokoll ---

enum Request {
    Snapshot(Sender<Result<Snapshot>>),
    SetSinkInputVolume {
        index: u32,
        volume: ChannelVolumes,
        reply: Sender<Result<()>>,
    },
    SetSinkInputMute {
        index: u32,
        mute: bool,
        reply: Sender<Result<()>>,
    },
    SetSinkVolume {
        name: String,
        volume: ChannelVolumes,
        reply: Sender<Result<()>>,
    },
    SetSinkMute {
        name: String,
        mute: bool,
        reply: Sender<Result<()>>,
    },
    SetDefaultSink {
        name: String,
        reply: Sender<Result<()>>,
    },
}

impl Request {
    /// Beantwortet die Anfrage mit einem Fehler, wenn keine Verbindung besteht.
    fn fail(self, error: &PlatformError) {
        match self {
            Request::Snapshot(reply) => {
                let _ = reply.send(Err(error.clone()));
            }
            Request::SetSinkInputVolume { reply, .. }
            | Request::SetSinkInputMute { reply, .. }
            | Request::SetSinkVolume { reply, .. }
            | Request::SetSinkMute { reply, .. }
            | Request::SetDefaultSink { reply, .. } => {
                let _ = reply.send(Err(error.clone()));
            }
        }
    }
}

static WORKER: OnceLock<Sender<Request>> = OnceLock::new();

fn worker() -> &'static Sender<Request> {
    WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("novadeck-pulse".to_owned())
            .spawn(move || run_worker(rx))
            .expect("PulseAudio-Thread konnte nicht gestartet werden");
        tx
    })
}

/// Startet den Worker-Thread, falls er noch nicht läuft.
pub fn ensure_started() {
    let _ = worker();
}

fn request<T>(build: impl FnOnce(Sender<Result<T>>) -> Request) -> Result<T> {
    let (reply_tx, reply_rx) = mpsc::channel();

    worker()
        .send(build(reply_tx))
        .map_err(|_| PlatformError::new("PulseAudio-Thread ist beendet"))?;

    reply_rx
        .recv_timeout(REQUEST_TIMEOUT)
        .map_err(|_| PlatformError::new("Zeitüberschreitung bei der PulseAudio-Anfrage"))?
}

pub fn snapshot() -> Result<Snapshot> {
    request(Request::Snapshot)
}

pub fn set_sink_input_volume(index: u32, volume: ChannelVolumes) -> Result<()> {
    request(|reply| Request::SetSinkInputVolume {
        index,
        volume,
        reply,
    })
}

pub fn set_sink_input_mute(index: u32, mute: bool) -> Result<()> {
    request(|reply| Request::SetSinkInputMute { index, mute, reply })
}

/// Geräte werden über den Namen und nicht über den Index angesprochen: der
/// PipeWire-Pulse-Server kann mehrere Knoten mit demselben Sink-Namen führen
/// (etwa nach HDMI-Hotplug). Nur der Server weiß, welcher davon gerade das
/// Standardgerät ist.
pub fn set_sink_volume(name: &str, volume: ChannelVolumes) -> Result<()> {
    request(|reply| Request::SetSinkVolume {
        name: name.to_owned(),
        volume,
        reply,
    })
}

pub fn set_sink_mute(name: &str, mute: bool) -> Result<()> {
    request(|reply| Request::SetSinkMute {
        name: name.to_owned(),
        mute,
        reply,
    })
}

/// Setzt den Standard-Sink und zieht laufende Streams mit auf das neue Gerät.
pub fn set_default_sink(name: &str) -> Result<()> {
    request(|reply| Request::SetDefaultSink {
        name: name.to_owned(),
        reply,
    })
}

fn run_worker(rx: Receiver<Request>) {
    let mut connection: Option<Connection> = None;
    let mut last_attempt: Option<Instant> = None;

    while let Ok(request) = rx.recv() {
        // Verbindung nach einem Server-Neustart (z. B. PipeWire-Restart) verwerfen
        if connection.as_ref().is_some_and(|c| !c.is_ready()) {
            warn!("Verbindung zum Audio-Server verloren, verbinde neu");
            crate::diagnostics::record_audio_backend_error();
            connection = None;
        }

        if connection.is_none()
            && last_attempt.is_none_or(|at| at.elapsed() >= RECONNECT_INTERVAL)
        {
            last_attempt = Some(Instant::now());
            match Connection::open() {
                Ok(open) => {
                    info!("Mit dem Audio-Server verbunden (PulseAudio-Protokoll)");
                    crate::diagnostics::record_audio_backend_connect();
                    connection = Some(open);
                }
                Err(error) => {
                    warn!("Verbindung zum Audio-Server fehlgeschlagen: {}", error);
                    crate::diagnostics::record_audio_backend_error();
                }
            }
        }

        match connection.as_mut() {
            Some(connection) => connection.handle(request),
            None => request.fail(&PlatformError::new(
                "keine Verbindung zum Audio-Server (PulseAudio/PipeWire)",
            )),
        }
    }
}

// --- Verbindung ---

/// Hält den Mainloop-Lock für die Dauer eines Blocks. Ohne den Lock darf keine
/// libpulse-Funktion aufgerufen werden, während der Mainloop-Thread läuft.
struct MainloopLock<'a> {
    mainloop: &'a Rc<RefCell<Mainloop>>,
}

impl<'a> MainloopLock<'a> {
    fn acquire(mainloop: &'a Rc<RefCell<Mainloop>>) -> Self {
        mainloop.borrow_mut().lock();
        Self { mainloop }
    }
}

impl Drop for MainloopLock<'_> {
    fn drop(&mut self) {
        self.mainloop.borrow_mut().unlock();
    }
}

struct Connection {
    mainloop: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,
    /// Wird vom Subscribe-Callback auf dem libpulse-Thread gesetzt.
    dirty: Arc<AtomicBool>,
    cache: Option<Snapshot>,
    fetched_at: Option<Instant>,
}

impl Connection {
    fn open() -> Result<Self> {
        let mut proplist =
            Proplist::new().ok_or_else(|| PlatformError::new("Proplist-Allokation fehlgeschlagen"))?;
        let _ = proplist.set_str(properties::APPLICATION_NAME, "NovaDeck");
        let _ = proplist.set_str(properties::APPLICATION_ID, "novadeck");

        let mainloop = Rc::new(RefCell::new(
            Mainloop::new().ok_or_else(|| PlatformError::new("Mainloop-Allokation fehlgeschlagen"))?,
        ));

        let context = {
            let borrowed = mainloop.borrow();
            Rc::new(RefCell::new(
                Context::new_with_proplist(&*borrowed, "NovaDeck", &proplist).ok_or_else(|| {
                    PlatformError::new("PulseAudio-Context konnte nicht erzeugt werden")
                })?,
            ))
        };

        context
            .borrow_mut()
            .connect(None, FlagSet::NOFLAGS, None)
            .map_err(|e| PlatformError::new(format!("connect fehlgeschlagen: {}", e)))?;

        mainloop
            .borrow_mut()
            .start()
            .map_err(|e| PlatformError::new(format!("Mainloop-Start fehlgeschlagen: {}", e)))?;

        // Auf `Ready` warten. Bewusst mit Deadline gepollt statt über
        // `Mainloop::wait()`: ein hängender Handshake würde den Worker-Thread
        // sonst dauerhaft blockieren.
        let deadline = Instant::now() + CONNECT_TIMEOUT;
        loop {
            let state = {
                let _lock = MainloopLock::acquire(&mainloop);
                context.borrow().get_state()
            };

            match state {
                State::Ready => break,
                State::Failed | State::Terminated => {
                    mainloop.borrow_mut().stop();
                    return Err(PlatformError::new("Audio-Server hat die Verbindung abgelehnt"));
                }
                _ if Instant::now() >= deadline => {
                    mainloop.borrow_mut().stop();
                    return Err(PlatformError::new(
                        "Zeitüberschreitung beim Verbinden mit dem Audio-Server",
                    ));
                }
                _ => thread::sleep(Duration::from_millis(20)),
            }
        }

        let connection = Self {
            mainloop,
            context,
            dirty: Arc::new(AtomicBool::new(true)),
            cache: None,
            fetched_at: None,
        };
        connection.subscribe();

        Ok(connection)
    }

    /// Lässt sich über Sink-, Sink-Input- und Server-Änderungen benachrichtigen,
    /// um den Snapshot-Cache zu invalidieren.
    fn subscribe(&self) {
        let _lock = MainloopLock::acquire(&self.mainloop);

        let dirty = Arc::clone(&self.dirty);
        self.context
            .borrow_mut()
            .set_subscribe_callback(Some(Box::new(move |_facility, _operation, _index| {
                dirty.store(true, Ordering::Relaxed);
            })));

        let mask = InterestMaskSet::SINK | InterestMaskSet::SINK_INPUT | InterestMaskSet::SERVER;
        let _ = self.context.borrow_mut().subscribe(mask, |_success| {});
    }

    fn is_ready(&self) -> bool {
        let _lock = MainloopLock::acquire(&self.mainloop);
        self.context.borrow().get_state() == State::Ready
    }

    fn handle(&mut self, request: Request) {
        match request {
            Request::Snapshot(reply) => {
                let _ = reply.send(self.cached_snapshot());
            }
            Request::SetSinkInputVolume {
                index,
                volume,
                reply,
            } => {
                let _ = reply.send(self.write(|introspect, done| {
                    introspect.set_sink_input_volume(index, &volume, Some(done))
                }));
            }
            Request::SetSinkInputMute { index, mute, reply } => {
                let _ = reply.send(self.write(|introspect, done| {
                    introspect.set_sink_input_mute(index, mute, Some(done))
                }));
            }
            Request::SetSinkVolume {
                name,
                volume,
                reply,
            } => {
                let _ = reply.send(self.write(|introspect, done| {
                    introspect.set_sink_volume_by_name(&name, &volume, Some(done))
                }));
            }
            Request::SetSinkMute { name, mute, reply } => {
                let _ = reply.send(self.write(|introspect, done| {
                    introspect.set_sink_mute_by_name(&name, mute, Some(done))
                }));
            }
            Request::SetDefaultSink { name, reply } => {
                let _ = reply.send(self.set_default_sink(&name));
            }
        }
    }

    fn cached_snapshot(&mut self) -> Result<Snapshot> {
        let stale = self.cache.is_none()
            || self.dirty.swap(false, Ordering::Relaxed)
            || self.fetched_at.is_none_or(|at| at.elapsed() >= CACHE_MAX_AGE);

        if stale {
            let snapshot = self.fetch()?;
            self.fetched_at = Some(Instant::now());
            self.cache = Some(snapshot);
        }

        Ok(self.cache.clone().unwrap_or_default())
    }

    /// Führt eine schreibende Operation aus und markiert den Cache als veraltet.
    ///
    /// `build` bekommt den Introspector und den Fertig-Callback, den es an die
    /// libpulse-Funktion weitergeben muss.
    fn write<F>(&mut self, build: F) -> Result<()>
    where
        F: FnOnce(
            &mut libpulse_binding::context::introspect::Introspector,
            Box<dyn FnMut(bool) + 'static>,
        ) -> Operation<dyn FnMut(bool)>,
    {
        self.dirty.store(true, Ordering::Relaxed);

        let _lock = MainloopLock::acquire(&self.mainloop);
        let mut introspect = self.context.borrow().introspect();

        let operation = build(&mut introspect, self.signal_done());
        self.await_operation(operation)
    }

    fn set_default_sink(&mut self, name: &str) -> Result<()> {
        // Laufende Streams müssen einzeln umgezogen werden – ein neuer
        // Standard-Sink allein verschiebt sie nicht.
        self.dirty.store(true, Ordering::Relaxed);
        let inputs = self.cached_snapshot()?.inputs;

        let result = {
            let _lock = MainloopLock::acquire(&self.mainloop);

            let operation = self.context.borrow_mut().set_default_sink(name, {
                let mainloop = Rc::clone(&self.mainloop);
                move |_success| unsafe {
                    (*mainloop.as_ptr()).signal(false);
                }
            });
            let result = self.await_operation(operation);

            let mut introspect = self.context.borrow().introspect();
            for input in inputs {
                let operation =
                    introspect.move_sink_input_by_name(input.index, name, Some(self.signal_done()));
                // Einzelne Streams dürfen den Umzug verweigern (z. B. exklusiv
                // geroutete Streams) – das darf den Gerätewechsel nicht scheitern lassen.
                let _ = self.await_operation(operation);
            }

            result
        };

        self.dirty.store(true, Ordering::Relaxed);
        result
    }

    /// Fertig-Callback, das den Mainloop weckt, damit [`Self::await_operation`]
    /// weiterlaufen kann.
    fn signal_done(&self) -> Box<dyn FnMut(bool) + 'static> {
        let mainloop = Rc::clone(&self.mainloop);
        Box::new(move |_success: bool| unsafe {
            (*mainloop.as_ptr()).signal(false);
        })
    }

    /// Wartet, bis die Operation abgeschlossen ist. Muss mit gehaltenem
    /// Mainloop-Lock aufgerufen werden.
    fn await_operation<T: ?Sized>(&self, mut operation: Operation<T>) -> Result<()> {
        // Der State-Callback feuert auch bei `Cancelled` – ohne ihn würde ein
        // Abbruch (Server weg) das `wait()` unten dauerhaft blockieren.
        {
            let mainloop = Rc::clone(&self.mainloop);
            operation.set_state_callback(Some(Box::new(move || unsafe {
                (*mainloop.as_ptr()).signal(false);
            })));
        }

        while operation.get_state() == OperationState::Running {
            self.mainloop.borrow_mut().wait();
        }

        match operation.get_state() {
            OperationState::Done => Ok(()),
            _ => Err(PlatformError::new("PulseAudio-Anfrage wurde abgebrochen")),
        }
    }

    fn fetch(&self) -> Result<Snapshot> {
        let _lock = MainloopLock::acquire(&self.mainloop);
        let introspect = self.context.borrow().introspect();

        let default_sink_name = Rc::new(RefCell::new(None::<String>));
        {
            let target = Rc::clone(&default_sink_name);
            let mainloop = Rc::clone(&self.mainloop);
            let operation = introspect.get_server_info(move |info| {
                *target.borrow_mut() = info.default_sink_name.as_ref().map(|n| n.to_string());
                unsafe {
                    (*mainloop.as_ptr()).signal(false);
                }
            });
            self.await_operation(operation)?;
        }

        let sinks = Rc::new(RefCell::new(Vec::new()));
        {
            let target = Rc::clone(&sinks);
            let mainloop = Rc::clone(&self.mainloop);
            let operation = introspect.get_sink_info_list(move |result| match result {
                ListResult::Item(info) => target.borrow_mut().push(SinkSnapshot {
                    name: info.name.as_deref().unwrap_or_default().to_owned(),
                    description: info
                        .description
                        .as_deref()
                        .or(info.name.as_deref())
                        .unwrap_or_default()
                        .to_owned(),
                    volume: info.volume,
                    muted: info.mute,
                }),
                ListResult::End | ListResult::Error => unsafe {
                    (*mainloop.as_ptr()).signal(false);
                },
            });
            self.await_operation(operation)?;
        }

        let inputs = Rc::new(RefCell::new(Vec::new()));
        {
            let target = Rc::clone(&inputs);
            let mainloop = Rc::clone(&self.mainloop);
            let operation = introspect.get_sink_input_info_list(move |result| match result {
                ListResult::Item(info) => {
                    let pid = info
                        .proplist
                        .get_str(properties::APPLICATION_PROCESS_ID)
                        .and_then(|value| value.parse::<u32>().ok());

                    target.borrow_mut().push(SinkInputSnapshot {
                        index: info.index,
                        pid,
                        binary: info
                            .proplist
                            .get_str(properties::APPLICATION_PROCESS_BINARY),
                        app_name: info.proplist.get_str(properties::APPLICATION_NAME),
                        media_name: info
                            .proplist
                            .get_str(properties::MEDIA_NAME)
                            .or_else(|| info.name.as_deref().map(str::to_owned)),
                        volume: info.volume,
                        muted: info.mute,
                        volume_writable: info.volume_writable,
                    });
                }
                ListResult::End | ListResult::Error => unsafe {
                    (*mainloop.as_ptr()).signal(false);
                },
            });
            self.await_operation(operation)?;
        }

        let default_sink_name = default_sink_name.borrow().clone();
        let sinks = sinks.borrow().clone();
        let inputs = inputs.borrow().clone();

        Ok(Snapshot {
            default_sink_name,
            sinks,
            inputs,
        })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        {
            let _lock = MainloopLock::acquire(&self.mainloop);
            self.context.borrow_mut().set_subscribe_callback(None);
            self.context.borrow_mut().disconnect();
        }
        self.mainloop.borrow_mut().stop();
    }
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Umgeht einen Absturz beim Fensteraufbau auf Wayland mit NVIDIA-Treiber.
///
/// GTK3 committet dort eine Surface über `wp_linux_drm_syncobj_surface_v1`,
/// ohne einen Acquire-Point zu setzen. Der Compositor beendet die Verbindung
/// daraufhin mit „Missing acquire timeline“, GDK meldet
/// `Error 71 (Protocol error)` und der Prozess stirbt, bevor ein Fenster
/// erscheint. Explicit Sync abzuschalten umgeht den Fehler; auf allen anderen
/// Treibern wird die Variable ignoriert.
///
/// Wird nur gesetzt, wenn sie noch nicht existiert – so bleibt ein bewusst
/// gesetzter Wert des Nutzers erhalten.
#[cfg(target_os = "linux")]
fn apply_wayland_workarounds() {
    const EXPLICIT_SYNC: &str = "__NV_DISABLE_EXPLICIT_SYNC";

    if std::env::var_os("WAYLAND_DISPLAY").is_some() && std::env::var_os(EXPLICIT_SYNC).is_none() {
        // SAFETY: läuft vor dem Start weiterer Threads und vor der
        // GTK-Initialisierung, es kann also niemand parallel die Umgebung lesen.
        unsafe { std::env::set_var(EXPLICIT_SYNC, "1") };
    }
}

#[cfg(not(target_os = "linux"))]
fn apply_wayland_workarounds() {}

fn main() {
    apply_wayland_workarounds();
    novadeckmiddleware_lib::run()
}

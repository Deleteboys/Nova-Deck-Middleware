use crate::action::actions::Action;
use rspotify::{prelude::*, AuthCodeSpotify, Credentials, OAuth, scopes, Config};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct SpotifyVolumeAction {
    pub step: i8,
}

impl Action for SpotifyVolumeAction {
    fn execute(&self) {
        let step = self.step;

        // Da Spotify übers Internet funkt, lagern wir das in einen async Task aus,
        // damit dein Streamdeck währenddessen nicht einfriert!
        tauri::async_runtime::spawn(async move {
            // Lädt die .env Datei (für Client ID & Secret)
            dotenvy::dotenv().ok();

            let creds = Credentials::from_env().unwrap_or_default();
            let scopes = scopes!("user-modify-playback-state", "user-read-playback-state");
            let oauth = OAuth::from_env(scopes).unwrap_or_default();

            let config = Config {
                token_cached: true,
                ..Default::default()
            };

            let mut spotify = AuthCodeSpotify::with_config(creds, oauth, config);

            // Prüfen, ob wir einen Token im Cache haben
            match spotify.read_token_cache(false).await {
                Ok(Some(token)) => {
                    *spotify.get_token().lock().await.unwrap() = Some(token);

                    // 1. Aktuelle Lautstärke auslesen
                    if let Ok(Some(playback)) = spotify.current_playback(None, None::<Vec<_>>).await {

                        // FIX: playback.device ist direkt verfügbar, wir müssen nur schauen,
                        // ob volume_percent gesetzt ist!
                        if let Some(current_vol) = playback.device.volume_percent {

                            // 2. Neue Lautstärke berechnen und zwischen 0 und 100 limitieren (.clamp)
                            let new_vol = (current_vol as i16 + step as i16).clamp(0, 100) as u8;

                            // 3. Neue Lautstärke setzen
                            if let Err(e) = spotify.volume(new_vol, None).await {
                                println!("Spotify API Fehler: {}", e);
                            } else {
                                println!("Spotify Volume von {}% auf {}% gesetzt", current_vol, new_vol);
                            }
                        } else {
                            println!("Konnte aktuelle Lautstärke nicht auslesen.");
                        }

                    } else {
                        println!("Kein aktives Gerät in Spotify gefunden. (Läuft Musik?)");
                    }
                }
                _ => {
                    println!("Kein Spotify Token gefunden! Bitte führe das Login-Script einmal separat aus.");
                }
            }
        });
    }
}
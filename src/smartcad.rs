//! SmartCAD: marca y pantalla de inicio configurables sin recompilar.
//!
//! Web: la página define `window.SMARTCAD_CONFIG = {...}` antes de cargar el
//! wasm (lo hace `web-app.html`), así que se lee de forma síncrona al arrancar,
//! sin parpadeo. Escritorio: `SMARTCAD_CONFIG` (ruta a un JSON) o
//! `smartcad.json` junto al ejecutable. Sin configuración, todo queda como en
//! Open CAD Studio original.

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StartConfig {
    pub show_title: bool,
    pub show_supporters: bool,
    pub show_discussions: bool,
    pub show_videos: bool,
    pub show_donate: bool,
    pub show_reddit: bool,
    pub show_sponsors: bool,
    pub show_desktop_link: bool,
}

impl Default for StartConfig {
    fn default() -> Self {
        Self {
            show_title: true,
            show_supporters: true,
            show_discussions: true,
            show_videos: true,
            show_donate: true,
            show_reddit: true,
            show_sponsors: true,
            show_desktop_link: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SmartcadConfig {
    /// Nombre que se muestra (título de inicio, «Acerca de»).
    pub app_name: String,
    pub start: StartConfig,
    /// Aviso de donación al arrancar.
    pub donation_prompt: bool,
}

impl Default for SmartcadConfig {
    fn default() -> Self {
        Self {
            app_name: "Open CAD Studio".to_string(),
            start: StartConfig::default(),
            donation_prompt: true,
        }
    }
}

static CONFIG: OnceLock<SmartcadConfig> = OnceLock::new();

/// Configuración vigente (se carga una sola vez, en el primer uso).
pub fn config() -> &'static SmartcadConfig {
    CONFIG.get_or_init(load)
}

fn parse(raw: &str) -> Option<SmartcadConfig> {
    match serde_json::from_str::<SmartcadConfig>(raw) {
        Ok(cfg) => Some(cfg),
        Err(err) => {
            log::warn!("smartcad: configuración inválida, se usan los valores por defecto: {err}");
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load() -> SmartcadConfig {
    let raw = web_sys::window()
        .and_then(|window| {
            js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("SMARTCAD_CONFIG")).ok()
        })
        .filter(|value| !value.is_undefined() && !value.is_null())
        .and_then(|value| js_sys::JSON::stringify(&value).ok())
        .and_then(|value| value.as_string());
    raw.as_deref().and_then(parse).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load() -> SmartcadConfig {
    let path = std::env::var_os("SMARTCAD_CONFIG")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|dir| dir.join("smartcad.json")))
        });
    path.and_then(|p| std::fs::read_to_string(p).ok())
        .as_deref()
        .and_then(parse)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_upstream() {
        let cfg = SmartcadConfig::default();
        assert_eq!(cfg.app_name, "Open CAD Studio");
        assert!(cfg.start.show_supporters && cfg.donation_prompt);
    }

    #[test]
    fn partial_json_keeps_other_defaults() {
        let cfg = parse(r#"{"appName":"SmartCAD","start":{"showSupporters":false}}"#).unwrap();
        assert_eq!(cfg.app_name, "SmartCAD");
        assert!(!cfg.start.show_supporters);
        assert!(cfg.start.show_videos);
        assert!(cfg.donation_prompt);
    }
}

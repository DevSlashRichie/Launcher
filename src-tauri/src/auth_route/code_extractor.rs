use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, Window, WindowBuilder, WindowEvent, WindowUrl};
use crate::auth_route::tokens::CodeToken;

const CODE_REPLACER: &str = "\
            if(location.href.includes('code=')) {
              location.replace('cognatize://' + location.search)
            }
        ";
const WINDOW_LABEL: &str = "authboxcodeextractor";

pub struct CodeExtractor {
    window: Window,
    stop_state: Arc<AtomicBool>,
    code_state: Arc<Mutex<Option<String>>>,
}

impl CodeExtractor {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            stop_state: Arc::new(AtomicBool::new(false)),
            code_state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn open(app_handle: &AppHandle, tag: &str, auth_url: &str) -> Self {
        let window = WindowBuilder::new(app_handle, WINDOW_LABEL, WindowUrl::App("index.html".parse().unwrap()))
            .title(tag)
            .build()
            .unwrap();

        // We need to redirect it because the only way we can use custom schema is by using an App URL
        window.eval(format!("location.replace('{}')", auth_url).as_str()).ok();

        Self::new(window)
    }

    pub async fn fetch(&self) -> Option<CodeToken> {
        let code_state = self.code_state.clone();
        let stop_state = self.stop_state.clone();
        let window = self.window.clone();

        let stop_state_clone = stop_state.clone();
        self.window.on_window_event(move |ev| {
            if let WindowEvent::CloseRequested { .. } = ev {
                stop_state_clone.store(true, Ordering::Relaxed);
            }
        });

        let event_handler = self.window.listen("code", move |ev| {
            if let Some(code) = ev.payload() {
                if CodeToken::uri_includes_code(code) {
                    if let Ok(mut state) = code_state.try_lock() {
                        stop_state.store(true, Ordering::Relaxed);
                        state.replace(code.to_string());
                        window.close().ok();
                    }
                }
            }
        });


        // Since tauri doesn't allow redirections without https scheme
        // we need to manually replace the location of the window
        // for our custom uri scheme listener to work correctly.
        // When the event is fired we stop the loop and fetch the code and state.
        loop {
            if self.stop_state.load(Ordering::Relaxed) {
                self.window.unlisten(event_handler);
                break;
            }

            self.window.eval(CODE_REPLACER).ok();
            thread::sleep(Duration::from_millis(1000));
        }

        self.code_state.lock()
            .map_or(None, |mut state| state.take())
            .map_or(None, |code| {
                CodeToken::from_uri(code.as_str()).ok()
            })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn config(app_handle: &AppHandle, uri: String) {
        if let Some(window) = app_handle.get_window(WINDOW_LABEL) {
            window.trigger("code", Some(uri));
        }
    }
}
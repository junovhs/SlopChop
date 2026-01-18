// src/spinner/mod.rs
//! Triptych HUD (Head-Up Display) for process execution feedback.

pub mod render;
pub mod state;

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use state::HudState;

/// A multi-level Head-Up Display for process execution.
#[derive(Clone)]
pub struct Spinner {
    running: Arc<AtomicBool>,
    /// Shared state for the HUD (title, status, log buffer).
    state: Arc<Mutex<HudState>>,
    /// Handle to the rendering thread.
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl Spinner {
    pub fn start(title: impl Into<String>) -> Self {
        let state = Arc::new(Mutex::new(HudState::new(title)));
        let running = Arc::new(AtomicBool::new(true));

        let r_clone = running.clone();
        let s_clone = state.clone();

        let handle = thread::spawn(move || {
            render::run_hud_loop(&r_clone, &s_clone);
        });

        Self {
            running,
            state,
            handle: Arc::new(Mutex::new(Some(handle))),
        }
    }

    pub fn push_log(&self, line: &str) {
        if let Ok(mut guard) = self.state.lock() {
            guard.push_log(line);
        }
    }

    pub fn step_progress(&self, current: usize, total: usize, message: String) {
        if let Ok(mut guard) = self.state.lock() {
            guard.update_progress(current, total, message);
        }
    }

    pub fn stop(&self, success: bool) {
        if let Ok(mut guard) = self.state.lock() {
            guard.set_finished(success);
        }

        if !self.running.swap(false, Ordering::Relaxed) {
            return;
        }

        if let Ok(mut guard) = self.handle.lock() {
            if let Some(h) = guard.take() {
                let _ = h.join();
            }
        }
    }
}
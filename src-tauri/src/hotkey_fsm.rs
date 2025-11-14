use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    Recording,
}

pub struct HotkeySM {
    state: Arc<Mutex<RecordingState>>,
    last_toggle_time: Arc<Mutex<Option<Instant>>>,
    debounce_ms: u64,
}

impl HotkeySM {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            last_toggle_time: Arc::new(Mutex::new(None)),
            debounce_ms,
        }
    }

    pub fn get_state(&self) -> Result<RecordingState, String> {
        self.state
            .lock()
            .map(|guard| *guard)
            .map_err(|e| e.to_string())
    }

    pub fn try_toggle(&self) -> Result<Option<RecordingState>, String> {
        let mut last_time = self.last_toggle_time.lock().map_err(|e| e.to_string())?;
        let now = Instant::now();

        if let Some(last_instant) = *last_time {
            if now.duration_since(last_instant) < Duration::from_millis(self.debounce_ms) {
                return Ok(None);
            }
        }

        *last_time = Some(now);
        let new_state = {
            let mut state = self.state.lock().map_err(|e| e.to_string())?;
            let new = match *state {
                RecordingState::Idle => RecordingState::Recording,
                RecordingState::Recording => RecordingState::Idle,
            };
            *state = new;
            new
        };

        Ok(Some(new_state))
    }

    pub fn force_set_state(&self, state: RecordingState) -> Result<(), String> {
        let mut state_guard = self.state.lock().map_err(|e| e.to_string())?;
        *state_guard = state;
        Ok(())
    }

    pub fn reset_debounce(&self) -> Result<(), String> {
        let mut last_time = self.last_toggle_time.lock().map_err(|e| e.to_string())?;
        *last_time = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = HotkeySM::new(150);
        assert_eq!(sm.get_state().unwrap(), RecordingState::Idle);
    }

    #[test]
    fn test_toggle_from_idle_to_recording() {
        let sm = HotkeySM::new(150);
        let new_state = sm.try_toggle().unwrap().unwrap();
        assert_eq!(new_state, RecordingState::Recording);
    }

    #[test]
    fn test_toggle_from_recording_to_idle() {
        let sm = HotkeySM::new(150);
        sm.try_toggle().unwrap();
        let new_state = sm.try_toggle().unwrap().unwrap();
        assert_eq!(new_state, RecordingState::Idle);
    }

    #[test]
    fn test_debounce() {
        let sm = HotkeySM::new(100);
        sm.try_toggle().unwrap();
        let result = sm.try_toggle().unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_debounce_timeout() {
        let sm = HotkeySM::new(50);
        sm.try_toggle().unwrap();
        std::thread::sleep(Duration::from_millis(60));
        let result = sm.try_toggle().unwrap().unwrap();
        assert_eq!(result, RecordingState::Idle);
    }

    #[test]
    fn test_force_set_state() {
        let sm = HotkeySM::new(150);
        sm.force_set_state(RecordingState::Recording).unwrap();
        assert_eq!(sm.get_state().unwrap(), RecordingState::Recording);
    }

    #[test]
    fn test_reset_debounce() {
        let sm = HotkeySM::new(10000);
        sm.try_toggle().unwrap();
        sm.reset_debounce().unwrap();
        let result = sm.try_toggle().unwrap();
        assert_ne!(result, None);
    }
}

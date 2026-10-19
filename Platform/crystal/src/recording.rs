//! A session's recording, the C# `ManagedSession.StartRecording` and `StopRecording`: the recorder of part (b) started
//! in a folder under the one given and stopped, each a change between frames; its lines said for the runtime's log
//! (EmuSen_Backend.md §29.36).

use emusen_session_service::abi::status as svc;

use crate::session::{Refused, Session};

impl Session {
    /// A recording begun under `base`, every `stride`th drawn frame kept; the folder it writes into.
    pub fn start_recording(&self, base: &str, stride: u32) -> Result<String, Refused> {
        self.change(|e, _| {
            let d = &mut e.debug;
            if let Some(folder) = d.recorder.folder() {
                return Err((svc::BAD_ARGUMENT, format!("session {} is already recording to {}", self.id, folder.display())));
            }
            let stride = stride.clamp(1, i32::MAX as u32);
            let name = d.core_name.clone();
            let folder = d.recorder.start(std::path::Path::new(base), &name, stride).map_err(|w| (crate::abi::status::IO, w))?;
            Ok(folder.to_string_lossy().into_owned())
        })
    }

    /// The recording ended and its video encoded where an encoder is installed; the folder it wrote into.
    pub fn stop_recording(&self) -> Result<String, Refused> {
        self.change(|e, _| {
            let mut said = Vec::new();
            let Some(folder) = e.debug.recorder.stop(&mut said) else { return Err((svc::ABSENT, format!("session {} is not recording", self.id))) };
            for line in said {
                e.say(line);
            }
            Ok(folder.to_string_lossy().into_owned())
        })
    }
}

/// A recording left running when its session closes, stopped first, `StopRecorder`.
pub fn stop_at_close(e: &mut crate::engine::Engine) {
    let mut said = Vec::new();
    if e.debug.recorder.stop(&mut said).is_some() {
        for line in said {
            e.say(line);
        }
    }
}

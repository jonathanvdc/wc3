//! Completed-submission waiting shared by offscreen tools.
use bevy::render::render_resource::PollType;
use bevy::render::renderer::RenderDevice;
use serde::Serialize;
use std::error::Error;
use std::hint::spin_loop;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionWait {
    #[default]
    SpinPoll,
    // Not selected by capture's default; crowd exposes it as an explicit experiment.
    #[allow(dead_code)]
    BlockingPoll,
}

/// Require queue completion with a finite deadline. BlockingPoll is the backend's
/// blocking poll; this API does not imply a native event-based wait.
pub fn wait(
    device: &RenderDevice,
    policy: CompletionWait,
    timeout: Duration,
) -> Result<(), Box<dyn Error>> {
    match policy {
        CompletionWait::SpinPoll => {
            let deadline = Instant::now() + timeout;
            while !device.poll(PollType::Poll)?.is_queue_empty() {
                if Instant::now() >= deadline {
                    return Err("GPU completion timed out".into());
                }
                spin_loop();
            }
        }
        CompletionWait::BlockingPoll => {
            if !device
                .poll(PollType::Wait {
                    submission_index: None,
                    timeout: Some(timeout),
                })?
                .is_queue_empty()
            {
                return Err("GPU queue remained busy after completion wait".into());
            }
        }
    }
    Ok(())
}

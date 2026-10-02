//! Recorder Job steps.

use blueos_recorder_paths::RecordingRelativePath;

/// One step of a Recorder Job.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RecorderJobStep {
    /// Rewrites one recording file.
    RepairRecording {
        /// Validated relative path.
        path: RecordingRelativePath,
    },
}

//! Cross-platform audio device discovery and management using `cpal`.

use crate::error::SharkordError;
use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};

/// Type of audio peripheral (Input microphone or Output speaker/headphone).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioDeviceKind {
    Input,
    Output,
}

/// Metadata describing an audio device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub kind: AudioDeviceKind,
}

/// Manages system audio device enumeration and default selection.
pub struct AudioDeviceManager {
    host: cpal::Host,
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDeviceManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// Lists all available input (recording/microphone) devices.
    pub fn list_input_devices(&self) -> Result<Vec<AudioDevice>, SharkordError> {
        let default_name = self.host.default_input_device().and_then(|d| d.name().ok());

        let devices = self
            .host
            .input_devices()
            .map_err(|e| SharkordError::Network(format!("Failed to query input devices: {}", e)))?;

        let mut result = Vec::new();
        for dev in devices {
            if let Ok(name) = dev.name() {
                let is_default = default_name.as_deref() == Some(&name);
                result.push(AudioDevice {
                    id: name.clone(),
                    name,
                    is_default,
                    kind: AudioDeviceKind::Input,
                });
            }
        }

        Ok(result)
    }

    /// Lists all available output (playback/speaker) devices.
    pub fn list_output_devices(&self) -> Result<Vec<AudioDevice>, SharkordError> {
        let default_name = self
            .host
            .default_output_device()
            .and_then(|d| d.name().ok());

        let devices = self.host.output_devices().map_err(|e| {
            SharkordError::Network(format!("Failed to query output devices: {}", e))
        })?;

        let mut result = Vec::new();
        for dev in devices {
            if let Ok(name) = dev.name() {
                let is_default = default_name.as_deref() == Some(&name);
                result.push(AudioDevice {
                    id: name.clone(),
                    name,
                    is_default,
                    kind: AudioDeviceKind::Output,
                });
            }
        }

        Ok(result)
    }
}

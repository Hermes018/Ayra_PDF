#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Canonical pointer device type for spatial studio input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum PointerKind {
    Mouse,
    Pen,
    Touch,
}

/// Pointer lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
    Hover,
}

/// High-frequency stylus/mouse/touch sample dispatched over the control bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PointerSample {
    pub phase: PointerPhase,
    pub x: f64,
    pub y: f64,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub twist: f32,
    pub pointer_type: PointerKind,
    pub buttons: u16,
    pub view_id: String,
    pub t_ms: f64,
}

/// Keyboard phase for shortcut routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum KeyPhase {
    Down,
    Up,
}

/// Deterministic control-bus message contract (`ADR-012` §1.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ControlMessage {
    Execute {
        command: String,
        #[serde(default)]
        #[ts(type = "unknown")]
        args: serde_json::Value,
    },
    Pointer(PointerSample),
    #[serde(rename = "ui.key")]
    UiKey {
        chord: String,
        phase: KeyPhase,
    },
    RenderPage {
        doc_id: String,
        page_index: u32,
        dpi: f32,
    },
    #[serde(rename = "tool.select")]
    ToolSelect {
        #[serde(rename = "toolId")]
        tool_id: String,
    },
}

/// Structured error returned by the control bus validator and replay engine.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum BusError {
    #[error("empty command identifier")]
    EmptyCommand,
    #[error("non-finite pointer coordinate or pressure sample")]
    NonFiniteSample,
    #[error("invalid render DPI: must be within [18.0, 2400.0]")]
    InvalidRenderDpi,
    #[error("empty key chord")]
    EmptyChord,
    #[error("execution error: {0}")]
    ExecutionFailed(String),
}

impl ControlMessage {
    /// Validate invariants before dispatching or persisting to a replay journal.
    pub fn validate(&self) -> Result<(), BusError> {
        match self {
            Self::Execute { command, .. } => {
                if command.trim().is_empty() {
                    return Err(BusError::EmptyCommand);
                }
                Ok(())
            }
            Self::Pointer(s) => {
                if !s.x.is_finite()
                    || !s.y.is_finite()
                    || !s.pressure.is_finite()
                    || !s.tilt_x.is_finite()
                    || !s.tilt_y.is_finite()
                    || !s.twist.is_finite()
                    || !s.t_ms.is_finite()
                {
                    return Err(BusError::NonFiniteSample);
                }
                Ok(())
            }
            Self::UiKey { chord, .. } => {
                if chord.trim().is_empty() {
                    return Err(BusError::EmptyChord);
                }
                Ok(())
            }
            Self::RenderPage { dpi, .. } => {
                if !dpi.is_finite() || *dpi < 18.0 || *dpi > 2400.0 {
                    return Err(BusError::InvalidRenderDpi);
                }
                Ok(())
            }
            Self::ToolSelect { tool_id } => {
                if tool_id.trim().is_empty() {
                    return Err(BusError::EmptyCommand);
                }
                Ok(())
            }
        }
    }
}

/// Trait implemented by deterministic state executors (headless test runner or live studio state).
pub trait Executor {
    type State;

    fn apply(&mut self, state: &mut Self::State, msg: &ControlMessage) -> Result<(), BusError>;
}

/// Deterministically replay a sequence of `ControlMessage`s against an `Executor`.
pub fn replay<E: Executor>(
    executor: &mut E,
    initial: E::State,
    journal: &[ControlMessage],
) -> Result<E::State, BusError> {
    let mut state = initial;
    for msg in journal {
        msg.validate()?;
        executor.apply(&mut state, msg)?;
    }
    Ok(state)
}

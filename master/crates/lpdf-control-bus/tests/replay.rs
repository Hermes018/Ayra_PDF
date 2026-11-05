use lpdf_control_bus::{
    replay, BusError, ControlMessage, Executor, KeyPhase, PointerKind, PointerPhase, PointerSample,
};

#[derive(Debug, Clone, PartialEq)]
struct SyntheticStudioState {
    active_tool: String,
    stroke_count: usize,
    last_x: f64,
    last_y: f64,
    executed_commands: usize,
    rendered_pages: usize,
}

impl Default for SyntheticStudioState {
    fn default() -> Self {
        Self {
            active_tool: "pan".to_string(),
            stroke_count: 0,
            last_x: 0.0,
            last_y: 0.0,
            executed_commands: 0,
            rendered_pages: 0,
        }
    }
}

struct SyntheticExecutor;

impl Executor for SyntheticExecutor {
    type State = SyntheticStudioState;

    fn apply(&mut self, state: &mut Self::State, msg: &ControlMessage) -> Result<(), BusError> {
        match msg {
            ControlMessage::ToolSelect { tool_id } => {
                state.active_tool = tool_id.clone();
            }
            ControlMessage::Execute { command, .. } => {
                state.executed_commands += 1;
                if let Some(tool) = command.strip_prefix("tool.") {
                    state.active_tool = tool.to_string();
                }
            }
            ControlMessage::Pointer(sample) => {
                state.last_x = sample.x;
                state.last_y = sample.y;
                if sample.phase == PointerPhase::Up {
                    state.stroke_count += 1;
                }
            }
            ControlMessage::UiKey { chord, phase } => {
                if *phase == KeyPhase::Down && chord == "P" {
                    state.active_tool = "pen.a".to_string();
                }
            }
            ControlMessage::RenderPage { .. } => {
                state.rendered_pages += 1;
            }
        }
        Ok(())
    }
}

#[test]
fn replay_1000_synthetic_messages_is_bit_identical() {
    let mut journal = Vec::with_capacity(1000);
    for i in 0..250 {
        journal.push(ControlMessage::ToolSelect {
            tool_id: if i % 2 == 0 {
                "pen.a".to_string()
            } else {
                "shape.rect".to_string()
            },
        });
        journal.push(ControlMessage::Pointer(PointerSample {
            phase: PointerPhase::Down,
            x: f64::from(i) * 1.5,
            y: f64::from(i) * 2.0,
            pressure: 0.65,
            tilt_x: 5.0,
            tilt_y: -3.0,
            twist: 0.0,
            pointer_type: PointerKind::Pen,
            buttons: 1,
            view_id: "main-canvas".to_string(),
            t_ms: f64::from(i) * 16.67,
        }));
        journal.push(ControlMessage::Pointer(PointerSample {
            phase: PointerPhase::Up,
            x: f64::from(i) * 1.5 + 10.0,
            y: f64::from(i) * 2.0 + 10.0,
            pressure: 0.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            twist: 0.0,
            pointer_type: PointerKind::Pen,
            buttons: 0,
            view_id: "main-canvas".to_string(),
            t_ms: f64::from(i) * 16.67 + 8.0,
        }));
        journal.push(ControlMessage::Execute {
            command: "history.checkpoint".to_string(),
            args: serde_json::json!({ "seq": i }),
        });
    }

    assert_eq!(journal.len(), 1000);

    let mut exec_a = SyntheticExecutor;
    let mut exec_b = SyntheticExecutor;
    let res_a = replay(&mut exec_a, SyntheticStudioState::default(), &journal);
    let res_b = replay(&mut exec_b, SyntheticStudioState::default(), &journal);

    assert!(res_a.is_ok());
    assert_eq!(res_a, res_b);
    let final_state = res_a.unwrap_or_default();
    assert_eq!(final_state.stroke_count, 250);
    assert_eq!(final_state.executed_commands, 250);
}

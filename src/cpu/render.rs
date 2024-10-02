use crate::cpu::cpu_state::CPUState;
use std::string::ToString;

fn render_as_text(state: &CPUState) {
    println!("―――――――――――――――――――――――――――――――――――――――");
    println!("CPU CORES");
    println!("―――――――――――――――――――――――――――――――――――――――");
    println!("- total:   {}", state.total_cores);
    println!("- online:  {}", state.cores_online);
    println!("- offline: {}", state.cores_offline);
    println!("―――――――――――――――――――――――――――――――――――――――");
    for (i, core_state) in state.ordered_core_states.iter().enumerate() {
        let extra = if i == 0 { "(always)" } else { "" };
        println!(
            "- [core {}]: {} ",
            i,
            if *core_state {
                format!("on {}", extra)
            } else {
                "off".to_string()
            }
        );
    }
    println!("―――――――――――――――――――――――――――――――――――――――");
}

fn render_as_json(state: &CPUState) {

    let core_states = state
        .ordered_core_states
        .iter()
        .enumerate()
        .map(|(core_index, core_state)| format!(r#""{}":{}"#, core_index, core_state))
        .collect::<Vec<String>>()
        .join(",");

    let core_states = format!("{{{}}}", core_states);
    let output = format!(
        "{{\"total\":{},\"online\":{},\"offline\":{},\"cores_online\":{}}}",
        state.total_cores, state.cores_online, state.cores_offline, core_states
    );

    println!("{}", output);
}

/// Renders the CPU state in the desired format.
/// # Arguments
/// * `state` - The state of the CPU(s) on the system.
/// * `as_json` - Whether to render the state in JSON format or in human-readable text format.
///
pub fn render(state: &CPUState, as_json: bool) {
    if as_json {
        render_as_json(state);
    } else {
        render_as_text(state);
    }
}

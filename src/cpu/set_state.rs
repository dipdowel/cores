use crate::cpu;
use crate::cpu::cpu_state::CPUState;
use std;
use std::collections::HashMap;
use std::fs;

/// Sets the state of a specified core.
pub fn set_core_state(core: usize, is_online: bool) -> Result<bool, Box<dyn std::error::Error>> {
    // Don't ever change state of the core 0.
    if core == 0 {
        return Ok(false);
    }

    let dest_state = if is_online { "1" } else { "0" };
    let cpu_state_path = format!("/sys/devices/system/cpu/cpu{}/online", core);

    let old_cpu_state: CPUState = cpu::get_state()?;
    if core > old_cpu_state.total_cores-1 {
        eprintln!("Core {} does not exist", core);
        return Ok(false);
    }

    if old_cpu_state.ordered_core_states[core] == is_online {
        return Ok(false);
    }

    let mut core_state_updated: bool = false;

    // Attempt to enable the core by writing "1" to the corresponding CPU file
    match fs::write(cpu_state_path, dest_state) {
        Ok(_) => {
            core_state_updated = true;
        }
        Err(e) => {
            eprintln!("Could not set core {} to state {}. {}", core, dest_state, e);
        }
    }
    Ok(core_state_updated)
}

/// Sets the state of the cores as specified in the `core_states` HashMap.
/// # Arguments
/// * `core_states` - A HashMap with the core index as the key and the desired state as the value.
pub fn set_cores(core_states: &HashMap<usize, bool>) -> Result<usize, Box<dyn std::error::Error>> {
    let old_cpu_state: CPUState = cpu::get_state()?;

    let mut core_states_updated: usize = 0;

    core_states.iter().for_each(|(core, state)| {
        // Don't ever change state of the core 0.
        if *core == 0 {
            return;
        }

        // The given core is already in the desired state
        if old_cpu_state.ordered_core_states[*core] == *state {
            return;
        }
        match set_core_state(*core, *state) {
            Ok(result) => {
                if result{
                    core_states_updated += 1;
                }
            }
            Err(e) => {
                eprintln!("Could not set core {} to state {}. {}", core, state, e);
            }
        }
    });

    Ok(core_states_updated)
}

/// Resets all cores to online state.
///
/// # Returns
/// The number of cores that were brought online.
pub fn reset_cores() -> Result<usize, Box<dyn std::error::Error>> {
    let old_cpu_state: CPUState = cpu::get_state()?;

    let mut core_states_updated: usize = 0;

    old_cpu_state
        .ordered_core_states
        .iter()
        .enumerate()
        .for_each(|(core, state)| {
            // Don't ever change state of the core 0.
            if core == 0 {
                return;
            }

            if !*state {
                match set_core_state(core, true) {
                    Ok( result ) => {
                        if result{
                            core_states_updated += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Could not bring core {} online. {}.", core, e);
                    }
                }
            }
        });

    Ok(core_states_updated)
}

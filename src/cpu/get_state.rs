use crate::core_list;
use crate::cpu::cpu_state::CPUState;
use std::error::Error;
use std::fs;

const LINUX_CPU_ONLINE: &str = "/sys/devices/system/cpu/online";
const LINUX_CPU_PRESENT: &str = "/sys/devices/system/cpu/present";

const MODERN_LINUX_MSG: &str = "Are you on a modern Linux system with kernel 2.6.0 or later?";
fn get_present_cores() -> Result<Vec<usize>, Box<dyn Error>> {
    match fs::read(LINUX_CPU_PRESENT) {
        Ok(content_raw) => {
            let content = String::from_utf8(content_raw)?.trim().to_string();
            let cores_present = core_list::parse(&content);
            Ok(cores_present.into_iter().collect())
        }
        Err(e) => {
            Err(Box::from(format!("Could not read {LINUX_CPU_PRESENT}. {e}. {MODERN_LINUX_MSG}")))
        }
    }
}

fn get_online_cores() -> Result<Vec<usize>, Box<dyn Error>> {
    match fs::read(LINUX_CPU_ONLINE) {
        Ok(content_raw) => {
            let content = String::from_utf8(content_raw)?.trim().to_string();
            let cores_online = core_list::parse(&content);
            Ok(cores_online.into_iter().collect())
        }
        Err(e) => {
            Err(Box::from(format!("Could not read {LINUX_CPU_ONLINE}. {e}. {MODERN_LINUX_MSG}")))
        }
    }
}

pub fn get_state() -> Result<CPUState, Box<dyn Error>> {
    let cores_present = get_present_cores()?;
    let online_cores = get_online_cores()?;

    let total_cores = cores_present.len();
    let cores_online = online_cores.len();

    // A vector of booleans representing the state of each core.
    let mut ordered_core_states = vec![false; total_cores];

    for core in online_cores {
        ordered_core_states[core] = true;
    }

    Ok(CPUState {
        total_cores,
        cores_online,
        cores_offline: total_cores - cores_online,
        ordered_core_states,
    })

}

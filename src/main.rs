mod core_list;
mod cpu;
mod sys_linux;

use std::collections::HashMap;
use std::env;

use clap::Parser;

use crate::cpu::cpu_state::CPUState;
use crate::sys_linux::{drop_privileges, restore_privileges, DropPrivilegeResult};

/// CLI argument parser using `clap`.
#[derive(Parser, Debug)]
#[command(
    name = "cores",
    override_usage = " cores <CORE_NUMBER> <on|off> [--json]\n\
                      \tcores --set <NUMBER> | --reset | --custom <RANGE> [--json]",
    about = "\
    cores ― a convenience tool for enabling and disabling CPU cores on Linux systems.\n\
    - Run without parameters to see the current state of the CPU cores.\n\
    - Root privileges are needed to modify the state.\n\
    - Root privileges are used \x1b[4monly\x1b[0m for writing to `/sys/devices/system/cpu/cpuN/online`\n\
    - NB: On most systems core 0 is always online as it is essential for handling critical system interrupts,
  low-level kernel tasks, and managing system stability, so attempts to disable core 0 are ignored.\n\
    ",
    term_width = 80,
    after_help = "\
\x1b[4mExamples\x1b[0m:
   cores 2 on            Set core 2 online, other cores remain unchanged.
   cores 2 off           Set core 2 offline, other cores remain unchanged.
   cores -s 1            Set cores 0 and 1 online, set all the other cores offline.
   cores -s 3            Set cores 0, 1, 2, 3 online, set all the other cores offline.
   cores -c 1-3,5        Set cores 0, 1, 2, 3, 5 online, set all the other cores offline.
   cores -c 0-2,4-5      Set cores 0, 1, 2, 4, 5 online, set all the other cores offline.
   cores -c \"0-2, 4-5\"   Set cores 0, 1, 2, 4, 5 online, set all the other cores offline.

\x1b[4mFeedback\x1b[0m:
   - Bug reports: https://github.com/dipdowel/cores/issues
   - Suggestions: https://github.com/dipdowel/cores/discussions"
)]
struct Args {
    /// Specifies a core to set online or offline.
    #[arg(required = false, conflicts_with_all = &["set", "reset", "custom"])]
    core: Option<usize>,

    ///  on | off - the state of the core to set.
    #[arg(required = false, conflicts_with_all = &["set", "reset", "custom"])]
    state: Option<String>,

    /// Set NUMBER of cores online and set all the other cores offline. Minimum value is 1 (core 0 is always online).
    #[arg(short, long, conflicts_with_all = &["reset", "custom", "core", "state"], value_name = "NUMBER")]
    set: Option<usize>,

    /// Enable all the cores of the system.
    #[arg(short, long, conflicts_with_all = &["set", "custom", "core", "state"])]
    reset: bool,

    /// Use CPU list format (as in /sys/devices/system/cpu/online) to specify cores. E.g.: 0 | 0-5 | 1-3,5 | 0-2,4-5
    #[arg(short, long, exclusive = true, conflicts_with_all = &["set", "reset", "core", "state"], value_name = "CPU_LIST")]
    custom: Option<String>,

    /// Print state of the cores in JSON format.
    #[arg(short, long)]
    json: bool,
}

fn print_cores_updated(cores_updated: usize, is_json: bool) {
    if is_json {
        return;
    }
    println!("Core(s) updated: {}", cores_updated);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    // Don't use root privileges for tasks that don't require it.
    // E.g. `clap` does not require root privileges to parse CLI arguments.
    let drop_result = drop_privileges();

    // Parse the CLI arguments (at this point as non-root)
    let args = Args::parse();

    //----------------------------------------------------------------------------------------------
    // Render the CPU state in JSON or human-friendly text format and exit (if no args or just `-j`)
    //----------------------------------------------------------------------------------------------
    let no_args = env::args().len() == 1;
    let one_arg = env::args().len() == 2;
    let just_render = no_args || args.json && one_arg;
    if just_render {
        let cpu_state: CPUState = cpu::get_state()?;
        cpu::render(&cpu_state, args.json);
        if no_args {
            println!("Run `cores --help` for more information");
        }
        return Ok(());
    }

    //----------------------------------------------------------------------------------------------
    // Check for root access
    //----------------------------------------------------------------------------------------------
    // If `cores` was not run with root privileges, we can't modify the state of the CPU cores,
    // hence exit with an error message.
    if drop_result == DropPrivilegeResult::NotRoot {
        eprintln!("Root privileges are needed to modify the state of the CPU cores.\nTry `sudo cores <arguments>`");
        std::process::exit(1);
    }

    //----------------------------------------------------------------------------------------------
    // Set all the cores to online, render the state, and exit
    //----------------------------------------------------------------------------------------------
    if args.reset {
        // println!("Resetting all cores to online...");
        restore_privileges(); // Restore root privileges
        let cores_updated = cpu::reset_cores()?;
        drop_privileges(); // Drop root privileges again
        cpu::render(&cpu::get_state()?, args.json); // Render the latest CPU state
        print_cores_updated(cores_updated, args.json);
        return Ok(());
    }

    //----------------------------------------------------------------------------------------------
    // Set as many cores as specified in the `set` argument to online, render the state, and exit
    //----------------------------------------------------------------------------------------------
    if let Some(mut cores_to_set_online) = args.set {
        let cpu_state: CPUState = cpu::get_state()?;

        if cores_to_set_online > cpu_state.total_cores {
            cores_to_set_online = cpu_state.total_cores;
        }

        // Limit the number of cores to the total number available (if exceeded)
        if cores_to_set_online > cpu_state.total_cores {
            cores_to_set_online = cpu_state.total_cores;
        }

        // Create a HashMap with the needed state for each core
        let mut core_states: HashMap<usize, bool> = HashMap::with_capacity(cpu_state.total_cores);
        for i in 0..cpu_state.total_cores {
            if i < cores_to_set_online {
                core_states.insert(i, true);
            } else {
                core_states.insert(i, false);
            }
        }

        restore_privileges(); // get root access to set the cores
        let cores_updated = cpu::set_cores(&core_states)?;
        drop_privileges(); // drop root access
        cpu::render(&cpu::get_state()?, args.json); // Render the latest CPU state
        print_cores_updated(cores_updated, args.json);
        return Ok(());
    }

    //----------------------------------------------------------------------------------------------
    // Parse a custom core range in CPU list format, apply the settings, render the state, and exit
    //----------------------------------------------------------------------------------------------

    if let Some(custom_cpu_range) = args.custom {
        let new_core_config = core_list::parse(&custom_cpu_range);

        let cpu_state: CPUState = cpu::get_state()?;

        // Create a HashMap with the needed state for each core
        let mut new_core_states: HashMap<usize, bool> =
            HashMap::with_capacity(cpu_state.total_cores);

        for core in 0..cpu_state.total_cores {
            if new_core_config.contains(&core) {
                new_core_states.insert(core, true);
            } else {
                new_core_states.insert(core, false);
            }
        }

        restore_privileges(); // Set root access
        let cores_updated = cpu::set_cores(&new_core_states)?; // Set the cores
        drop_privileges(); // Drop root access

        cpu::render(&cpu::get_state()?, args.json); // Render the latest CPU state
        print_cores_updated(cores_updated, args.json);
        return Ok(());
    }

    //----------------------------------------------------------------------------------------------
    // Set online/offline an individual core, render the state, and exit
    //----------------------------------------------------------------------------------------------
    match (args.core, args.state) {
        (Some(core), Some(state)) => {
            let cpu_state: CPUState = cpu::get_state()?;
            if core < 1 || core > cpu_state.total_cores - 1 {
                eprintln!(
                    "<CORE_NUMBER> must be greater than 0 and less than {}",
                    cpu_state.total_cores
                );
                std::process::exit(1);
            }

            if state != "on" && state != "off" {
                eprintln!("<STATE> must be either 'on' or 'off'");
                std::process::exit(1);
            }

            restore_privileges(); // get root access to set the core
            let core_updated = cpu::set_core_state(core, state == "on")?;
            drop_privileges(); // drop root access
            cpu::render(&cpu::get_state()?, args.json); // Render the latest CPU state
            print_cores_updated(core_updated as usize, args.json);
        }
        (_, _) => {
            eprintln!("Bad syntax. Try `cores --help` for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

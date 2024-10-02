#[derive(Debug)]
/// Represents the state of the CPU(s) on the system.
pub struct CPUState {
    /// The total number of cores on the system.
    pub total_cores: usize,
    /// The number of cores that are enabled.
    pub cores_online: usize,
    /// The number of cores that are disabled.
    pub cores_offline: usize,
    /// A vector of booleans representing the state of each core. `true` means the core is enabled, `false` means the core is disabled.
    pub ordered_core_states: Vec<bool>,
}
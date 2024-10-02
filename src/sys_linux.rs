use std::env;
use std::string::ToString;

const NOT_SUDOER: &str = "not_sudoer";

/* FFI declaration of system calls (C / POSIX) for dropping and restoring root privileges
*****************************************************************************************/
extern "C" {
    /// set effective user ID (try `man seteuid` in your terminal)
    fn seteuid(uid: u32) -> i32;
    /// get real user ID (`man getuid`)
    fn getuid() -> u32;
    // /// get effective user ID (`man geteuid`)
    // fn geteuid() -> u32;
}

#[derive(Debug)]
pub enum DropPrivilegeResult {
    /// The privileges were dropped successfully
    Dropped,
    /// The user is not root, hence can't drop privileges
    NotRoot,
    /// `SUDO_UID` not set, hence the user is not not a sudoer but a pure root user
    PureRoot,
}

impl PartialEq for DropPrivilegeResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DropPrivilegeResult::Dropped, DropPrivilegeResult::Dropped) => true,
            (DropPrivilegeResult::NotRoot, DropPrivilegeResult::NotRoot) => true,
            (DropPrivilegeResult::PureRoot, DropPrivilegeResult::PureRoot) => true,
            _ => false,
        }
    }
}

/// Drop root privileges temporarily.
/// Should be called as soon as root tasks are finished and root privileges are no longer needed.
pub fn drop_privileges() -> DropPrivilegeResult {
    /*
    ```shell
        $ sudo env |grep SUDO
        SUDO_COMMAND=/usr/bin/env
        SUDO_USER=username
        SUDO_UID=1111
        SUDO_GID=1111
    ```
    */

    // If the user is not `root`, no need to drop privileges
    let uid = unsafe { getuid() };
    if uid != 0 {
        return DropPrivilegeResult::NotRoot;
    }

    // Get the original user ID from the `SUDO_UID` env variable in case of `sudo`
    let sudoer_uid_string = env::var("SUDO_UID").unwrap_or(NOT_SUDOER.to_string());

    // The user is root but not a sudoer, hence not possible to drop privileges
    if sudoer_uid_string == NOT_SUDOER || sudoer_uid_string == "0" {
        return DropPrivilegeResult::PureRoot;
    }

    let sudoer_uid: u32 = sudoer_uid_string.parse().unwrap_or_else(|_| {
        eprintln!("SUDO_UID env var is likely to be malformed");
        std::process::exit(1);
    });

    // Drop privileges by setting the effective UID to the UID of a sudoer
    if unsafe { seteuid(sudoer_uid) } != 0 {
        eprintln!("Failed to drop root privileges");
        std::process::exit(1);
    }

    DropPrivilegeResult::Dropped
}

/// Restore root privileges
pub fn restore_privileges() {
    let root_uid = 0; // root UID is `0`
    if unsafe { seteuid(root_uid) } != 0 {
        eprintln!("Failed to restore root privileges");
        std::process::exit(1);
    }
}

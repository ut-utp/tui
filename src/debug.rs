//! Just indicates whether debug mode is enabled.

pub const DEBUG_ENV_VAR: &'static str = "TUI_DEBUG";

pub fn in_debug_mode() -> bool {
    std::env::var_os(DEBUG_ENV_VAR).is_some()
}

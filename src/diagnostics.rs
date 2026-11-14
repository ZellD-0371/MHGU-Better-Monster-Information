use core::sync::atomic::{AtomicBool,Ordering};
static ENABLED:AtomicBool=AtomicBool::new(false);
pub fn enabled()->bool{ENABLED.load(Ordering::Relaxed)}
pub fn set_enabled(value:bool){ENABLED.store(value,Ordering::Relaxed);}
// Check before formatting: disabled diagnostics neither allocate nor call Forge.
#[macro_export]
macro_rules! diagnostic_info{($($arg:tt)*)=>{if $crate::diagnostics::enabled(){forge::log::prelude::info!($($arg)*);}}}

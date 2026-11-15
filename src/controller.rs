//! Consume the engine menu word after Recomp's optional action remapping.
use core::sync::atomic::{AtomicU32,Ordering};
use crate::STATE;
pub static EVENTS:AtomicU32=AtomicU32::new(0);
pub static HELD:AtomicU32=AtomicU32::new(0);
static mut GATE:crate::menu_input::Gate=crate::menu_input::Gate::NEW;
#[unsafe(no_mangle)]
pub static monster_info_input_audit:[AtomicU32;8]=[const{AtomicU32::new(0)};8];

// NSO +0x3de4f8 = runtime f_5de4f8. The original translator also folds
// the left stick into menu directions, on both the original game and Recomp.
// Recomp 0.17.2 runs its outer remapping hook before this mod's hook.
#[forge::hook(offset=0x3de4f8)]
fn menu_translate(object:u32,word:u32)->u32{
 let mapped=original!(object,word);
 let menu=forge::graphics::Graphics::is_menu_visible();
 let(active,mode,interactive,rank,forms)=STATE.with(|s|(s.visible&&s.ready&&s.input_ready,s.mode,s.interactive(),s.tab==3&&s.ranks.len()>1,s.has_forms()));
 let output=unsafe{(&mut *core::ptr::addr_of_mut!(GATE)).step(mapped,active,mode,interactive,rank,forms,menu)};
 if output.events!=0{EVENTS.fetch_or(output.events,Ordering::AcqRel);}
 if menu||!active{EVENTS.store(0,Ordering::Release);}
 HELD.store(output.held,Ordering::Release);
 for(i,v)in [mapped,output.filtered,word,mapped&!output.filtered,active as u32,mode as u32,menu as u32].iter().enumerate(){monster_info_input_audit[i].store(*v,Ordering::Relaxed);}
 monster_info_input_audit[7].fetch_add(1,Ordering::Release);
 output.filtered
}
pub fn install(base:u32)->bool{
 // Validate the original MHGU 1.4.0 code before patching. No host APIs or
 // Recomp config files are required by the hook on Switch/emulators.
 if unsafe{crate::native::rd(base+0x3de4f8)}!=0xe3a02010||unsafe{crate::native::rd(base+0x3de4fc)}!=0xe3a00040{return false}
 forge::install_hook!(base,menu_translate);true
}

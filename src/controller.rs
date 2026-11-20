//! Consume the engine menu word after Recomp's optional action remapping.
use core::sync::atomic::{AtomicU32,Ordering};
use crate::STATE;
pub static EVENTS:AtomicU32=AtomicU32::new(0);
pub static HELD:AtomicU32=AtomicU32::new(0);
static MENU_HELD:AtomicU32=AtomicU32::new(0);
/// Frames left to withhold MenuConfirm after a left click we used (Recomp's menu mouse also turns it into a confirm)
pub static SWALLOW:AtomicU32=AtomicU32::new(0);
/// Frames left to give the game a cancel (our back bar clicked while the game owns cancel)
pub static INJECT:AtomicU32=AtomicU32::new(0);
/// Frames left to give Recomp's confirm to the game only: a left click outside our page (the native monster list)
static PASS:AtomicU32=AtomicU32::new(0);
static LEFT:AtomicU32=AtomicU32::new(0);
static mut GATE:crate::menu_input::Gate=crate::menu_input::Gate::NEW;
#[unsafe(no_mangle)]
pub static monster_info_input_audit:[AtomicU32;8]=[const{AtomicU32::new(0)};8];

// NSO +0x3de4f8 = runtime f_5de4f8. The original translator also folds
// the left stick into menu directions, on both the original game and Recomp.
// Recomp 0.17.2 runs its outer remapping hook before this mod's hook.
#[forge::hook(offset=0x3de4f8)]
fn menu_translate(object:u32,word:u32)->u32{
 let mapped=original!(object,word);
 let mapped=if SWALLOW.load(Ordering::Acquire)>0{mapped&!crate::menu_input::CONFIRM}else{mapped};
 // A new menu press: the controller, unless a key came just before it (button_prompts::Device).
 if mapped&!MENU_HELD.swap(mapped,Ordering::AcqRel)!=0{crate::button_prompts::note_menu_press();}
 let menu=forge::graphics::Graphics::is_menu_visible();
 // Recomp mouse: a left press off our targets belongs to the game. Its confirm skips our gate and reaches the game,
 // while our page is not modal (mode 0); with a row or list open the game stays blocked as before.
 let mut pass=PASS.load(Ordering::Acquire);
 if let Some((x,y,buttons))=crate::button_prompts::mouse_now(){
  let left=buttons&1;let new=left!=0&&LEFT.swap(left,Ordering::AcqRel)==0;if left==0{LEFT.store(0,Ordering::Release);}
  if new{
   let off=STATE.with(|s|s.visible&&s.mode==0&&s.view.is_some_and(|v|{let(px,py)=crate::mouse::to_display(x,y,v);crate::mouse::hit(&s.targets,px,py).is_none()}));
   pass=if off{10}else{0};
  }else if pass>0{pass=if left!=0{10}else{pass-1};}
 }else if pass>0{pass-=1;}
 PASS.store(pass,Ordering::Release);
 let given=if pass>0{mapped&crate::menu_input::CONFIRM}else{0};let mapped=mapped&!given;
 let(active,mode,interactive,rank,forms)=STATE.with(|s|(s.visible&&s.ready&&s.input_ready,s.mode,s.interactive(),s.tab==3&&s.ranks.len()>1,s.has_forms()));
 let output=unsafe{(&mut *core::ptr::addr_of_mut!(GATE)).step(mapped,active,mode,interactive,rank,forms,menu)};
 if output.events!=0{EVENTS.fetch_or(output.events,Ordering::AcqRel);}
 if menu||!active{EVENTS.store(0,Ordering::Release);}
 HELD.store(output.held,Ordering::Release);
 for(i,v)in [mapped,output.filtered,word,mapped&!output.filtered,active as u32,mode as u32,menu as u32].iter().enumerate(){monster_info_input_audit[i].store(*v,Ordering::Relaxed);}
 monster_info_input_audit[7].fetch_add(1,Ordering::Release);
 let inject=INJECT.load(Ordering::Acquire);
 if inject>0{INJECT.store(inject-1,Ordering::Release);return output.filtered|given|crate::menu_input::CANCEL}
 output.filtered|given
}
pub fn install(base:u32)->bool{
 // Preserve Forge's hook chain when another mod installed its translator
 // first. Validate the unchanged tail instead of requiring a pristine entry.
 let site=base+0x3de4f8;
 let words=core::array::from_fn(|i|unsafe{crate::native::rd(site+i as u32*4)});
 if !crate::hook_guard::menu_entry(words,site){return false}
 forge::install_hook!(base,menu_translate);true
}

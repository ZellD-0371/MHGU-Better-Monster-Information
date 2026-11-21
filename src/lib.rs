#![no_std]
extern crate alloc;
mod numeric;mod retry;mod diagram_lifetime;mod settings_labels;mod part_info;mod diagnostics;mod language;mod language_pack;mod native;mod data;
#[cfg(not(bmi_low_memory))]mod generated;
#[cfg(bmi_low_memory)]#[path="generated_low.rs"]mod generated;mod generated_kbm;mod model;mod resources;mod controller;mod mouse;mod hook_guard;mod purplelib;mod button_prompts;mod menu_input;mod ui;mod storage;mod layout;mod history;mod history_format;mod perf;
use core::{cell::UnsafeCell,ffi::c_void,sync::atomic::{AtomicBool,AtomicU32,Ordering}};
use forge::log::prelude::error;
static BASE:AtomicU32=AtomicU32::new(0);
static mut ARM:mouse::Arm=mouse::Arm::NEW;
/// The scrollbar is held by the left button
static mut DRAG:bool=false;
struct Shared{lock:AtomicBool,data:UnsafeCell<Option<model::State>>}
unsafe impl Sync for Shared{}
impl Shared{fn with<R>(&self,f:impl FnOnce(&mut model::State)->R)->R{
 while self.lock.compare_exchange_weak(false,true,Ordering::Acquire,Ordering::Relaxed).is_err(){core::hint::spin_loop()}
 let s=unsafe{&mut*self.data.get()};let r=f(s.get_or_insert_with(model::State::new));self.lock.store(false,Ordering::Release);r
}}
static STATE:Shared=Shared{lock:AtomicBool::new(false),data:UnsafeCell::new(None)};

#[forge::entry]
fn main(){
    let base=forge::mem::text_addr();BASE.store(base,Ordering::Release);
    if !native::install(base){error!("[monster-info] MHGU 1.4.0 signatures rejected; disabled");return}
    if model::REQUIRE_RANK_HUNT&&!history::install(base){error!("[monster-info] rank-history signature rejected; disabled");return}
    let input=controller::install(base);STATE.with(|s|{s.installed=true;s.input_ready=input;});
    crate::diagnostic_info!("[monster-info] v{} native notes adapter; input capture={}",env!("CARGO_PKG_VERSION"),input);
}

#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(_dt:f32){
 let sample=perf::start();
 let base=BASE.load(Ordering::Acquire);if base==0||!STATE.with(|s|s.installed&&s.ready){return}
 storage::poll();
 if model::REQUIRE_RANK_HUNT{history::update(base);}
 // The game's Forge delta uses game ticks. ImGui's delta is wall-clock seconds.
 let dt=forge::imgui::get_io().DeltaTime.clamp(0.001,0.1);
 let snap=native::snapshot(base);let refresh=STATE.with(|s|{s.progress_age+=dt;!s.visible||s.progress_age>=0.5||perf::legacy()});let progress=if refresh&&snap.as_ref().is_some_and(|s|s.extra){let mut p=native::progress(base);if model::REQUIRE_RANK_HUNT{native::hunt_progress(base,&mut p);}Some(p)}else{None};
 let menu=forge::graphics::Graphics::is_menu_visible();
 // Recomp only: keyboard / mouse prompts and the wheel, which scrolls like the right stick.
 let input=button_prompts::poll_input(dt);let active=STATE.with(|s|s.visible&&s.input_ready);
 let mut events=controller::EVENTS.swap(0,Ordering::AcqRel);
 let mut clicked=None;let mut cancel=false;let held=controller::HELD.load(Ordering::Acquire);
 let cue=STATE.with(|s|{
  let was=s.visible;s.visible=snap.as_ref().is_some_and(|n|n.extra);
  // Recomp mouse: a new left press on one of our targets (drawn last frame) is ours, not a menu confirm
  // A right click while we hold the pointer (modal page) is our back: Recomp's menu mouse no longer makes it a cancel
  let right=unsafe{(*core::ptr::addr_of!(ARM)).armed()}&&input.pressed&4!=0&&button_prompts::mouse_free()&&s.visible&&!menu;
  if right{events|=model::BACK;}
  let click=unsafe{(&mut*core::ptr::addr_of_mut!(ARM)).step(s.visible&&s.input_ready&&!menu,input.buttons,input.pressed)};
  // Target under the pointer (targets drawn last frame)
  let under=match(input.mouse,s.view){(Some((x,y)),Some(view))=>{let(px,py)=mouse::to_display(x,y,view);mouse::hit(&s.targets,px,py)}_=>None};
  if click{clicked=under;}
  if active&&!menu{events|=mouse::wheel_events(input.wheel,s.mode,under);}
  if let Some(n)=snap{s.select(n.id,n.note,n.pages);}
  if let Some(p)=progress{s.progress=p;s.progress_age=0.;if !was{s.mode=0;s.row=0;s.scroll=0;s.clock=0.;s.reset_form()}}
  if !s.visible{s.hover=None;s.mode=0;s.row=0;s.scroll=0;s.detail_scroll=0;s.clock=0.;return None}
  let old=(s.mode,s.tab,s.row,s.rank_cursor,s.detail_scroll,s.form_cursor,s.scroll);
  // Scrollbar: a press starts dragging, the held button keeps the thumb under the pointer
  unsafe{
   if clicked==Some(model::Target::Scrollbar)&&!matches!(s.mode,2|3){DRAG=true;}
   if input.buttons&1==0||menu{DRAG=false;}
   if DRAG{if let(Some((x,y)),Some(view),Some(([_,top,_,bottom],_)))=(input.mouse,s.view,s.targets.iter().find(|(_,t)|*t==model::Target::Scrollbar)){
    let(_,py)=mouse::to_display(x,y,view);s.scroll_to(mouse::scroll_offset(py,*top,*bottom,s.rows(),layout::PAGE_ROWS));
   }}
  }
  if let Some(t)=clicked{events&=!model::ACCEPT;cancel=s.click(t);crate::diagnostic_info!("[monster-info] mouse click {:?} -> mode {} tab {} row {}",t,s.mode,s.tab,s.row);}
  if !menu{s.action(events);s.tick(dt,held);}
  // Gold frame and cursor sound when the pointer reaches another target (keyboard / mouse players only)
  let hover=if s.visible&&!menu&&button_prompts::kbm_active(){under.filter(|t|mouse::hoverable(*t))}else{None};
  let moved=hover!=s.hover&&hover.is_some();s.hover=hover;
  if s.tab!=old.1{Some(27)}else if s.mode!=old.0{Some(if events&model::BACK!=0||clicked==Some(model::Target::Back){2}else{1})}
  else if (s.row,s.rank_cursor,s.detail_scroll,s.form_cursor,s.scroll)!=(old.2,old.3,old.4,old.5,old.6)||moved{Some(4)}else{None}
 });
 // Keep withholding Recomp's confirm while the used button is held and a few frames after (it may come late)
 let swallow=controller::SWALLOW.load(Ordering::Acquire);
 controller::SWALLOW.store(if clicked.is_some()||swallow>0&&input.buttons&1!=0{6}else{swallow.saturating_sub(1)},Ordering::Release);
 if cancel{controller::INJECT.store(2,Ordering::Release);}
 button_prompts::set_mouse_free(!menu&&STATE.with(|s|s.visible&&s.mode!=0));
 resources::tick_retry(dt,STATE.with(|s|s.visible));
 // A click on a button or row always answers with the decide sound (a row change alone would only give the cursor
 // sound, already heard when the pointer reached it). Tab changes and back keep their own sounds.
 let cue=if clicked.is_some_and(mouse::hoverable)&&matches!(cue,None|Some(4)){Some(1)}else{cue};
 if let Some(id)=cue{native::play_ui(base,id);}
 let focus=STATE.with(|s|s.visible&&s.mode==1&&s.tab==3&&!menu);native::material_focus(base,focus);
 let item=STATE.with(|s|if s.visible&&s.tab==3&&s.mode==1{s.material().map(|m|generated::ITEMS[m.item as usize].native)}else{None});
 let refresh=STATE.with(|s|{s.stock_age+=dt;let refresh=s.stock_item!=item||s.stock_age>=0.25||perf::legacy();s.stock_item=item;if refresh{s.stock_age=0.;}refresh});if refresh{let owned=item.and_then(|id|native::stock(base,id));STATE.with(|s|s.owned=owned);}
 perf::update_done(sample);
}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiInit(_ctx:*mut c_void,_alloc:*mut c_void,_free:*mut c_void){ui::init();}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiFreeRender(){let sample=perf::start();ui::render();perf::render_done(sample);}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiRender(){ui::menu();}

#![no_std]
extern crate alloc;
mod numeric;mod retry;mod diagram_lifetime;mod settings_labels;mod diagnostics;mod language;mod language_pack;mod native;mod data;
#[cfg(not(bmi_low_memory))]mod generated;
#[cfg(bmi_low_memory)]#[path="generated_low.rs"]mod generated;mod model;mod resources;mod controller;mod ui;mod storage;mod layout;mod history;mod history_format;mod perf;
use core::{cell::UnsafeCell,ffi::c_void,sync::atomic::{AtomicBool,AtomicU32,Ordering}};
use forge::log::prelude::error;
static BASE:AtomicU32=AtomicU32::new(0);
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
 let menu=forge::graphics::Graphics::is_menu_visible();let events=controller::EVENTS.swap(0,Ordering::AcqRel);let held=controller::HELD.load(Ordering::Acquire);
 let cue=STATE.with(|s|{
  let was=s.visible;s.visible=snap.as_ref().is_some_and(|n|n.extra);
  if let Some(n)=snap{s.select(n.id,n.note,n.pages);}
  if let Some(p)=progress{s.progress=p;s.progress_age=0.;if !was{s.mode=0;s.row=0;s.scroll=0;s.clock=0.;s.reset_form()}}
  if !s.visible{s.mode=0;s.row=0;s.scroll=0;s.detail_scroll=0;s.clock=0.;return None}
  let old=(s.mode,s.tab,s.row,s.rank_cursor,s.detail_scroll,s.form_cursor);
  if !menu{s.action(events);s.tick(dt,held);}
  if s.tab!=old.1{Some(27)}else if s.mode!=old.0{Some(if events&model::BACK!=0{2}else{1})}
  else if (s.row,s.rank_cursor,s.detail_scroll,s.form_cursor)!=(old.2,old.3,old.4,old.5){Some(4)}else{None}
 });
 resources::tick_retry(dt,STATE.with(|s|s.visible));
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

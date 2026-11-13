//! Opt-in diagnostic sampling; no logging or clocks when profile=0 (default).
use core::sync::atomic::{AtomicU32,Ordering};
pub static MODE:AtomicU32=AtomicU32::new(0);
pub fn legacy()->bool{MODE.load(Ordering::Relaxed)==1}
pub fn start()->u64{if MODE.load(Ordering::Relaxed)==0{return 0}ticks()}
fn ticks()->u64{let(lo,hi):(u32,u32);unsafe{core::arch::asm!("svc 0x1e",lateout("r0")lo,lateout("r1")hi,options(nostack));}lo as u64|((hi as u64)<<32)}
static mut TAG:u32=u32::MAX;static mut COUNT:usize=0;
static mut FRAME:[u32;600]=[0;600];static mut UPDATE:[u32;600]=[0;600];static mut DRAW:[u32;600]=[0;600];
static mut LAST_UPDATE:u32=0;
fn micros(t:u64)->u32{((t*10)/192) as u32}
pub fn update_done(start:u64){if start!=0{unsafe{LAST_UPDATE=micros(ticks()-start)}}}
pub fn render_done(start:u64){if start==0{return}let elapsed=micros(ticks()-start);let(tag,dt)=crate::STATE.with(|s|{let tag=if s.visible{100+s.tab as u32*10+s.mode as u32}else{0};(tag,forge::imgui::get_io().DeltaTime)});unsafe{
 if TAG!=tag{TAG=tag;COUNT=0;}let n=COUNT;if n>=600{return}FRAME[n]=(dt*1000000.) as u32;UPDATE[n]=LAST_UPDATE;DRAW[n]=elapsed;COUNT+=1;
 if COUNT==600{let mut frame=core::ptr::addr_of!(FRAME).read();let mut update=core::ptr::addr_of!(UPDATE).read();let mut draw=core::ptr::addr_of!(DRAW).read();frame.sort_unstable();update.sort_unstable();draw.sort_unstable();
  forge::log::prelude::info!("[monster-info-perf] mode={} tag={} n=600 frame_us median={} p95={} max={} update_us median={} p95={} draw_us median={} p95={}",MODE.load(Ordering::Relaxed),tag,frame[300],frame[570],frame[599],update[300],update[570],draw[300],draw[570]);COUNT=0;
 }
}}

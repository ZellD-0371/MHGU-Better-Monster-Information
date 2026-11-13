use alloc::{ffi::CString,string::String};
use core::{cell::UnsafeCell,sync::atomic::{AtomicBool,Ordering}};
use crate::language_pack::{Pack,Store};
struct Shared{lock:AtomicBool,data:UnsafeCell<Store>}
unsafe impl Sync for Shared{}
static DATA:Shared=Shared{lock:AtomicBool::new(false),data:UnsafeCell::new(Store{active:None})};
impl Shared{fn with<R>(&self,f:impl FnOnce(&mut Store)->R)->R{while self.lock.compare_exchange_weak(false,true,Ordering::Acquire,Ordering::Relaxed).is_err(){core::hint::spin_loop()}let result=f(unsafe{&mut*self.data.get()});self.lock.store(false,Ordering::Release);result}}
pub fn current()->Option<usize>{DATA.with(|s|s.active.as_ref().map(|p|p.language))}
pub fn text(id:u32)->String{DATA.with(|s|s.active.as_ref().map_or_else(||"—".into(),|p|p.text(id)))}
pub fn prepare(lang:usize)->Result<Pack,()>{let code=["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"].get(lang).copied().ok_or(())?;let path=CString::new(alloc::format!("app:/nativeNX/plugins/better-monster-information/languages/text.{}.bin",code)).unwrap();let bytes=unsafe{crate::resources::read_file(&path)}.ok_or(())?;Pack::prepare(lang,bytes)}
pub fn commit(next:Pack){let lang=next.language;let bytes=next.len();let old=DATA.with(|s|s.commit(next));drop(old);forge::log::prelude::info!("[monster-info] language={} active text bytes={}; previous pack released",lang,bytes);}
pub fn reset(){let old=DATA.with(|s|s.active.take());drop(old);}
pub fn font_valid()->bool{unsafe{
 use forge::sys::imgui as sys;let f=sys::igImFileOpen(c"app:/nativeNX/plugins/better-monster-information/languages/font.shared.otf".as_ptr(),c"rb".as_ptr());if f.is_null(){return false}
 let(size,wanted)=crate::language_pack::FONT;let mut crc=!0;let mut remaining=size;let mut buf=[0u8;4096];let mut ok=sys::igImFileGetSize(f)==size as u64;
 while ok&&remaining>0{let n=remaining.min(buf.len());if sys::igImFileRead(buf.as_mut_ptr().cast(),1,n as u64,f)!=n as u64{ok=false;break}crc=crate::language_pack::crc_step(crc,&buf[..n]);remaining-=n;}
 sys::igImFileClose(f);ok&&!crc==wanted
}}

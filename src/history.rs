//! Exact future rank hunts, isolated by the native Guild Card owner ID.
//! The native quest-result callback already excludes abandoned quests and zero hunts.
use core::sync::atomic::{AtomicU32,Ordering};
use alloc::ffi::CString;
static PENDING:[AtomicU32;130]=[const{AtomicU32::new(0)};130];
static PENDING_LO:AtomicU32=AtomicU32::new(0);static PENDING_HI:AtomicU32=AtomicU32::new(0);
static mut KEY:u64=0;static mut MASKS:[u16;130]=[0;130];static mut DIRTY:bool=false;
static mut DELAY:u32=0;
#[forge::hook(offset=0x1c95a4)]
fn record_hunt(owner:u32,code:u32)->u32{
 let result=original!(owner,code);let base=crate::BASE.load(Ordering::Acquire);
 let Some(key)=crate::native::character_key(base)else{return result};
 let quests=unsafe{crate::native::rd(base+0x1897f78)};if !crate::native::valid(quests){return result}
 let data=unsafe{crate::native::rd(quests+0xa4)};if !crate::native::valid(data){return result}
 let raw=unsafe{crate::native::byte(data+0xf)};let rank=if raw==5{3}else if raw==3{2}else{1};
 let quest=unsafe{crate::native::rd(data+8)};
 if let Some(id)=crate::generated::HUNT_CODES.iter().position(|&c|c as u32==code&&c!=0){
  let mask=crate::history_format::event_mask(id as u16,rank,quest);
  PENDING_LO.store(key as u32,Ordering::Release);PENDING_HI.store((key>>32) as u32,Ordering::Release);
  PENDING[id].fetch_or(mask as u32,Ordering::AcqRel);
 }result
}
pub fn install(base:u32)->bool{
 if unsafe{crate::native::rd(base+0x1c95a4)}!=0xe92d4830||unsafe{crate::native::rd(base+0x1c95a8)}!=0xe1a04001{return false}
 forge::install_hook!(base,record_hunt);true
}
pub fn update(base:u32){unsafe{
 let Some(key)=crate::native::character_key(base)else{return};
 if DELAY>0{DELAY-=1;return}
 if KEY==key&&!DIRTY&&PENDING.iter().all(|p|p.load(Ordering::Relaxed)==0){return}
 let name=CString::new(alloc::format!("savedata:/better-monster-information-hunts-{:016x}.ini",key)).unwrap();
 let location=crate::storage::Location{path:&name,display:"Better Monster Information rank history",defaults:""};
 if KEY!=key{
  let mut bytes=[0;crate::storage::MAX_BYTES];let Ok(text)=location.read(&mut bytes)else{DELAY=120;return};
  MASKS=crate::history_format::decode(text,key).unwrap_or([0;130]);KEY=key;DIRTY=false;
 }
 let pending_key=PENDING_LO.load(Ordering::Acquire) as u64|((PENDING_HI.load(Ordering::Acquire) as u64)<<32);
 for(id,pending)in PENDING.iter().enumerate(){let bits=pending.swap(0,Ordering::AcqRel) as u16;if pending_key==key&&bits!=0{let before=MASKS[id];MASKS[id]|=bits;DIRTY|=before!=MASKS[id];}}
 if DIRTY{let snapshot=core::ptr::addr_of!(MASKS).read();if location.save(&crate::history_format::encode(key,&snapshot)).is_ok(){DIRTY=false}else{DELAY=120;}}
}}
pub fn merge(p:&mut crate::model::Progress){unsafe{if crate::native::character_key(crate::BASE.load(Ordering::Acquire))!=Some(KEY){return}let masks=core::ptr::addr_of!(MASKS).read();for(id,mask)in masks.iter().enumerate(){p.hunted[id]|=*mask;}}}

//! Chain the existing five HID wrappers; consume inputs only on our added page.
use core::{ptr,sync::atomic::{AtomicU32,AtomicBool,Ordering}};
use crate::{STATE,model as m};
const GOT:[u32;5]=[0x1838494,0x1838498,0x183849c,0x18384a0,0x18384a4];
const PLT:[u32;5]=[0x13f0af0,0x13f0afc,0x13f0b08,0x13f0b14,0x13f0b20];
static ORIGINAL:[AtomicU32;5]=[const{AtomicU32::new(0)};5];
static PREVIOUS:AtomicU32=AtomicU32::new(0);
static RELEASE:AtomicU32=AtomicU32::new(0);
static WAIT_NEUTRAL:AtomicBool=AtomicBool::new(false);
pub static EVENTS:AtomicU32=AtomicU32::new(0);
pub static HELD:AtomicU32=AtomicU32::new(0);
#[unsafe(no_mangle)]
pub static monster_info_input_audit:[AtomicU32;8]=[const{AtomicU32::new(0)};8];
unsafe fn read(p:u32)->u32{unsafe{ptr::read_unaligned(p as *const u32)}}
unsafe fn write(p:u32,v:u32){unsafe{ptr::write_unaligned(p as *mut u32,v)}}
fn events(raw:u32)->u32{
 let mut e=0;for (mask,event) in [(1,m::ACCEPT),(2,m::BACK),(4,m::RANK),(8,m::FORM),(256,m::PREV),(512,m::NEXT),((1<<13)|(1<<17),m::UP),((1<<15)|(1<<19),m::DOWN),(1<<21,m::SCROLL_UP),(1<<23,m::SCROLL_DOWN)]{if raw&mask!=0{e|=event}}e
}
unsafe fn filter(index:usize,state:u32,id:u32){unsafe{
 let call:unsafe extern "C" fn(u32,u32)=core::mem::transmute(ORIGINAL[index].load(Ordering::Acquire));call(state,id);
 if state<0x10000||read(state+32)&1==0{return}
 let buttons=read(state+8);let mut raw=buttons;
 let x=read(state+16) as i32;let y=read(state+20) as i32;
 if x < -14000{raw|=1<<16}if x>14000{raw|=1<<18}if y>14000{raw|=1<<17}if y < -14000{raw|=1<<19}
 let ry=read(state+28) as i32;if ry>14000{raw|=1<<21}if ry < -14000{raw|=1<<23}
 let menu=forge::graphics::Graphics::is_menu_visible();
 if menu{WAIT_NEUTRAL.store(true,Ordering::Release);EVENTS.store(0,Ordering::Release);HELD.store(0,Ordering::Release);PREVIOUS.store(raw,Ordering::Release);return}
 let (active,mode,opening)=STATE.with(|s|(s.visible&&s.ready&&s.input_ready,s.mode,s.mode==0&&((s.interactive()&&raw&1!=0)||(s.tab==3&&s.ranks.len()>1&&raw&4!=0)||(raw&8!=0&&s.forms().len()>1))));
 let waiting=WAIT_NEUTRAL.load(Ordering::Acquire);
 if waiting&&raw==0&&read(state+24)==0&&read(state+28)==0{WAIT_NEUTRAL.store(false,Ordering::Release);}
 let previous=PREVIOUS.swap(raw,Ordering::AcqRel);let press=raw&!previous;
 let old_release=RELEASE.load(Ordering::Acquire)&raw;
 let capture=if active{if mode!=0||opening||waiting{raw}else{raw&(1|4|8|256|512)}}else{old_release};
 let capture=capture|old_release;RELEASE.store(capture,Ordering::Release);
 if active&&!waiting{
  let accepted=if mode!=0{press}else{press&(1|4|8|256|512)};
  EVENTS.fetch_or(events(accepted),Ordering::AcqRel);HELD.store(events(raw)&(m::UP|m::DOWN),Ordering::Release);
 }else{EVENTS.store(0,Ordering::Release);HELD.store(0,Ordering::Release);}
 let filtered=buttons&!capture;write(state+8,filtered);
 if active&&(mode!=0||opening||waiting){write(state+12,0);for offset in [16,20,24,28]{write(state+offset,0)}}
 else if capture&0xf0000!=0{write(state+16,0);write(state+20,0);}
 if capture&0xf00000!=0{write(state+24,0);write(state+28,0);}
 for (i,value) in [buttons,filtered,raw,capture,active as u32,mode as u32,waiting as u32].iter().enumerate(){monster_info_input_audit[i].store(*value,Ordering::Relaxed);}
 monster_info_input_audit[7].fetch_add(1,Ordering::Release);
}}
unsafe extern "C" fn full(s:u32,id:u32){unsafe{filter(0,s,id)}}
unsafe extern "C" fn dual(s:u32,id:u32){unsafe{filter(1,s,id)}}
unsafe extern "C" fn left(s:u32,id:u32){unsafe{filter(2,s,id)}}
unsafe extern "C" fn right(s:u32,id:u32){unsafe{filter(3,s,id)}}
unsafe extern "C" fn handheld(s:u32,id:u32){unsafe{filter(4,s,id)}}
pub fn install(base:u32)->bool{
 let detours=[full as *const (),dual as *const (),left as *const (),right as *const (),handheld as *const ()];
 for (i,off) in GOT.iter().enumerate(){let mut address=unsafe{read(base+off)};
  if address==base+0x13efd44{
   #[repr(C,align(8))]struct Pad([u8;40]);let mut pad=Pad([0;40]);let port=0;
   let bind:unsafe extern "C" fn(*mut Pad,*const u32)=unsafe{core::mem::transmute(base+PLT[i])};unsafe{bind(&mut pad,&port)};address=unsafe{read(base+off)};
  }
  if address<0x10000||address&3!=0||address==base+0x13efd44{return false}ORIGINAL[i].store(address,Ordering::Release);
 }
 for (i,off) in GOT.iter().enumerate(){unsafe{ptr::write_volatile((base+off) as *mut u32,detours[i] as u32)}}true
}

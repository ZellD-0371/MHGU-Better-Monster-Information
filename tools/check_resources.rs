//! Exercise the actual resource loader against a delayed, checked texture backend.
//! This detects ownership/order errors; it does not replace a real GPU game review.
#![allow(non_snake_case,non_camel_case_types,dead_code,unsafe_op_in_unsafe_fn)]
extern crate alloc;
extern crate self as forge;
#[path="../src/data.rs"]mod data;
#[cfg(not(bmi_low_memory))]#[path="../src/generated.rs"]mod generated;
#[cfg(bmi_low_memory)]#[path="../src/generated_low.rs"]mod generated;
#[path="../src/retry.rs"]mod retry;
#[path="../src/diagram_lifetime.rs"]mod diagram_lifetime;
#[path="../src/resources.rs"]mod resources;
mod language{pub fn text(_:u32)->alloc::rc::Rc<str>{"test".into()}}
#[macro_export]macro_rules! test_log{($($arg:tt)*)=>{let _=format_args!($($arg)*);}}
pub mod log{pub mod prelude{pub use crate::test_log as info;pub use crate::test_log as warn;}}
use std::sync::{Mutex,atomic::{AtomicUsize,Ordering}};
static REGISTERED:Mutex<Vec<usize>>=Mutex::new(Vec::new());
static ALLOCATED:AtomicUsize=AtomicUsize::new(0);
static FREED:AtomicUsize=AtomicUsize::new(0);
static DRAWS:AtomicUsize=AtomicUsize::new(0);
static LIVE_PIXEL_BYTES:AtomicUsize=AtomicUsize::new(0);
static OPEN_FILES:AtomicUsize=AtomicUsize::new(0);
static ATLAS_OPENS:AtomicUsize=AtomicUsize::new(0);
// Inject errors without modifying any on-disk asset: missing/size/read/pixel allocation.
static ATLAS_FAILURE:AtomicUsize=AtomicUsize::new(0);
pub mod imgui{
 #[derive(Clone,Copy)]pub struct DrawList;
 impl DrawList{pub fn add_image(self,t:crate::sys::imgui::ImTextureRef,_a:crate::sys::imgui::ImVec2,_b:crate::sys::imgui::ImVec2,_uv1:crate::sys::imgui::ImVec2,_uv2:crate::sys::imgui::ImVec2,_tint:u32){unsafe{
  assert!(crate::REGISTERED.lock().unwrap().contains(&(t.0 as usize)));
  assert_eq!((*t.0).Status,crate::sys::imgui::ImTextureStatus::OK);
  assert!(!(*t.0).WantDestroyNextFrame);assert_ne!((*t.0).TexID,0);
  crate::DRAWS.fetch_add(1,crate::Ordering::Relaxed);
 }}}
}
pub mod sys{pub mod imgui{
 use std::{ffi::{CStr,c_void},fs::File,io::Read};
 #[derive(Clone,Copy)]pub struct ImVec2{pub x:f32,pub y:f32}
 #[derive(Clone,Copy)]pub struct ImTextureRef(pub *mut ImTextureData);
 #[derive(Clone,Copy,PartialEq,Debug)]pub enum ImTextureStatus{Destroyed,OK,WantCreate,WantUpdates,WantDestroy}
 pub enum ImTextureFormat{RGBA32}
 #[derive(Clone,Copy,Default)]pub struct ImTextureRect{pub x:u16,pub y:u16,pub w:u16,pub h:u16}
 pub struct ImTextureData{
  pub Status:ImTextureStatus,pub WantDestroyNextFrame:bool,pub UnusedFrames:i32,
  pub UsedRect:ImTextureRect,pub UpdateRect:ImTextureRect,pub TexID:u64,
  pixels:Vec<u8>,
 }
 struct TestFile{file:File,atlas:bool}
 pub unsafe fn igImFileOpen(path:*const i8,_mode:*const i8)->*mut c_void{
  let path=CStr::from_ptr(path).to_str().unwrap();
  let relative=path.strip_prefix("app:/nativeNX/plugins/better-monster-information/").unwrap();
  let atlas=relative=="monster-icons.rgba";
  if atlas{crate::ATLAS_OPENS.fetch_add(1,crate::Ordering::Relaxed);if crate::ATLAS_FAILURE.load(crate::Ordering::Relaxed)==1{return std::ptr::null_mut()}}
  let root=if cfg!(bmi_low_memory){"assets/native-low"}else{"assets/native"};
  File::open(std::path::Path::new(root).join(relative)).ok().map_or(std::ptr::null_mut(),|file|{crate::OPEN_FILES.fetch_add(1,crate::Ordering::Relaxed);Box::into_raw(Box::new(TestFile{file,atlas})).cast()})
 }
 pub unsafe fn igImFileGetSize(f:*mut c_void)->u64{let f=&*(f as *mut TestFile);let n=f.file.metadata().unwrap().len();if f.atlas&&crate::ATLAS_FAILURE.load(crate::Ordering::Relaxed)==2{n-4}else{n}}
 pub unsafe fn igImFileRead(out:*mut c_void,size:u64,count:u64,f:*mut c_void)->u64{
  let f=&mut*(f as *mut TestFile);if f.atlas&&crate::ATLAS_FAILURE.load(crate::Ordering::Relaxed)==3{return 0}
  f.file.read(std::slice::from_raw_parts_mut(out.cast(),(size*count) as usize)).unwrap() as u64/size
 }
 pub unsafe fn igImFileClose(f:*mut c_void){drop(Box::from_raw(f as *mut TestFile));crate::OPEN_FILES.fetch_sub(1,crate::Ordering::Relaxed);}
 pub unsafe fn ImTextureData_ImTextureData()->*mut ImTextureData{
  crate::ALLOCATED.fetch_add(1,crate::Ordering::Relaxed);
  Box::into_raw(Box::new(ImTextureData{Status:ImTextureStatus::Destroyed,WantDestroyNextFrame:false,UnusedFrames:0,UsedRect:ImTextureRect::default(),UpdateRect:ImTextureRect::default(),TexID:0,pixels:Vec::new()}))
 }
 pub unsafe fn ImTextureData_Create(t:*mut ImTextureData,_format:ImTextureFormat,w:i32,h:i32){
  assert_eq!((*t).Status,ImTextureStatus::Destroyed);(*t).pixels=vec![0;(w*h*4) as usize];(*t).Status=ImTextureStatus::WantCreate;crate::LIVE_PIXEL_BYTES.fetch_add((w*h*4) as usize,crate::Ordering::Relaxed);
 }
 pub unsafe fn ImTextureData_GetPixels(t:*mut ImTextureData)->*mut c_void{if crate::ATLAS_FAILURE.load(crate::Ordering::Relaxed)==4{std::ptr::null_mut()}else{(*t).pixels.as_mut_ptr().cast()}}
 pub unsafe fn ImTextureData_GetTexRef(t:*mut ImTextureData)->ImTextureRef{ImTextureRef(t)}
 pub unsafe fn ImTextureData_GetTexID(t:*mut ImTextureData)->u64{(*t).TexID}
 pub unsafe fn ImTextureData_SetStatus(t:*mut ImTextureData,s:ImTextureStatus){
  // Match ImGui's automatic recreation rule: forgetting this flag leaks/recreates GPU textures.
  (*t).Status=if s==ImTextureStatus::Destroyed&&!(*t).WantDestroyNextFrame&&!(*t).pixels.is_empty(){ImTextureStatus::WantCreate}else{s};
 }
 pub unsafe fn igRegisterUserTexture(t:*mut ImTextureData){
  let mut list=crate::REGISTERED.lock().unwrap();assert!(!list.contains(&(t as usize)));list.push(t as usize);
 }
 pub unsafe fn igUnregisterUserTexture(t:*mut ImTextureData){
  assert_eq!((*t).Status,ImTextureStatus::Destroyed);assert_eq!((*t).TexID,0);
  let mut list=crate::REGISTERED.lock().unwrap();let index=list.iter().position(|&p|p==t as usize).unwrap();list.remove(index);
 }
 pub unsafe fn ImTextureData_destroy(t:*mut ImTextureData){
  assert_eq!((*t).TexID,0,"GPU still owns this texture");
  assert!(!crate::REGISTERED.lock().unwrap().contains(&(t as usize)),"registered dangling pointer");
  crate::LIVE_PIXEL_BYTES.fetch_sub((*t).pixels.len(),crate::Ordering::Relaxed);
  drop(Box::from_raw(t));crate::FREED.fetch_add(1,crate::Ordering::Relaxed);
 }
}}
fn backend_frame(){use sys::imgui::*;for p in REGISTERED.lock().unwrap().iter().copied(){unsafe{
 let t=p as *mut ImTextureData;
 match (*t).Status{
  ImTextureStatus::WantCreate|ImTextureStatus::WantUpdates=>{(*t).TexID=1;ImTextureData_SetStatus(t,ImTextureStatus::OK);},
  ImTextureStatus::WantDestroy if (*t).UnusedFrames>=3=>{assert!((*t).WantDestroyNextFrame);(*t).TexID=0;ImTextureData_SetStatus(t,ImTextureStatus::Destroyed);},
  _=>{},
 }
}}}
fn draw(id:u16)->bool{resources::diagram(imgui::DrawList,id,sys::imgui::ImVec2{x:0.,y:0.},sys::imgui::ImVec2{x:298.,y:232.})}
fn texture()->usize{let list=REGISTERED.lock().unwrap();assert_eq!(list.len(),1);list[0]}
#[test]fn real_loaders_close_reopen_and_deferred_free_are_bounded(){
 use sys::imgui::ImTextureStatus as S;
 let id=generated::DIAGRAM_SOURCES.iter().position(|&(w,_)|w>0).unwrap() as u16;
 resources::maintain_diagram(false);assert!(REGISTERED.lock().unwrap().is_empty());
 for cycle in 0..100{
  resources::maintain_diagram(true);resources::load_diagram(id);
  let t=texture() as *mut sys::imgui::ImTextureData;
  assert_eq!(unsafe{(*t).Status},S::WantCreate);assert!(!draw(id));
  // Closing before upload completion must never free its pixel source.
  resources::maintain_diagram(false);assert_eq!(unsafe{(*t).Status},S::WantCreate);
  backend_frame();resources::maintain_diagram(true);assert!(draw(id));
  for _ in 0..3{resources::maintain_diagram(true);resources::load_diagram(id);assert_eq!(texture(),t as usize);}
  resources::maintain_diagram(false);assert_eq!(unsafe{(*t).Status},S::WantDestroy);assert!(!draw(id));
  // Reopen during destruction: do not draw/reload/re-register the retired object.
  let allocations=ALLOCATED.load(Ordering::Relaxed);
  for frame in 1..=3{
   if frame>1{resources::maintain_diagram(true);}
   resources::load_diagram(id);assert_eq!(ALLOCATED.load(Ordering::Relaxed),allocations);assert!(!draw(id));backend_frame();
  }
  assert_eq!(unsafe{(*t).Status},S::Destroyed);
  resources::maintain_diagram(true);assert!(REGISTERED.lock().unwrap().is_empty());
  assert_eq!(FREED.load(Ordering::Relaxed),cycle+1);
  assert_eq!(ALLOCATED.load(Ordering::Relaxed),FREED.load(Ordering::Relaxed));
  resources::maintain_diagram(false); // repeated close is a no-op, not a double free
 }
 assert_eq!(DRAWS.load(Ordering::Relaxed),100);
 check_icon_lifetime();check_both_textures(id);check_icon_failures();
}
fn draw_icon(){resources::icon(imgui::DrawList,0,sys::imgui::ImVec2{x:0.,y:0.},sys::imgui::ImVec2{x:22.,y:22.},u32::MAX,false);}
fn icons(open:bool){resources::maintain_icons(open,1./60.);}
fn assert_no_resources(){
 assert!(REGISTERED.lock().unwrap().is_empty());assert_eq!(LIVE_PIXEL_BYTES.load(Ordering::Relaxed),0);
 assert_eq!(OPEN_FILES.load(Ordering::Relaxed),0);assert_eq!(ALLOCATED.load(Ordering::Relaxed),FREED.load(Ordering::Relaxed));
}
fn finish_icon_close(){
 icons(false);for _ in 0..4{backend_frame();icons(false);}assert_no_resources();
}
fn check_icon_lifetime(){
 use sys::imgui::ImTextureStatus as S;
 let start_opens=ATLAS_OPENS.load(Ordering::Relaxed);let start_draws=DRAWS.load(Ordering::Relaxed);
 for _ in 0..100{icons(false);}assert_no_resources();assert_eq!(ATLAS_OPENS.load(Ordering::Relaxed),start_opens);
 for cycle in 0..100{
  icons(true);let t=texture() as *mut sys::imgui::ImTextureData;
  assert_eq!(LIVE_PIXEL_BYTES.load(Ordering::Relaxed),2048*generated::ATLAS_HEIGHT*4);
  assert_eq!(unsafe{(*t).Status},S::WantCreate);draw_icon();assert_eq!(DRAWS.load(Ordering::Relaxed),start_draws+cycle);
  // Closing mid-upload retains the pixel source until it is safe to retire.
  icons(false);assert_eq!(unsafe{(*t).Status},S::WantCreate);
  backend_frame();icons(true);draw_icon();assert_eq!(DRAWS.load(Ordering::Relaxed),start_draws+cycle+1);
  for _ in 0..5{icons(true);assert_eq!(texture(),t as usize);} // tabs/monsters share the same atlas
  assert_eq!(ATLAS_OPENS.load(Ordering::Relaxed),start_opens+cycle+1);
  icons(false);assert_eq!(unsafe{(*t).Status},S::WantDestroy);
  let allocations=ALLOCATED.load(Ordering::Relaxed);
  for frame in 1..=3{
   if frame>1{icons(true);}draw_icon();assert_eq!(DRAWS.load(Ordering::Relaxed),start_draws+cycle+1);
   assert_eq!(ALLOCATED.load(Ordering::Relaxed),allocations);backend_frame();
  }
  assert_eq!(unsafe{(*t).Status},S::Destroyed);icons(false);assert_no_resources();
 }
 // If reopened just as destruction completes, free the old atlas before allocating another.
 icons(true);backend_frame();icons(false);
 for frame in 1..=3{if frame>1{icons(true);}backend_frame();}
 let before_alloc=ALLOCATED.load(Ordering::Relaxed);let before_free=FREED.load(Ordering::Relaxed);
 icons(true);assert_eq!(ALLOCATED.load(Ordering::Relaxed),before_alloc+1);assert_eq!(FREED.load(Ordering::Relaxed),before_free+1);
 assert_eq!(REGISTERED.lock().unwrap().len(),1);assert_eq!(LIVE_PIXEL_BYTES.load(Ordering::Relaxed),2048*generated::ATLAS_HEIGHT*4);
 backend_frame();icons(true);draw_icon();finish_icon_close();
}
fn check_both_textures(id:u16){
 for _ in 0..20{
  icons(true);resources::maintain_diagram(true);resources::load_diagram(id);
  assert_eq!(REGISTERED.lock().unwrap().len(),2);
  assert_eq!(LIVE_PIXEL_BYTES.load(Ordering::Relaxed),2048*generated::ATLAS_HEIGHT*4+generated::DIAGRAM_TEX_W*generated::DIAGRAM_TEX_H*4);
  backend_frame();icons(true);resources::maintain_diagram(true);draw_icon();assert!(draw(id));
  icons(false);resources::maintain_diagram(false);
  for _ in 0..4{backend_frame();icons(false);resources::maintain_diagram(false);}
  assert_no_resources();
 }
}
fn check_icon_failures(){
 for failure in 1..=4{
  let before=ATLAS_OPENS.load(Ordering::Relaxed);ATLAS_FAILURE.store(failure,Ordering::Relaxed);
  for _ in 0..2000{icons(true);backend_frame();}
  assert_eq!(ATLAS_OPENS.load(Ordering::Relaxed)-before,3);assert_no_resources();
  // Recovery waits for a close/reopen once all three attempts have been used.
  ATLAS_FAILURE.store(0,Ordering::Relaxed);icons(true);assert_no_resources();
  icons(false);icons(true);assert_eq!(REGISTERED.lock().unwrap().len(),1);
  backend_frame();icons(true);draw_icon();finish_icon_close();
 }
}

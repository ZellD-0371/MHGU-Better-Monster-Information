use alloc::{vec::Vec,ffi::CString};
use core::ffi::CStr;
use forge::{imgui::DrawList,sys::imgui::{self as sys,ImVec2,ImTextureRef}};
use crate::generated as g;
static mut ICONS:Option<(*mut sys::ImTextureData,ImTextureRef)>=None;
static mut ICON_LIFETIME:crate::diagram_lifetime::Lifetime=crate::diagram_lifetime::Lifetime::NEW;
static mut ICON_RETRY:crate::retry::RetryGate=crate::retry::RetryGate::NEW;
static mut DIAGRAM:Option<(*mut sys::ImTextureData,ImTextureRef,u16)>=None;
static mut LIFETIME:crate::diagram_lifetime::Lifetime=crate::diagram_lifetime::Lifetime::NEW;

pub(crate) unsafe fn read_file(path:&CStr)->Option<Vec<u8>>{unsafe{
    let file=sys::igImFileOpen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return None}
    let size=sys::igImFileGetSize(file);if size>16*1024*1024{sys::igImFileClose(file);return None}
    let mut bytes=Vec::new();if bytes.try_reserve_exact(size as usize).is_err(){sys::igImFileClose(file);return None}bytes.resize(size as usize,0);let read=sys::igImFileRead(bytes.as_mut_ptr().cast(),1,size,file);sys::igImFileClose(file);
    if read!=size{None}else{Some(bytes)}
}}
unsafe fn texture(path:&CStr,w:u16,h:u16)->Option<(*mut sys::ImTextureData,ImTextureRef)>{unsafe{
    let file=sys::igImFileOpen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return None}
    let size=w as u64*h as u64*4;if sys::igImFileGetSize(file)!=size{sys::igImFileClose(file);return None}
    let tex=sys::ImTextureData_ImTextureData();if tex.is_null(){sys::igImFileClose(file);return None}
    sys::ImTextureData_Create(tex,sys::ImTextureFormat::RGBA32,w as i32,h as i32);
    let pixels=sys::ImTextureData_GetPixels(tex);
    if pixels.is_null(){sys::igImFileClose(file);sys::ImTextureData_destroy(tex);return None}
    let read=sys::igImFileRead(pixels,1,size,file);sys::igImFileClose(file);
    if read!=size{sys::ImTextureData_destroy(tex);return None}
    sys::igRegisterUserTexture(tex);Some((tex,sys::ImTextureData_GetTexRef(tex)))
}}
// Rules have identical strings in every pack; UI reads the committed language.
pub fn text(id:u32,_lang:usize)->alloc::rc::Rc<str>{crate::language::text(id)}
fn dimensions()->(usize,usize){(g::DIAGRAM_TEX_W,g::DIAGRAM_TEX_H)}
static mut RETRY:crate::retry::RetryGate=crate::retry::RetryGate::NEW;
pub fn tick_retry(dt:f32,visible:bool){unsafe{(&mut*core::ptr::addr_of_mut!(RETRY)).tick(dt,visible);}}
// Run on the ImGui render callback, before adding this frame's draw commands.
// Keep textures registered until the backend confirms their GPU objects are gone.
unsafe fn retire_texture(tex:Option<*mut sys::ImTextureData>,lifetime:&mut crate::diagram_lifetime::Lifetime,notes_open:bool)->bool{unsafe{
 use crate::diagram_lifetime::{Action,Status};
 let status=tex.map(|tex|{
  let _=sys::ImTextureData_GetTexID(tex); // refresh Recomp's guest mirror
  match (*tex).Status{sys::ImTextureStatus::OK=>Status::Ready,sys::ImTextureStatus::Destroyed=>Status::Destroyed,_=>Status::Busy}
 });
 match lifetime.advance(notes_open,status){
  Action::Keep=>{},
  Action::RequestDestroy=>if let Some(tex)=tex{
   // Prevent SetStatus(Destroyed) from automatically requesting recreation.
   (*tex).WantDestroyNextFrame=true;(*tex).UnusedFrames=1;
   sys::ImTextureData_SetStatus(tex,sys::ImTextureStatus::WantDestroy);
  },
  Action::WaitForDestroy(frames)=>if let Some(tex)=tex{
   // ImGui ages font textures automatically, but not registered user textures.
   (*tex).UnusedFrames=frames;let _=sys::ImTextureData_GetTexID(tex);
  },
  Action::Free=>return true,
 }false
}}
unsafe fn free_texture(tex:*mut sys::ImTextureData){unsafe{
 sys::igUnregisterUserTexture(tex);
 // The Forge bridge also frees its guest pixel copy in this destructor.
 sys::ImTextureData_destroy(tex);
}}
pub fn maintain_diagram(notes_open:bool){unsafe{
 if retire_texture(DIAGRAM.map(|(tex,_,_)|tex),&mut*core::ptr::addr_of_mut!(LIFETIME),notes_open){
  if let Some((tex,_,id))=core::ptr::addr_of_mut!(DIAGRAM).replace(None){
   free_texture(tex);crate::diagnostic_info!("[monster-info] diagram {} released after notes closed",id);
  }
 }
}}
pub fn maintain_icons(notes_open:bool,dt:f32){unsafe{
 let lifetime=&mut*core::ptr::addr_of_mut!(ICON_LIFETIME);
 if retire_texture(ICONS.map(|(tex,_)|tex),lifetime,notes_open){
  if let Some((tex,_))=core::ptr::addr_of_mut!(ICONS).replace(None){
   free_texture(tex);crate::diagnostic_info!("[monster-info] icon atlas released after notes closed");
  }
 }
 let retry=&mut*core::ptr::addr_of_mut!(ICON_RETRY);retry.tick(dt,notes_open);
 if !notes_open||lifetime.retiring()||core::ptr::addr_of!(ICONS).read().is_some(){return}
 retry.select(0);if !retry.allows(){return}
 if let Some(texture)=texture(c"app:/nativeNX/plugins/better-monster-information/monster-icons.rgba",2048,g::ATLAS_HEIGHT as u16){
  ICONS=Some(texture);retry.succeeded();crate::diagnostic_info!("[monster-info] icon atlas loaded on demand");
 }else{retry.failed();forge::log::prelude::warn!("[monster-info] icon atlas load failed; retry limited");}
}}
pub fn load_diagram(id:u16){unsafe{
 if (&*core::ptr::addr_of!(LIFETIME)).retiring(){return}
 let retry=&mut*core::ptr::addr_of_mut!(RETRY);retry.select(id);
 if let Some((tex,_,old))=DIAGRAM{
  // Recomp mirrors ImTextureData. The renderer changes the host object after
  // this callback; refresh via its API before testing the guest Status copy.
  let _=sys::ImTextureData_GetTexID(tex);
  if old==id||(*tex).Status!=sys::ImTextureStatus::OK{return}
 }
 if !retry.allows(){return}
 if crate::generated::DIAGRAM_SOURCES.get(id as usize).is_none_or(|&(w,_)|w==0){return}
 match try_load_diagram(id){Ok(())=>retry.succeeded(),Err(reason)=>{retry.failed();forge::log::prelude::warn!("[monster-info] diagram {}: {}; retry limited",id,reason);}}
}}
unsafe fn try_load_diagram(id:u16)->Result<(),&'static str>{unsafe{
 let(w,h)=g::DIAGRAM_SOURCES.get(id as usize).copied().unwrap_or((0,0));if w==0{return Err("invalid diagram metadata")}
 
 let path=CString::new(alloc::format!("app:/nativeNX/plugins/better-monster-information/parts/{}.rgba",id)).unwrap();
 let file=sys::igImFileOpen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return Err("file open failed")}
 if sys::igImFileGetSize(file)!=w as u64*h as u64*4{sys::igImFileClose(file);return Err("file size mismatch")}
 let(tw,th)=dimensions();if w as usize>tw||h as usize>th{sys::igImFileClose(file);return Err("diagram exceeds texture canvas")}
 let(tex,reference,fresh)=if let Some((tex,reference,_))=DIAGRAM{(tex,reference,false)}else{
  
  let tex=sys::ImTextureData_ImTextureData();if tex.is_null(){sys::igImFileClose(file);return Err("texture object allocation failed")}
  // Use Forge's texture API: Recomp marshals host pixels into guest memory.
  // Assigning a guest allocation directly to Pixels bypasses that ownership bridge.
  sys::ImTextureData_Create(tex,sys::ImTextureFormat::RGBA32,tw as i32,th as i32);
  let pixels=sys::ImTextureData_GetPixels(tex) as *mut u8;
  if pixels.is_null(){sys::ImTextureData_destroy(tex);sys::igImFileClose(file);return Err("CPU pixels allocation failed")}
  (tex,sys::ImTextureData_GetTexRef(tex),true)
 };
 let pixels=sys::ImTextureData_GetPixels(tex) as *mut u8;if pixels.is_null(){sys::igImFileClose(file);if fresh{sys::ImTextureData_destroy(tex);}return Err("CPU pixel mapping failed")}core::ptr::write_bytes(pixels,0,tw*th*4);
 // Both variants read ready-to-display rows directly into the reused texture.
 // Low Memory ships pre-resized files; no runtime resize or second image buffer.
 let mut ok=true;
 for row in 0..h as usize{
  let count=w as u64*4;
  if sys::igImFileRead(pixels.add(row*tw*4).cast(),1,count,file)!=count{ok=false;break}
 }
 sys::igImFileClose(file);if !ok{if fresh{sys::ImTextureData_destroy(tex);}return Err("image read failed")}
 let rect=sys::ImTextureRect{x:0,y:0,w:tw as u16,h:th as u16};(*tex).UsedRect=rect;
 if fresh{sys::igRegisterUserTexture(tex);}else{
  (*tex).UpdateRect=rect;
  sys::ImTextureData_SetStatus(tex,sys::ImTextureStatus::WantUpdates);
 }
 DIAGRAM=Some((tex,reference,id));Ok(())
}}
pub fn icon(draw:DrawList,id:usize,p1:ImVec2,p2:ImVec2,tint:u32,flip:bool){unsafe{
    sprite(draw,id,p1,p2,tint,flip,false);
}}
pub fn sprite(draw:DrawList,id:usize,p1:ImVec2,p2:ImVec2,tint:u32,flip_x:bool,flip_y:bool){unsafe{
    if (&*core::ptr::addr_of!(ICON_LIFETIME)).retiring(){return}
    let Some((tex,reference))=ICONS else{return};if (*tex).Status!=sys::ImTextureStatus::OK{return}
    let Some(sprite)=g::SPRITES.get(id) else{return};
    let (left,right)=if flip_x{(sprite.x+sprite.w,sprite.x)}else{(sprite.x,sprite.x+sprite.w)};
    let (top,bottom)=if flip_y{(sprite.y+sprite.h,sprite.y)}else{(sprite.y,sprite.y+sprite.h)};
    draw.add_image(reference,p1,p2,ImVec2{x:left as f32/2048.,y:top as f32/g::ATLAS_HEIGHT as f32},ImVec2{x:right as f32/2048.,y:bottom as f32/g::ATLAS_HEIGHT as f32},tint);
}}
pub fn diagram(draw:DrawList,id:u16,p1:ImVec2,p2:ImVec2)->bool{unsafe{
    if (&*core::ptr::addr_of!(LIFETIME)).retiring(){return false}
    let Some((tex,reference,loaded))=DIAGRAM else{return false};if loaded!=id||(*tex).Status!=sys::ImTextureStatus::OK{return false}
    let(w,h)=g::DIAGRAM_SOURCES[id as usize];let(tw,th)=dimensions();draw.add_image(reference,p1,p2,ImVec2{x:0.,y:0.},ImVec2{x:w as f32/tw as f32,y:h as f32/th as f32},0xffffffff);true
}}

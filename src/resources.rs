use alloc::{vec::Vec,ffi::CString};
use core::ffi::CStr;
use forge::{imgui::DrawList,sys::imgui::{self as sys,ImVec2,ImTextureRef}};
use crate::generated as g;
static mut ICONS:Option<ImTextureRef>=None;
static mut DIAGRAM:Option<(*mut sys::ImTextureData,ImTextureRef,u16)>=None;

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
pub fn init()->bool{unsafe{
    ICONS=texture(c"app:/nativeNX/plugins/better-monster-information/monster-icons.rgba",2048,g::ATLAS_HEIGHT as u16).map(|(_,reference)|reference);
    let ready=core::ptr::addr_of!(ICONS).read().is_some();forge::log::prelude::info!("[monster-info] icon atlas ready={}",ready);ready
}}
// Rules have identical strings in every pack; UI reads the committed language.
pub fn text(id:u32,_lang:usize)->alloc::string::String{crate::language::text(id)}
fn dimensions()->(usize,usize){if cfg!(bmi_low_memory){(384,256)}else{(g::DIAGRAM_TEX_W,g::DIAGRAM_TEX_H)}}
pub fn load_diagram(id:u16){unsafe{
 if let Some((tex,_,old))=DIAGRAM{
  // Recomp mirrors ImTextureData. The renderer changes the host object after
  // this callback; refresh via its API before testing the guest Status copy.
  let _=sys::ImTextureData_GetTexID(tex);
  if old==id||(*tex).Status!=sys::ImTextureStatus::OK{return}
 }
 let(w,h)=g::DIAGRAM_SOURCES.get(id as usize).copied().unwrap_or((0,0));if w==0{return}
 
 let path=CString::new(alloc::format!("app:/nativeNX/plugins/better-monster-information/parts/{}.rgba",id)).unwrap();
 let file=sys::igImFileOpen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return}
 if sys::igImFileGetSize(file)!=w as u64*h as u64*4{sys::igImFileClose(file);return}
 let(tw,th)=dimensions();
 let(tex,reference,fresh)=if let Some((tex,reference,_))=DIAGRAM{(tex,reference,false)}else{
  
  let tex=sys::ImTextureData_ImTextureData();if tex.is_null(){sys::igImFileClose(file);return}
  // Use Forge's texture API: Recomp marshals host pixels into guest memory.
  // Assigning a guest allocation directly to Pixels bypasses that ownership bridge.
  sys::ImTextureData_Create(tex,sys::ImTextureFormat::RGBA32,tw as i32,th as i32);
  let pixels=sys::ImTextureData_GetPixels(tex) as *mut u8;
  if pixels.is_null(){sys::ImTextureData_destroy(tex);sys::igImFileClose(file);return}
  (tex,sys::ImTextureData_GetTexRef(tex),true)
 };
 let pixels=sys::ImTextureData_GetPixels(tex) as *mut u8;if pixels.is_null(){sys::igImFileClose(file);return}core::ptr::write_bytes(pixels,0,tw*th*4);
 let(dw,dh)=if cfg!(bmi_low_memory){let scale=(tw as f32/w as f32).min(th as f32/h as f32).min(1.);(((w as f32*scale) as usize).max(1),((h as f32*scale) as usize).max(1))}else{(w as usize,h as usize)};
 // Stream rows instead of keeping a second full RGBA image in memory.
 let mut row=alloc::vec::Vec::new();if row.try_reserve_exact(w as usize*4).is_err(){sys::igImFileClose(file);if fresh{sys::ImTextureData_destroy(tex);}return}row.resize(w as usize*4,0u8);
 let mut out_y=0;let mut ok=true;
 for sy in 0..h as usize{
  if sys::igImFileRead(row.as_mut_ptr().cast(),1,row.len() as u64,file)!=row.len() as u64{ok=false;break}
  while out_y<dh&&out_y*h as usize/dh==sy{
   let dst=pixels.add(out_y*tw*4);
   if dw==w as usize{core::ptr::copy_nonoverlapping(row.as_ptr(),dst,dw*4)}else{for x in 0..dw{core::ptr::copy_nonoverlapping(row.as_ptr().add(x*w as usize/dw*4),dst.add(x*4),4);}}
   out_y+=1;
  }
 }
 sys::igImFileClose(file);if !ok{if fresh{sys::ImTextureData_destroy(tex);}return}
 let rect=sys::ImTextureRect{x:0,y:0,w:tw as u16,h:th as u16};(*tex).UsedRect=rect;
 if fresh{sys::igRegisterUserTexture(tex);}else{
  (*tex).UpdateRect=rect;
  sys::ImTextureData_SetStatus(tex,sys::ImTextureStatus::WantUpdates);
 }
 DIAGRAM=Some((tex,reference,id));
}}
pub fn icon(draw:DrawList,id:usize,p1:ImVec2,p2:ImVec2,tint:u32,flip:bool){unsafe{
    sprite(draw,id,p1,p2,tint,flip,false);
}}
pub fn sprite(draw:DrawList,id:usize,p1:ImVec2,p2:ImVec2,tint:u32,flip_x:bool,flip_y:bool){unsafe{
    let Some(reference)=ICONS else{return};let Some(sprite)=g::SPRITES.get(id) else{return};
    let (left,right)=if flip_x{(sprite.x+sprite.w,sprite.x)}else{(sprite.x,sprite.x+sprite.w)};
    let (top,bottom)=if flip_y{(sprite.y+sprite.h,sprite.y)}else{(sprite.y,sprite.y+sprite.h)};
    draw.add_image(reference,p1,p2,ImVec2{x:left as f32/2048.,y:top as f32/g::ATLAS_HEIGHT as f32},ImVec2{x:right as f32/2048.,y:bottom as f32/g::ATLAS_HEIGHT as f32},tint);
}}
pub fn diagram(draw:DrawList,id:u16,p1:ImVec2,p2:ImVec2)->bool{unsafe{
    let Some((tex,reference,loaded))=DIAGRAM else{return false};if loaded!=id||(*tex).Status!=sys::ImTextureStatus::OK{return false}
    let(w,h)=g::DIAGRAM_SOURCES[id as usize];let(tw,th)=dimensions();let scale=if cfg!(bmi_low_memory){(tw as f32/w as f32).min(th as f32/h as f32).min(1.)}else{1.};draw.add_image(reference,p1,p2,ImVec2{x:0.,y:0.},ImVec2{x:(w as f32*scale) as usize as f32/tw as f32,y:(h as f32*scale) as usize as f32/th as f32},0xffffffff);true
}}

use alloc::{vec,vec::Vec,ffi::CString};
use core::ffi::CStr;
use forge::{imgui::DrawList,sys::imgui::{self as sys,ImVec2,ImTextureRef}};
use crate::generated as g;
static mut TEXT:Option<Vec<u8>>=None;
static mut ICONS:Option<ImTextureRef>=None;
static mut DIAGRAM:Option<(*mut sys::ImTextureData,ImTextureRef,u16)>=None;

unsafe fn read_file(path:&CStr)->Option<Vec<u8>>{unsafe{
    let file=sys::igImFileOpen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return None}
    let size=sys::igImFileGetSize(file);if size>16*1024*1024{sys::igImFileClose(file);return None}
    let mut bytes=vec![0;size as usize];let read=sys::igImFileRead(bytes.as_mut_ptr().cast(),1,size,file);sys::igImFileClose(file);
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
fn word(bytes:&[u8],offset:usize)->Option<u32>{Some(u32::from_le_bytes(bytes.get(offset..offset+4)?.try_into().ok()?))}
pub fn init()->bool{unsafe{
    let Some(bytes)=read_file(c"app:/nativeNX/plugins/better-monster-information/monster-text.bin") else{return false};
    if bytes.get(..8)!=Some(b"BMITXT01")||bytes.len()<12{return false}
    let count=word(&bytes,8).unwrap_or(0) as usize;
    if count==0||12+count*36>bytes.len()||core::str::from_utf8(&bytes[12+count*36..]).is_err(){return false}
    for i in 0..count*9{let Some(offset)=word(&bytes,12+i*4) else{return false};if (offset as usize)<12+count*36||offset as usize>=bytes.len(){return false}}
    TEXT=Some(bytes);
    ICONS=texture(c"app:/nativeNX/plugins/better-monster-information/monster-icons.rgba",2048,g::ATLAS_HEIGHT as u16).map(|(_,reference)|reference);
    let ready=core::ptr::addr_of!(ICONS).read().is_some();forge::log::prelude::info!("[monster-info] {} multilingual text records; icon atlas ready={}",count,ready);ready
}}
pub fn text(id:u32,lang:usize)->&'static str{unsafe{
    let Some(bytes)=(&*core::ptr::addr_of!(TEXT)).as_ref() else{return "—"};
    let count=word(bytes,8).unwrap_or(0);if id>=count{return "—"}
    let offset=word(bytes,12+id as usize*36+lang.min(8)*4).unwrap_or(0) as usize;
    let Some(rest)=bytes.get(offset..) else{return "—"};
    let end=rest.iter().position(|&b|b==0).unwrap_or(0);
    core::str::from_utf8(&rest[..end]).unwrap_or("—")
}}
pub static DIAGRAM_MODE:core::sync::atomic::AtomicU32=core::sync::atomic::AtomicU32::new(if cfg!(bmi_low_memory){2}else{0});
pub static DIAGRAM_TRACE:core::sync::atomic::AtomicBool=core::sync::atomic::AtomicBool::new(false);
static mut TRACE:Option<alloc::string::String>=None;
static mut LAST_STATUS:i32=-1;
static UPDATE_ACK:core::sync::atomic::AtomicBool=core::sync::atomic::AtomicBool::new(false);
// Renderer callbacks run after its texture request pass. NVN currently uploads
// WantUpdates but omits SetStatus(OK). Acknowledge only at this real render point,
// with an existing GPU texture, rather than assuming that one elapsed frame did it.
unsafe extern "C" fn acknowledge_update(_list:*const sys::ImDrawList,cmd:*const sys::ImDrawCmd){unsafe{
 let tex=(*cmd).UserCallbackData as *mut sys::ImTextureData;
 if !tex.is_null()&&(*tex).Status==sys::ImTextureStatus::WantUpdates&&(*tex).TexID!=0&&!(*tex).BackendUserData.is_null(){
  sys::ImTextureData_SetStatus(tex,sys::ImTextureStatus::OK);
  UPDATE_ACK.store(true,core::sync::atomic::Ordering::Release);
 }
}}

fn stage(message:&str){
 if !DIAGRAM_TRACE.load(core::sync::atomic::Ordering::Relaxed){return}
 forge::log::prelude::info!("[monster-diagram] {}",message);
 unsafe{let trace=(&mut*core::ptr::addr_of_mut!(TRACE)).get_or_insert_with(alloc::string::String::new);if trace.len()>3500{trace.clear();}trace.push_str(message);trace.push('\n');
 let location=crate::storage::Location{path:c"savedata:/better-monster-information-diagram.log",display:"Monster diagram diagnostics",defaults:""};let _=location.save(trace);}
}
fn dimensions()->(usize,usize){if DIAGRAM_MODE.load(core::sync::atomic::Ordering::Relaxed)==2{(384,256)}else{(g::DIAGRAM_TEX_W,g::DIAGRAM_TEX_H)}}
pub fn load_diagram(id:u16){unsafe{
 if UPDATE_ACK.swap(false,core::sync::atomic::Ordering::AcqRel){stage("render callback acknowledged GPU update; status=OK");}
 if DIAGRAM_MODE.load(core::sync::atomic::Ordering::Relaxed)==1{return}
 if let Some((tex,_,old))=DIAGRAM{
  let status=(*tex).Status as i32;
  if status!=LAST_STATUS{stage(&alloc::format!("backend status={} id={} tex_id={} backend={:p}",status,old,(*tex).TexID,(*tex).BackendUserData));LAST_STATUS=status;}
  if old==id||(*tex).Status!=sys::ImTextureStatus::OK{return}
 }
 let(w,h)=g::DIAGRAM_SOURCES.get(id as usize).copied().unwrap_or((0,0));if w==0{return}
 stage(&alloc::format!("begin id={} source={}x{} mode={}",id,w,h,DIAGRAM_MODE.load(core::sync::atomic::Ordering::Relaxed)));
 let path=CString::new(alloc::format!("app:/nativeNX/plugins/better-monster-information/parts/{}.rgba",id)).unwrap();
 let file=sys::igImFileOpen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){stage("file open failed");return}
 if sys::igImFileGetSize(file)!=w as u64*h as u64*4{sys::igImFileClose(file);stage("file size mismatch");return}
 let(tw,th)=dimensions();
 let(tex,reference,fresh)=if let Some((tex,reference,_))=DIAGRAM{(tex,reference,false)}else{
  stage(&alloc::format!("allocate CPU texture bytes={} canvas={}x{} (source pixels preserved)",tw*th*4,tw,th));
  let tex=sys::ImTextureData_ImTextureData();if tex.is_null(){sys::igImFileClose(file);stage("texture object allocation failed");return}
  // Use Forge's texture API: Recomp marshals host pixels into guest memory.
  // Assigning a guest allocation directly to Pixels bypasses that ownership bridge.
  sys::ImTextureData_Create(tex,sys::ImTextureFormat::RGBA32,tw as i32,th as i32);
  let pixels=sys::ImTextureData_GetPixels(tex) as *mut u8;
  if pixels.is_null(){sys::ImTextureData_destroy(tex);sys::igImFileClose(file);stage("CPU pixels allocation failed");return}
  let rects=sys::igMemAlloc(core::mem::size_of::<sys::ImTextureRect>()) as *mut sys::ImTextureRect;
  if rects.is_null(){sys::ImTextureData_destroy(tex);sys::igImFileClose(file);stage("update rectangle allocation failed");return}
  (*tex).Updates.Data=rects;(*tex).Updates.Capacity=1;(*tex).Updates.Size=0;
  sys::ImTextureData_SetStatus(tex,sys::ImTextureStatus::WantCreate);
  (tex,sys::ImTextureData_GetTexRef(tex),true)
 };
 let pixels=sys::ImTextureData_GetPixels(tex) as *mut u8;if pixels.is_null(){sys::igImFileClose(file);stage("CPU pixel mapping failed");return}core::ptr::write_bytes(pixels,0,tw*th*4);
 let(dw,dh)=if DIAGRAM_MODE.load(core::sync::atomic::Ordering::Relaxed)==2{let scale=(tw as f32/w as f32).min(th as f32/h as f32).min(1.);(((w as f32*scale) as usize).max(1),((h as f32*scale) as usize).max(1))}else{(w as usize,h as usize)};
 // Stream rows instead of keeping a second full RGBA image in memory.
 let mut row=alloc::vec::Vec::new();if row.try_reserve_exact(w as usize*4).is_err(){sys::igImFileClose(file);if fresh{sys::ImTextureData_destroy(tex);}stage("row allocation failed");return}row.resize(w as usize*4,0u8);
 let mut out_y=0;let mut ok=true;
 for sy in 0..h as usize{
  if sys::igImFileRead(row.as_mut_ptr().cast(),1,row.len() as u64,file)!=row.len() as u64{ok=false;break}
  while out_y<dh&&out_y*h as usize/dh==sy{
   let dst=pixels.add(out_y*tw*4);
   if dw==w as usize{core::ptr::copy_nonoverlapping(row.as_ptr(),dst,dw*4)}else{for x in 0..dw{core::ptr::copy_nonoverlapping(row.as_ptr().add(x*w as usize/dw*4),dst.add(x*4),4);}}
   out_y+=1;
  }
 }
 sys::igImFileClose(file);if !ok{if fresh{sys::ImTextureData_destroy(tex);}stage("image read failed");return}
 let rect=sys::ImTextureRect{x:0,y:0,w:tw as u16,h:th as u16};(*tex).UsedRect=rect;
 if fresh{stage("register texture for GPU creation");sys::igRegisterUserTexture(tex);}else{
  // NVN uploads each entry in Updates; UpdateRect alone is only a flush bound.
  (*tex).Updates.Data.write(rect);(*tex).Updates.Size=1;(*tex).UpdateRect=rect;
  sys::ImTextureData_SetStatus(tex,sys::ImTextureStatus::WantUpdates);
  let draw=DrawList::foreground();if !draw.raw().is_null(){sys::ImDrawList_AddCallback(draw.raw(),Some(acknowledge_update),tex.cast(),0);}
  stage("queued full update rectangle and render acknowledgement");
 }
 DIAGRAM=Some((tex,reference,id));LAST_STATUS=-1;stage("CPU load complete; waiting for backend");
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
    let(w,h)=g::DIAGRAM_SOURCES[id as usize];let(tw,th)=dimensions();let scale=if DIAGRAM_MODE.load(core::sync::atomic::Ordering::Relaxed)==2{(tw as f32/w as f32).min(th as f32/h as f32).min(1.)}else{1.};draw.add_image(reference,p1,p2,ImVec2{x:0.,y:0.},ImVec2{x:(w as f32*scale) as usize as f32/tw as f32,y:(h as f32*scale) as usize as f32/th as f32},0xffffffff);true
}}

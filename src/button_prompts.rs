//! Recomp menu bindings affect artwork only; input remains the engine menu word.
use alloc::string::{String,ToString};
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Prompt{Original,Position(u8),Text(String)}
pub fn parse(ini:&str)->[Prompt;6]{
 let mut out=core::array::from_fn(|_|Prompt::Original);
 let mut section="";let mut enabled=true;let mut menus=true;
 for line in ini.lines(){let line=line.split(';').next().unwrap_or("").trim();
  if line.starts_with('[')&&line.ends_with(']'){section=&line[1..line.len()-1];continue}
  let Some((key,value))=line.split_once('=')else{continue};let key=key.trim();let value=value.trim();
  if section.eq_ignore_ascii_case("Actions"){
   let on=!(value.eq_ignore_ascii_case("off")||value.eq_ignore_ascii_case("false")||value=="0");
   if key.eq_ignore_ascii_case("Enable"){enabled=on}if key.eq_ignore_ascii_case("ManageMenus"){menus=on}
  }
  if !section.eq_ignore_ascii_case("Actions.Pad"){continue}
  let Some(i)=["MenuConfirm","MenuCancel","MenuX","MenuPageL","MenuPageR","MenuY"].iter().position(|x|key.eq_ignore_ascii_case(x)||(*x=="MenuConfirm"&&key.eq_ignore_ascii_case("Confirm"))||(*x=="MenuCancel"&&key.eq_ignore_ascii_case("Cancel")))else{continue};
  out[i]=binding(value);
 }
 if !enabled||!menus{return core::array::from_fn(|_|Prompt::Original)}out
}
fn binding(value:&str)->Prompt{
 if value.is_empty()||value.eq_ignore_ascii_case("default"){return Prompt::Original}
 if value.eq_ignore_ascii_case("none"){return Prompt::Text("—".to_string())}
 // The first alternative is sufficient to explain one valid way to activate it.
 let first=value.split('|').next().unwrap_or("").trim();
 let position=match first.to_ascii_lowercase().as_str(){"pad:south"=>Some(0),"pad:east"=>Some(1),"pad:north"=>Some(2),"pad:west"=>Some(3),"pad:lefttrigger"|"pad:ltrigger"|"pad:lt"=>Some(4),"pad:righttrigger"|"pad:rtrigger"|"pad:rt"=>Some(5),_=>None};
 if let Some(p)=position{return Prompt::Position(p)}
 let mut label=String::new();for part in first.split('+'){
  if !label.is_empty(){label.push('+')}
  let token=part.trim().strip_prefix("pad:").unwrap_or(part.trim());
  label.push_str(match token{"south"=>"↓","east"=>"→","north"=>"↑","west"=>"←","leftshoulder"|"lshoulder"=>"L","rightshoulder"|"rshoulder"=>"R","lefttrigger"|"ltrigger"|"lt"=>"ZL","righttrigger"|"rtrigger"|"rt"=>"ZR","back"=>"−","start"=>"+",_=>token});
 }Prompt::Text(label)
}
pub fn icon_slot(position:u8,_layout:usize)->usize{
 // All three art rows preserve Nintendo's physical positions (Xbox B / PS
 // Circle occupy the east/accept cell), rather than swapping by letter.
 match position{0=>1,1=>0,2=>2,3=>5,4=>3,5=>4,_=>0}
}
pub fn recomp_imports(a:usize,b:usize)->bool{
 [a,b].iter().all(|&p|(0x03000000..0x03100000).contains(&p)&&p&15==0)
}
#[cfg(target_arch="arm")]
mod runtime{
 use super::*;
 use forge::sys::imgui as sys;
 static mut PROMPTS:Option<[Prompt;6]>=None;
 static mut WAIT:f32=0.;
 unsafe extern "C"{fn forge_graphics_isMenuVisible()->bool;}
 pub fn refresh(dt:f32){unsafe{
  // Recomp resolves both imports directly to its reserved HLE address range.
  // Fail closed on hardware/emulators: never send a PC path to nn::fs.
  if !recomp_imports(forge_graphics_isMenuVisible as *const () as usize,sys::igImFileOpen as *const () as usize){return}
  WAIT-=dt.max(0.);if WAIT>0.{return}WAIT=1.;
  let file=sys::igImFileOpen(c"mhgurecomp.ini".as_ptr(),c"rb".as_ptr());if file.is_null(){return}
  let size=sys::igImFileGetSize(file);
  if size>128*1024{sys::igImFileClose(file);return}
  let mut bytes=alloc::vec![0u8;size as usize];let n=sys::igImFileRead(bytes.as_mut_ptr().cast(),1,size,file);sys::igImFileClose(file);
  if n==size{if let Ok(text)=core::str::from_utf8(&bytes){let next=parse(text);if (&*core::ptr::addr_of!(PROMPTS)).as_ref()!=Some(&next){crate::diagnostic_info!("[monster-info] Recomp controller prompt bindings refreshed");PROMPTS=Some(next);}}}
 }}
 pub fn get(index:usize)->Prompt{unsafe{(&*core::ptr::addr_of!(PROMPTS)).as_ref().map(|p|p[index].clone()).unwrap_or(Prompt::Original)}}
}
#[cfg(target_arch="arm")]pub use runtime::{refresh,get};
#[cfg(test)]mod tests{
 use super::*;use alloc::format;
 #[test]fn swapped_and_all_menu_prompts(){let p=parse("[Actions.Pad]\nMenuConfirm=pad:south\nMenuCancel=pad:east\nMenuX=pad:west\nMenuY=pad:north\nMenuPageL=pad:righttrigger\nMenuPageR=pad:lefttrigger");assert_eq!(p,[Prompt::Position(0),Prompt::Position(1),Prompt::Position(3),Prompt::Position(5),Prompt::Position(4),Prompt::Position(2)]);assert_eq!(icon_slot(0,0),1);assert_eq!(icon_slot(1,0),0);assert_eq!(icon_slot(0,1),1);assert_eq!(icon_slot(1,2),0);}
 #[test]fn host_files_are_disabled_for_native_import_addresses(){assert!(recomp_imports(0x03000100,0x03000200));for a in [0,0x02400100,0x03200100,0x71000000,0x03000101]{assert!(!recomp_imports(a,0x03000200));assert!(!recomp_imports(0x03000100,a));}}
 #[test]fn disabled_unbound_chords_and_other_sections(){for key in ["Enable","ManageMenus"]{assert_eq!(parse(&format!("[Actions.Pad]\nMenuConfirm=pad:south\n[Actions]\n{key}=Off")),core::array::from_fn(|_|Prompt::Original));}let p=parse("[Actions.Kbm]\nMenuConfirm=key:e\n[Actions.Pad]\nMenuCancel=None\nMenuX=pad:leftshoulder+pad:north | pad:west\nMenuY=Default");assert_eq!(p[0],Prompt::Original);assert_eq!(p[1],Prompt::Text("—".into()));assert_eq!(p[2],Prompt::Text("L+↑".into()));assert_eq!(p[5],Prompt::Original);}
}

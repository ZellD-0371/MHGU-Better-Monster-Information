//! Recomp bindings affect artwork only; input remains the engine menu word.
//! Pad prompts follow [Actions.Pad]. While the last input came from the keyboard or mouse (Recomp only) and
//! [Controls] KeyboardPrompts is on, prompts show keys instead. Our input is the menu word, so the keys are Recomp's
//! menu actions ([Actions.Kbm]) when actions manage menus, else the plain [Keybinds] rows.
use alloc::{string::{String,ToString},vec::Vec};
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Prompt{
 Original,
 /// Another pad button by position (0 south, 1 east, 2 north, 3 west, 4 left trigger, 5 right trigger)
 Position(u8),
 /// A pad input without a picture (combo, stick click, ...): the first alternative, labelled per layout at draw time
 Pad(String),
 /// Fixed text ("—" = unbound)
 Text(String),
 /// A keyboard key or mouse button by its [Keybinds] name ("Space", "Mouse Left")
 Key(String),
}
/// Prompt actions in PROMPTS order: accept, back, rank, previous, next, form
pub const MENU_ACTIONS:[&str;6]=["MenuConfirm","MenuCancel","MenuX","MenuPageL","MenuPageR","MenuY"];
/// [Keybinds] row of each prompt action
pub const ROWS:[&str;6]=["A","B","X","ZL","ZR","Y"];
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Config{
 pub pad:[Prompt;6],
 /// [Keybinds] keys per action, as the file writes them
 pub rows:[Vec<String>;6],
 /// [Actions.Kbm] menu actions the player changed; None = default
 pub menu_keys:[Option<Vec<String>>;6],
 /// [Actions] Enable and ManageMenus are both on
 pub menus:bool,
 /// [Controls] KeyboardPrompts (on when missing)
 pub prompts:bool,
}
fn is_off(value:&str)->bool{value.eq_ignore_ascii_case("off")||value.eq_ignore_ascii_case("false")||value=="0"}
fn is_default(value:&str)->bool{value.is_empty()||value.eq_ignore_ascii_case("default")||value.eq_ignore_ascii_case("original")}
fn action_index(key:&str)->Option<usize>{
 MENU_ACTIONS.iter().position(|x|key.eq_ignore_ascii_case(x)||(*x=="MenuConfirm"&&key.eq_ignore_ascii_case("Confirm"))||(*x=="MenuCancel"&&key.eq_ignore_ascii_case("Cancel")))
}
pub fn parse(ini:&str)->Config{
 let mut pad:[Prompt;6]=core::array::from_fn(|_|Prompt::Original);
 let mut rows:[Vec<String>;6]=core::array::from_fn(|_|Vec::new());
 let mut menu_keys:[Option<Vec<String>>;6]=core::array::from_fn(|_|None);
 let(mut enabled,mut menus,mut prompts)=(true,true,true);
 let mut section="";
 for line in ini.lines(){let line=line.split(';').next().unwrap_or("").trim();
  if line.starts_with('[')&&line.ends_with(']'){section=&line[1..line.len()-1];continue}
  let Some((key,value))=line.split_once('=')else{continue};let key=key.trim();let value=value.trim();
  if section.eq_ignore_ascii_case("Actions"){
   if key.eq_ignore_ascii_case("Enable"){enabled=!is_off(value)}if key.eq_ignore_ascii_case("ManageMenus"){menus=!is_off(value)}
  }else if section.eq_ignore_ascii_case("Controls"){
   if key.eq_ignore_ascii_case("KeyboardPrompts"){prompts=!is_off(value)}
  }else if section.eq_ignore_ascii_case("Keybinds"){
   if let Some(i)=ROWS.iter().position(|r|key.eq_ignore_ascii_case(r)){
    rows[i]=value.split(',').map(str::trim).filter(|k|!k.is_empty()&&!k.eq_ignore_ascii_case("none")).map(String::from).collect();
   }
  }else if section.eq_ignore_ascii_case("Actions.Kbm"){
   if let Some(i)=action_index(key){if !is_default(value){menu_keys[i]=Some(action_keys(value));}}
  }else if section.eq_ignore_ascii_case("Actions.Pad"){
   if let Some(i)=action_index(key){pad[i]=binding(value);}
  }
 }
 let menus=enabled&&menus;
 if !menus{pad=core::array::from_fn(|_|Prompt::Original)}
 Config{pad,rows,menu_keys,menus,prompts}
}
/// Inputs of an [Actions.Kbm] line ("key:Space | mouse:Left") as [Keybinds]-style names ("Space", "Mouse Left").
/// Combos ('+') and pad inputs are skipped; "Keypad +" itself is a key, not a combo.
fn action_keys(value:&str)->Vec<String>{
 value.split('|').map(str::trim).filter(|v|!v.contains('+')||v.ends_with('+')).filter_map(|v|{
  let lower=v.to_ascii_lowercase();
  if lower.starts_with("key:"){Some(String::from(v[4..].trim()))}
  else if lower.starts_with("mouse:"){Some(alloc::format!("Mouse {}",v[6..].trim()))}
  else{None}
 }).filter(|k|!k.is_empty()).collect()
}
/// Keys of an action in Recomp's menus: the changed menu action, else the menu default (confirm = left click,
/// cancel = right click, X = Space, the rest as [Keybinds]); without menu actions, the [Keybinds] row
pub fn keys(c:&Config,i:usize)->Vec<String>{
 if !c.menus{return c.rows[i].clone()}
 if let Some(keys)=&c.menu_keys[i]{return keys.clone()}
 match i{0=>alloc::vec![String::from("Mouse Left")],1=>alloc::vec![String::from("Mouse Right")],2=>alloc::vec![String::from("Space")],_=>c.rows[i].clone()}
}
/// The prompt of an action for a keyboard / mouse player: its first key, or "—" when nothing is bound.
/// We never take the pointer, so mouse bindings keep working through Recomp's menu mouse and are shown as is.
pub fn kbm_prompt(c:&Config,i:usize)->Prompt{
 match keys(c,i).into_iter().next(){Some(k)=>Prompt::Key(k),None=>Prompt::Text("—".to_string())}
}
fn binding(value:&str)->Prompt{
 if is_default(value){return Prompt::Original}
 if value.eq_ignore_ascii_case("none"){return Prompt::Text("—".to_string())}
 // The first alternative is sufficient to explain one valid way to activate it.
 let first=value.split('|').next().unwrap_or("").trim();
 let position=match first.to_ascii_lowercase().as_str(){"pad:south"=>Some(0),"pad:east"=>Some(1),"pad:north"=>Some(2),"pad:west"=>Some(3),"pad:lefttrigger"|"pad:ltrigger"|"pad:lt"=>Some(4),"pad:righttrigger"|"pad:rtrigger"|"pad:rt"=>Some(5),_=>None};
 if let Some(p)=position{return Prompt::Position(p)}
 Prompt::Pad(first.to_string())
}
/// Text for a pad input without a picture, using the names of the chosen layout (0 Nintendo, 1 Xbox, 2 PlayStation).
/// Face buttons are positions in Recomp's names, so they stay arrows on every layout.
pub fn pad_label(raw:&str,layout:usize)->String{
 let mut label=String::new();
 for part in raw.split('+'){
  if !label.is_empty(){label.push('+')}
  let part=part.trim();let lower=part.to_ascii_lowercase();
  let token=lower.strip_prefix("pad:").or_else(|| lower.strip_prefix("sw:")).unwrap_or(&lower);
  let names=|n:&'static str,x:&'static str,p:&'static str|match layout{1=>x,2=>p,_=>n};
  let name=match token{
   "south"=>"↓","east"=>"→","north"=>"↑","west"=>"←",
   "leftshoulder"|"lshoulder"|"lb"=>names("L","LB","L1"),"rightshoulder"|"rshoulder"|"rb"=>names("R","RB","R1"),
   "lefttrigger"|"ltrigger"|"lt"=>names("ZL","LT","L2"),"righttrigger"|"rtrigger"|"rt"=>names("ZR","RT","R2"),
   "leftstick"|"lstick"|"ls"=>names("L3","LS","L3"),"rightstick"|"rstick"|"rs"=>names("R3","RS","R3"),
   "dpup"|"dpadup"=>"D↑","dpdown"|"dpaddown"=>"D↓","dpleft"|"dpadleft"=>"D←","dpright"|"dpadright"=>"D→",
   "back"=>"−","start"=>"+",
   _=>{label.push_str(part.strip_prefix("pad:").or_else(|| part.strip_prefix("sw:")).unwrap_or(part));continue}
  };
  label.push_str(name);
 }
 label
}
/// Picture name of a key ("Left Shift" -> "left-shift", "-" -> "minus"): the [Keybinds] name in lower case, spaces as
/// '-', punctuation as words. Names with two spellings map to one picture. Pictures: tools/prepare_kbm_icons.py
pub fn key_sprite(key:&str)->String{
 let key=key.trim();
 let word=match key{"-"=>Some("minus"),"="=>Some("equals"),"["=>Some("bracket-open"),"]"=>Some("bracket-close"),";"=>Some("semicolon"),"'"=>Some("apostrophe"),","=>Some("comma"),"."=>Some("period"),"/"=>Some("slash"),"\\"=>Some("backslash"),"`"=>Some("grave"),"Keypad +"=>Some("keypad-plus"),"Keypad *"=>Some("keypad-asterisk"),_=>None};
 let mut s=String::new();
 match word{Some(w)=>s.push_str(w),None=>for ch in key.chars(){s.push(if ch==' '{'-'}else{ch.to_ascii_lowercase()})}}
 match s.as_str(){"caps-lock"=>"capslock".into(),"page-up"=>"pageup".into(),"page-down"=>"pagedown".into(),"enter"=>"return".into(),"scroll-lock"=>"scrolllock".into(),"print-screen"=>"printscreen".into(),"num-lock"=>"numlock".into(),"right-gui"=>"left-gui".into(),_=>s}
}
/// Short text for a key without a picture (about 3 letters at most)
pub fn key_label(key:&str)->String{
 let lower=key.trim().to_ascii_lowercase();
 let short=match lower.as_str(){
  "up"=>"↑","down"=>"↓","left"=>"←","right"=>"→","space"=>"SPC","return"|"enter"=>"ENT","keypad enter"=>"NEN","backspace"=>"BS","escape"=>"ESC",
  "left shift"|"right shift"=>"SFT","left ctrl"|"right ctrl"=>"CTL","left alt"|"right alt"=>"ALT","left gui"|"right gui"=>"WIN",
  "pageup"|"page up"=>"PGU","pagedown"|"page down"=>"PGD","delete"=>"DEL","insert"=>"INS","capslock"|"caps lock"=>"CAP",
  "mouse left"=>"LMB","mouse right"=>"RMB","mouse middle"=>"MMB","mouse 4"=>"M4","mouse 5"=>"M5","wheel up"=>"W↑","wheel down"=>"W↓",
  _=>{
   let t=key.trim();
   // Keypad keys get an N prefix so "Keypad 1" does not read as "1"
   if let Some(rest)=t.strip_prefix("Keypad "){let rest:String=rest.chars().filter(|c|*c!=' ').collect();return alloc::format!("N{}",rest.to_uppercase()).chars().take(3).collect()}
   let t:String=t.chars().filter(|c|*c!=' ').collect();return t.to_uppercase().chars().take(3).collect();
  }
 };
 short.to_string()
}
pub fn recomp_imports(a:usize,b:usize)->bool{
 [a,b].iter().all(|&p|(0x03000000..0x03100000).contains(&p)&&p&15==0)
}
/// Last input device. Keys reach us as menu words too, so a fresh menu press counts as the controller only when no
/// key, click, wheel notch or pointer move came within 250 ms before it.
/// Keyboard / mouse input of one frame (Recomp only)
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Input{
 /// Wheel notch as model scroll events (Wheel Up / Wheel Down, same as the right stick), or 0
 pub wheel:u32,
 /// Pointer in console pixels (1280x720 over the game's picture); None when captured, off the picture or in Forge's menu
 pub mouse:Option<(f32,f32)>,
 /// Mouse buttons held / newly pressed (1 left, 2 middle, 4 right)
 pub buttons:u32,pub pressed:u32,
}
impl Input{pub const NONE:Self=Self{wheel:0,mouse:None,buttons:0,pressed:0};}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Device{pub kbm:bool,key_age:f32}
impl Device{
 pub const NEW:Self=Self{kbm:false,key_age:1.};
 /// One frame: kbm = keyboard / mouse input this frame, pad = a new menu press since the last frame
 pub fn step(&mut self,dt:f32,kbm:bool,pad:bool)->bool{
  let old=self.kbm;self.key_age=(self.key_age+dt.max(0.)).min(10.);
  if kbm{self.key_age=0.;self.kbm=true}else if pad&&self.key_age>0.25{self.kbm=false}
  old!=self.kbm
 }
}
#[cfg(target_arch="arm")]
mod runtime{
 use super::*;
 use core::sync::atomic::{AtomicBool,AtomicU32,Ordering};
 use forge::sys::imgui as sys;
 static mut CONFIG:Option<Config>=None;
 static mut WAIT:f32=0.;
 static mut DEVICE:Device=Device::NEW;
 static KBM:AtomicBool=AtomicBool::new(false);
 static PAD_PRESS:AtomicBool=AtomicBool::new(false);
 static LAST_X:AtomicU32=AtomicU32::new(0);static LAST_Y:AtomicU32=AtomicU32::new(0);static LAST_BUTTONS:AtomicU32=AtomicU32::new(0);
 unsafe extern "C"{
  fn forge_graphics_isMenuVisible()->bool;
  // Recomp's own Forge extensions. Forge on hardware/emulators binds missing imports to stubs, so these are only
  // called after is_recomp() (older Recomp builds make them return 0).
  fn forge_input_getKeyPressed(out:*mut u8,size:u32)->bool;
  fn forge_input_getMouse(x:*mut f32,y:*mut f32,buttons:*mut u32)->bool;
  fn forge_input_setMouseFree(free:bool);
 }
 pub fn is_recomp()->bool{recomp_imports(forge_graphics_isMenuVisible as *const () as usize,sys::igImFileOpen as *const () as usize)}
 pub fn refresh(dt:f32){unsafe{
  // Recomp resolves both imports directly to its reserved HLE address range.
  // Fail closed on hardware/emulators: never send a PC path to nn::fs.
  if !is_recomp(){return}
  WAIT-=dt.max(0.);if WAIT>0.{return}WAIT=1.;
  let file=sys::igImFileOpen(c"mhgurecomp.ini".as_ptr(),c"rb".as_ptr());if file.is_null(){return}
  let size=sys::igImFileGetSize(file);
  if size>128*1024{sys::igImFileClose(file);return}
  let mut bytes=alloc::vec![0u8;size as usize];let n=sys::igImFileRead(bytes.as_mut_ptr().cast(),1,size,file);sys::igImFileClose(file);
  if n==size{if let Ok(text)=core::str::from_utf8(&bytes){let next=parse(text);if (&*core::ptr::addr_of!(CONFIG)).as_ref()!=Some(&next){crate::diagnostic_info!("[monster-info] Recomp prompt bindings refreshed: menus={} keyboard_prompts={} keys={:?}",next.menus,next.prompts,(0..6).map(|i|keys(&next,i).into_iter().next()).collect::<Vec<_>>());CONFIG=Some(next);}}}
 }}
 /// Menu translation hook (game thread): a menu button was newly pressed
 pub fn note_menu_press(){PAD_PRESS.store(true,Ordering::Release);}
 /// Every frame on the game thread: update the last input device and read the wheel and the pointer.
 /// The pointer is never taken (no setMouseFree), so Recomp's menu mouse keeps working.
 pub fn poll_input(dt:f32)->Input{unsafe{
  let pad=PAD_PRESS.swap(false,Ordering::AcqRel);
  if !is_recomp(){return Input::NONE}
  let mut name=[0u8;32];let mut kbm=false;let mut wheel=0;
  if forge_input_getKeyPressed(name.as_mut_ptr(),name.len() as u32){
   kbm=true;let n=name.iter().position(|&c|c==0).unwrap_or(name.len());
   wheel=match &name[..n]{b"Wheel Up"=>crate::model::SCROLL_UP,b"Wheel Down"=>crate::model::SCROLL_DOWN,_=>0};
  }
  let(mut x,mut y,mut buttons)=(0f32,0f32,0u32);let(mut mouse,mut held,mut pressed)=(None,0,0);
  if forge_input_getMouse(&mut x,&mut y,&mut buttons){
   let(px,py)=(f32::from_bits(LAST_X.swap(x.to_bits(),Ordering::Relaxed)),f32::from_bits(LAST_Y.swap(y.to_bits(),Ordering::Relaxed)));
   let new=buttons&!LAST_BUTTONS.swap(buttons,Ordering::Relaxed);
   if (px-x).abs()+(py-y).abs()>3.||new!=0{kbm=true}
   mouse=Some((x,y));held=buttons;pressed=new;
  }else{LAST_BUTTONS.store(0,Ordering::Relaxed);}
  let device=&mut*core::ptr::addr_of_mut!(DEVICE);
  if device.step(dt,kbm,pad){crate::diagnostic_info!("[monster-info] prompt device: {}",if device.kbm{"keyboard/mouse"}else{"controller"});}
  KBM.store(device.kbm,Ordering::Release);
  Input{wheel,mouse,buttons:held,pressed}
 }}
 static FREE:AtomicBool=AtomicBool::new(false);
 /// Take Recomp's pointer for our page (game thread, only on change): its menu mouse, wheel and clicks stop reaching
 /// the game. Only while the page is modal, where the game gets no input anyway.
 pub fn set_mouse_free(free:bool){
  if !is_recomp()||FREE.swap(free,Ordering::AcqRel)==free{return}
  unsafe{forge_input_setMouseFree(free)};crate::diagnostic_info!("[monster-info] pointer {}",if free{"taken by the page"}else{"returned to Recomp menus"});
 }
 pub fn mouse_free()->bool{FREE.load(Ordering::Acquire)}
 /// The last input came from the keyboard / mouse
 pub fn kbm_active()->bool{KBM.load(Ordering::Acquire)}
 /// The pointer now (game thread, Recomp only): position (console pixels) and held buttons, or None
 pub fn mouse_now()->Option<(f32,f32,u32)>{unsafe{
  if !is_recomp(){return None}
  let(mut x,mut y,mut buttons)=(0f32,0f32,0u32);
  forge_input_getMouse(&mut x,&mut y,&mut buttons).then_some((x,y,buttons))
 }}
 fn config()->Option<&'static Config>{unsafe{(&*core::ptr::addr_of!(CONFIG)).as_ref()}}
 pub fn get(index:usize)->Prompt{
  let Some(c)=config()else{return Prompt::Original};
  if KBM.load(Ordering::Acquire)&&c.prompts{kbm_prompt(c,index)}else{c.pad[index].clone()}
 }
}
#[cfg(target_arch="arm")]pub use runtime::{refresh,get,is_recomp,poll_input,note_menu_press,kbm_active,mouse_now,set_mouse_free,mouse_free};
#[cfg(test)]mod tests{
 use super::*;use alloc::format;
 fn pads(p:&Config)->[Prompt;6]{p.pad.clone()}
 #[test]fn swapped_and_all_menu_prompts(){let p=parse("[Actions.Pad]\nMenuConfirm=pad:south\nMenuCancel=pad:east\nMenuX=pad:west\nMenuY=pad:north\nMenuPageL=pad:righttrigger\nMenuPageR=pad:lefttrigger");assert_eq!(pads(&p),[Prompt::Position(0),Prompt::Position(1),Prompt::Position(3),Prompt::Position(5),Prompt::Position(4),Prompt::Position(2)]);assert_eq!(icon_slot(0,0),1);assert_eq!(icon_slot(1,0),0);assert_eq!(icon_slot(0,1),1);assert_eq!(icon_slot(1,2),0);}
 #[test]fn host_files_are_disabled_for_native_import_addresses(){assert!(recomp_imports(0x03000100,0x03000200));for a in [0,0x02400100,0x03200100,0x71000000,0x03000101]{assert!(!recomp_imports(a,0x03000200));assert!(!recomp_imports(0x03000100,a));}}
 #[test]fn disabled_unbound_chords_and_other_sections(){for key in ["Enable","ManageMenus"]{assert_eq!(pads(&parse(&format!("[Actions.Pad]\nMenuConfirm=pad:south\n[Actions]\n{key}=Off"))),core::array::from_fn(|_|Prompt::Original));}let p=parse("[Actions.Kbm]\nMenuConfirm=key:e\n[Actions.Pad]\nMenuCancel=None\nMenuX=pad:leftshoulder+pad:north | pad:west\nMenuY=Default");assert_eq!(p.pad[0],Prompt::Original);assert_eq!(p.pad[1],Prompt::Text("—".into()));assert_eq!(p.pad[2],Prompt::Pad("pad:leftshoulder+pad:north".into()));assert_eq!(pad_label("pad:leftshoulder+pad:north",0),"L+↑");assert_eq!(p.pad[5],Prompt::Original);}
 #[test]fn pad_text_uses_layout_names(){
  let raw="pad:leftshoulder+pad:rightshoulder+pad:lefttrigger+pad:righttrigger+pad:leftstick+pad:rstick";
  assert_eq!(pad_label(raw,0),"L+R+ZL+ZR+L3+R3");assert_eq!(pad_label(raw,1),"LB+RB+LT+RT+LS+RS");assert_eq!(pad_label(raw,2),"L1+R1+L2+R2+L3+R3");
  assert_eq!(pad_label("pad:dpup+pad:back",1),"D↑+−");assert_eq!(pad_label("pad:mystery",2),"mystery");
 }
 // The user's Recomp writes no [Actions.Kbm] section until a menu key is changed.
 const USER:&str="[Controls]\nKeyboardPrompts = On\n[Keybinds]\nA = Mouse Right\nB = Space\nX = Mouse Left\nY = R\nZL = Tab\nZR = T\n[Actions]\nEnable=1\nManageMenus=1\n";
 #[test]fn menu_defaults_follow_menu_actions_not_keybinds(){
  let c=parse(USER);assert!(c.menus&&c.prompts);
  let first=|i|kbm_prompt(&c,i);
  assert_eq!(first(0),Prompt::Key("Mouse Left".into()));assert_eq!(first(1),Prompt::Key("Mouse Right".into()));assert_eq!(first(2),Prompt::Key("Space".into()));
  assert_eq!(first(3),Prompt::Key("Tab".into()));assert_eq!(first(4),Prompt::Key("T".into()));assert_eq!(first(5),Prompt::Key("R".into()));
 }
 #[test]fn changed_menu_actions_take_the_first_plain_key(){
  let c=parse(&format!("{USER}[Actions.Kbm]\nMenuX=key:Left Shift+key:F | key:G | mouse:Middle\nMenuConfirm = mouse:Left | key:Return\nMenuCancel=None\nMenuY=pad:north | key:Keypad +\nMenuPageL=Default\n"));
  assert_eq!(keys(&c,2),["G","Mouse Middle"]);assert_eq!(kbm_prompt(&c,2),Prompt::Key("G".into()));
  assert_eq!(kbm_prompt(&c,0),Prompt::Key("Mouse Left".into()));
  assert_eq!(kbm_prompt(&c,1),Prompt::Text("—".into()));
  assert_eq!(kbm_prompt(&c,5),Prompt::Key("Keypad +".into()));
  assert_eq!(kbm_prompt(&c,3),Prompt::Key("Tab".into()));
 }
 #[test]fn without_menu_actions_keybinds_rows_are_used(){
  for off in ["[Actions]\nManageMenus = Off\n","[Actions]\nEnable=0\n"]{
   let c=parse(&format!("{USER}[Actions.Kbm]\nMenuX=key:G\n{off}"));assert!(!c.menus);
   assert_eq!(kbm_prompt(&c,0),Prompt::Key("Mouse Right".into()));assert_eq!(kbm_prompt(&c,1),Prompt::Key("Space".into()));assert_eq!(kbm_prompt(&c,2),Prompt::Key("Mouse Left".into()));
  }
  let c=parse("[Actions]\nManageMenus=Off\n[Keybinds]\nA = None\nB = Escape, Backspace\n");assert_eq!(kbm_prompt(&c,0),Prompt::Text("—".into()));assert_eq!(kbm_prompt(&c,1),Prompt::Key("Escape".into()));
 }
 #[test]fn keyboard_prompts_switch(){assert!(parse("").prompts);assert!(!parse("[Controls]\nKeyboardPrompts = Off").prompts);assert!(parse("[Controls]\nKeyboardPrompts = On").prompts);}
 #[test]fn key_pictures_and_labels(){
  for(key,sprite)in [("Left Shift","left-shift"),("Mouse Left","mouse-left"),("-","minus"),("Keypad +","keypad-plus"),("CapsLock","capslock"),("Caps Lock","capslock"),("PageUp","pageup"),("Page Up","pageup"),("Return","return"),("Enter","return"),("Wheel Up","wheel-up"),("Mouse 4","mouse-4"),("\\","backslash"),("A","a"),("F12","f12")]{assert_eq!(key_sprite(key),sprite,"{key}");}
  for(key,label)in [("Space","SPC"),("Return","ENT"),("PageUp","PGU"),("Mouse Middle","MMB"),("Up","↑"),("Keypad 1","N1"),("Keypad /","N/"),("Application","APP"),("Mouse 5","M5"),("Wheel Down","W↓")]{assert_eq!(key_label(key),label,"{key}");}
 }
 #[test]fn device_follows_the_last_input(){
  let mut d=Device::NEW;assert!(!d.kbm);
  assert!(d.step(0.016,true,true));assert!(d.kbm);
  // The key's own menu word arrives a frame later: still the keyboard
  assert!(!d.step(0.016,false,true));assert!(d.kbm);
  for _ in 0..20{d.step(0.016,false,false);}
  assert!(d.step(0.016,false,true));assert!(!d.kbm);
  assert!(!d.step(0.016,false,true));
 }
}
pub fn icon_slot(position:u8,_layout:usize)->usize{
 // All three art rows preserve Nintendo's physical positions (Xbox B / PS
 // Circle occupy the east/accept cell), rather than swapping by letter.
 match position{0=>1,1=>0,2=>2,3=>5,4=>3,5=>4,_=>0}
}

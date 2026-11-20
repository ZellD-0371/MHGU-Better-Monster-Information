//! Recomp mouse clicks on our page. We never take the pointer (no setMouseFree): Recomp's menu mouse keeps working on
//! the rest of the hunter's notes, and a left click on one of our targets is ours (its MenuConfirm is withheld).
//! While our page is modal (a row selected, a list open) we do take it (setMouseFree): Recomp's menu mouse applies the
//! wheel straight to the native monster list, which no menu word filter can stop. Its clicks then come only to us, so
//! a right click is our back.
use crate::model::{Target,View};
/// A Recomp mouse position (console pixels, 1280x720) in display pixels. Recomp maps the pointer over the game's
/// picture, not the host window, and its x over the HUD's width only when the HUD is narrower than the picture
/// (16:9 HUD on 21:9). Same mapping as Wishlist hud_layout::mouse and Skill-Simulator picture.rs Area::mouse.
pub fn to_display(x:f32,y:f32,v:View)->(f32,f32){
 let whole=(x*v.w/1280.,y*v.h/720.);
 let Some((rw,rh,uw))=v.hud else{return whole};
 if !(64..=16384).contains(&rw)||!(64..=16384).contains(&rh){return whole}
 let aspect=rw as f32/rh as f32;let ph=v.h.min(v.w/aspect);let pw=ph*aspect;
 let span=uw as f32/720.;let canvas=if (1280..=16384).contains(&uw)&&span<aspect{span/aspect}else{1.};
 let u=x/1280.*canvas+(1.-canvas)*0.5;
 ((v.w-pw)*0.5+u*pw,(v.h-ph)*0.5+y/720.*ph)
}
/// The target under a display point; the last drawn one wins where areas overlap (open lists are drawn last)
pub fn hit(targets:&[([f32;4],Target)],x:f32,y:f32)->Option<Target>{
 targets.iter().rev().find(|([l,t,r,b],_)|x>=*l&&x<*r&&y>=*t&&y<*b).map(|(_,t)|*t)
}
/// Targets that show the gold hover frame and play the cursor sound (not the page backgrounds)
pub fn hoverable(t:Target)->bool{!matches!(t,Target::Panel|Target::Detail|Target::Scrollbar)}
/// First visible row for a pointer y on a scrollbar track (display pixels): the thumb's middle follows the pointer.
/// The thumb has the drawn size (at least 18 of the 272 track units, layout::scrollbar).
pub fn scroll_offset(y:f32,top:f32,bottom:f32,total:usize,visible:usize)->usize{
 let h=bottom-top;if total<=visible||h<=0.{return 0}
 let thumb=(h*visible as f32/total as f32).max(h*18./272.).min(h);let span=(h-thumb).max(1.);
 let t=((y-top-thumb*0.5)/span).clamp(0.,1.);
 (t*(total-visible) as f32+0.5) as usize
}
/// Wheel notches (model SCROLL_UP / SCROLL_DOWN) as events for the current mode: with a row selected (mode 1) or a
/// list open (modes 2, 3) the wheel moves the cursor, except over the material detail panels, which it scrolls.
pub fn wheel_events(wheel:u32,mode:u8,under:Option<Target>)->u32{
 use crate::model::{SCROLL_UP,SCROLL_DOWN,UP,DOWN};
 if wheel==0{return 0}
 let rows=match mode{1=>under!=Some(Target::Detail),2|3=>true,_=>false};
 if !rows{return wheel}
 (if wheel&SCROLL_UP!=0{UP}else{0})|(if wheel&SCROLL_DOWN!=0{DOWN}else{0})
}
/// Clicks count only after every button was seen released while the page is shown: the click that opened the page
/// (a native notes row) must not land on it.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Arm{armed:bool}
impl Arm{
 pub const NEW:Self=Self{armed:false};
 /// Every button was seen released since the page appeared
 pub fn armed(&self)->bool{self.armed}
 /// One frame: true when a new left press may be used as a click
 pub fn step(&mut self,visible:bool,buttons:u32,pressed:u32)->bool{
  if !visible{self.armed=false;return false}
  let click=self.armed&&pressed&1!=0;
  if buttons==0{self.armed=true}
  click
 }
}

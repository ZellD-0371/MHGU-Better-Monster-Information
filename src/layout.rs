//! Geometry shared with native rendering tests; table positions stay independent of scrolling.
pub const PAGE_ROWS:usize=8;
/// Recomp's centered, uniformly scaled HUD. Narrow pictures use the full width
/// and top anchor (the game supports a minimum picture aspect of 16:10).
#[derive(Clone,Copy,Debug)]
pub struct Viewport{pub scale:f32,pub ox:f32,pub oy:f32,pub right_ox:f32}
/// Include both independently anchored canvases and the bottom button row.
pub fn viewport_fits(v:Viewport,w:f32,h:f32)->bool{
    w.is_finite()&&h.is_finite()&&w>0.&&h>0.&&v.scale.is_finite()&&v.scale>0.
    &&[v.ox,v.right_ox,v.oy].iter().all(|x|x.is_finite()&&*x>=-0.01)
    &&v.ox+1280.*v.scale<=w+0.01&&v.right_ox+1280.*v.scale<=w+0.01
    &&v.oy+720.*v.scale<=h+0.01
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Placement{Hud,MissingHud,InvalidHud}
impl Placement{pub fn label(self)->&'static str{match self{Self::Hud=>"native-hud",Self::MissingHud=>"fallback-missing-hud",Self::InvalidHud=>"fallback-invalid-or-offscreen-hud"}}}
pub fn resolve_viewport(w:f32,h:f32,hud:Option<(u32,u32,u32)>)->Option<(Viewport,Placement)>{
    if let Some((rw,rh,uw))=hud{if let Some(v)=hud_viewport(w,h,rw,rh,uw){return Some((v,Placement::Hud))}}
    let v=native_viewport(w,h)?;
    viewport_fits(v,w,h).then_some((v,if hud.is_some(){Placement::InvalidHud}else{Placement::MissingHud}))
}
/// One entry per visible session/input change, rather than one per rendered frame.
pub struct ViewportLog{last:Option<[u32;8]>}
impl ViewportLog{
    pub const NEW:Self=Self{last:None};
    pub fn reset(&mut self){self.last=None;}
    pub fn record(&mut self,enabled:bool,key:[u32;8])->bool{if !enabled{self.reset();false}else{self.changed(key)}}
    pub fn changed(&mut self,key:[u32;8])->bool{if self.last==Some(key){false}else{self.last=Some(key);true}}
}
pub fn native_viewport(w:f32,h:f32)->Option<Viewport>{
    if !w.is_finite()||!h.is_finite()||w<=0.||h<=0.{return None}
    let scale=(w/1280.).min(h/720.);
    if w/h>=16./9.-0.01{let ox=(w-1280.*scale)*0.5;Some(Viewport{scale,ox,oy:(h-720.*scale)*0.5,right_ox:ox})}
    else{Some(Viewport{scale:w/1280.,ox:0.,oy:(h-h.min(w/1.6))*0.5,right_ox:0.})}
}
/// Match the actual native render picture and HUD width, including Edges and
/// fixed aspect ratios. The two native book windows anchor independently.
pub fn hud_viewport(w:f32,h:f32,rw:u32,rh:u32,uw:u32)->Option<Viewport>{
    native_viewport(w,h)?;
    if !(64..=16384).contains(&rw)||!(64..=16384).contains(&rh)||!(1280..=7200).contains(&uw){return None}
    let aspect=rw as f32/rh as f32;
    let ph=h.min(w/aspect);let pw=ph*aspect;
    let scale=(ph/720.).min(pw/1280.);
    let ox=(w-uw as f32*scale)*0.5;
    let v=Viewport{scale,ox,oy:(h-ph)*0.5,right_ox:ox+(uw as f32-1280.)*scale};
    viewport_fits(v,w,h).then_some(v)
}
/// Every mapped DB monster has a page, including the 36 small monsters.
pub fn supports_monster(id:u16)->bool{id>0&&(id as usize)<crate::generated::MONSTERS.len()}
/// The added page is semantic: keep it when changing monsters, even if its index changes.
pub fn extra_page_on_monster_change(was_extra:bool,old_note:u8,note:u8,original_pages:u8,total_pages:u8)->Option<u32>{
    (was_extra&&old_note!=note&&(1..=3).contains(&original_pages)&&total_pages==original_pages+1).then_some(original_pages as u32)
}
pub fn scrollbar(total:usize,visible:usize,offset:usize,height:f32)->Option<(f32,f32)>{
    if visible==0||total<=visible||height<=0.{return None}
    let thumb=(height*visible as f32/total as f32).max(18.).min(height);
    let top=(height-thumb)*offset.min(total-visible) as f32/(total-visible) as f32;
    Some((top,thumb))
}

/// UTF-8 safe, linear prefix fitting with no temporary allocation.
pub fn fitting_prefix(text:&str,available:f32,mut width:impl FnMut(char)->f32)->usize{let mut sum=0.;let mut end=0;for(at,ch)in text.char_indices(){let next=sum+width(ch);if next>available{break}sum=next;end=at+ch.len_utf8();}end}

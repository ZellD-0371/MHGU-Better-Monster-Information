//! Geometry shared with native rendering tests; table positions stay independent of scrolling.
pub const PAGE_ROWS:usize=8;
/// Recomp's centered, uniformly scaled HUD. Narrow pictures use the full width
/// and top anchor (the game supports a minimum picture aspect of 16:10).
#[derive(Clone,Copy,Debug)]
pub struct Viewport{pub scale:f32,pub ox:f32,pub oy:f32,pub right_ox:f32}
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
    Some(Viewport{scale,ox,oy:(h-ph)*0.5,right_ox:ox+(uw as f32-1280.)*scale})
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

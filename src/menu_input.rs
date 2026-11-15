//! Pure input ownership for the final game menu word, not physical HID buttons.
use crate::model as m;
pub const CONFIRM:u32=0x2000;pub const CANCEL:u32=0x4000;
pub const RANK:u32=0x1000;pub const FORM:u32=0x8000;
pub const PREV:u32=0x400;pub const NEXT:u32=0x800;
pub const UP:u32=0x10;pub const DOWN:u32=0x40;
pub const RIGHT_UP:u32=0x10000;pub const RIGHT_DOWN:u32=0x40000;
const OWNED:u32=CONFIRM|RANK|FORM|PREV|NEXT;
pub struct Output{pub filtered:u32,pub events:u32,pub held:u32}
pub struct Gate{previous:u32,captured:u32,waiting:bool}
impl Gate{
 pub const NEW:Self=Self{previous:0,captured:0,waiting:false};
 pub fn step(&mut self,word:u32,active:bool,mode:u8,interactive:bool,rank:bool,forms:bool,overlay:bool)->Output{
  if overlay{self.waiting=true;self.previous=word;return Output{filtered:word,events:0,held:0}}
  let waiting=self.waiting;
  if word==0{self.waiting=false;}
  let pressed=word&!self.previous;self.previous=word;
  let opening=mode==0&&((interactive&&word&CONFIRM!=0)||(rank&&word&RANK!=0)||(forms&&word&FORM!=0));
  let modal=active&&(mode!=0||opening||waiting);
  let release=self.captured&word;
  let capture=release|if active{if modal{word}else{word&OWNED}}else{0};
  self.captured=capture;
  let accepted=if active&&!waiting{if mode!=0{pressed}else{pressed&OWNED}}else{0};
  Output{filtered:word&!capture,events:events(accepted),held:if active&&!waiting{events(word)&(m::UP|m::DOWN)}else{0}}
 }
}
fn events(word:u32)->u32{
 let mut events=0;
 for(mask,event)in [(CONFIRM,m::ACCEPT),(CANCEL,m::BACK),(RANK,m::RANK),(FORM,m::FORM),(PREV,m::PREV),(NEXT,m::NEXT),(UP,m::UP),(DOWN,m::DOWN),(RIGHT_UP,m::SCROLL_UP),(RIGHT_DOWN,m::SCROLL_DOWN)]{if word&mask!=0{events|=event}}
 events
}

#[cfg(test)]mod tests{
 use super::*;
 fn poll(g:&mut Gate,w:u32,mode:u8)->Output{g.step(w,true,mode,true,true,true,false)}
 #[test]fn final_menu_bits_are_decoded_instead_of_hid_bits(){
  let mut g=Gate::NEW;
  assert_eq!(poll(&mut g,CONFIRM,0).events,m::ACCEPT);
  let mut g=Gate::NEW;assert_eq!(poll(&mut g,1,1).events,0); // HID A / engine Minus is not Confirm.
  let mut g=Gate::NEW;assert_eq!(poll(&mut g,CANCEL,1).events,m::BACK);
 }
 #[test]fn remapped_physical_a_cancel_only_closes_mod_until_released(){
  let mut s=m::State::new();s.mode=1;let mut g=Gate::NEW;
  // Upstream maps the physical A to MenuCancel, delivering the engine B bit.
  let first=poll(&mut g,CANCEL,1);assert_eq!(first.filtered,0);s.action(first.events);assert_eq!(s.mode,0);
  for _ in 0..20{let held=poll(&mut g,CANCEL,0);assert_eq!(held.filtered,0);assert_eq!(held.events,0);}
  assert_eq!(poll(&mut g,0,0).filtered,0);
  assert_eq!(poll(&mut g,CANCEL,0).filtered,CANCEL);
 }
 #[test]fn remapped_confirm_is_consumed_on_the_opening_frame(){let mut g=Gate::NEW;let o=poll(&mut g,CONFIRM|DOWN,0);assert_eq!(o.filtered,0);assert_eq!(o.events,m::ACCEPT);}
 #[test]fn modal_owns_all_directions_shoulders_and_unknown_menu_bits(){
  for mode in [1,2,3]{let mut g=Gate::NEW;let word=UP|DOWN|0x20|0x80|0x100|0x200|0x1000000;let o=poll(&mut g,word,mode);assert_eq!(o.filtered,0);assert_eq!(o.events,m::UP|m::DOWN);}
 }
 #[test]fn outside_mod_selection_native_navigation_and_cancel_work(){let mut g=Gate::NEW;let word=CANCEL|UP|0x20|0x100|0x200;assert_eq!(poll(&mut g,word,0).filtered,word);}
 #[test]fn dropdown_cancel_does_not_leak_after_mode_changes(){for mode in [2,3]{let mut s=m::State::new();s.mode=mode;let mut g=Gate::NEW;let o=poll(&mut g,CANCEL,mode);assert_eq!(o.filtered,0);s.action(o.events);assert_eq!(s.mode,0);assert_eq!(poll(&mut g,CANCEL,0).filtered,0);}}
 #[test]fn held_direction_is_queued_once_and_keeps_repeat_state(){let mut g=Gate::NEW;let first=poll(&mut g,UP,1);assert_eq!(first.events,m::UP);for _ in 0..60{let held=poll(&mut g,UP,1);assert_eq!(held.events,0);assert_eq!(held.held,m::UP);assert_eq!(held.filtered,0);}}
 #[test]fn forge_overlay_waits_for_neutral_and_keeps_its_own_input(){let mut g=Gate::NEW;let o=g.step(CONFIRM,true,1,true,true,true,true);assert_eq!(o.filtered,CONFIRM);assert_eq!(o.events,0);let o=poll(&mut g,CONFIRM,1);assert_eq!(o.events,0);assert_eq!(o.filtered,0);poll(&mut g,0,1);assert_eq!(poll(&mut g,CONFIRM,1).events,m::ACCEPT);}
 #[test]fn inactive_notes_do_not_consume_input(){let mut g=Gate::NEW;let o=g.step(CONFIRM|CANCEL|UP,false,0,true,true,true,false);assert_eq!(o.filtered,CONFIRM|CANCEL|UP);assert_eq!(o.events,0);assert_eq!(o.held,0);}
 #[test]fn vanilla_left_stick_and_confirm_are_consumed_together(){
  // Original NSO +0x3de4f8 maps (word >> 16) & 0xf0 into menu directions.
  // Thus left-stick-up + Confirm (0x102000) produces 0x102010, without Actions.
  let mut g=Gate::NEW;let o=poll(&mut g,0x102010,1);
  assert_eq!(o.events,m::UP|m::ACCEPT);assert_eq!(o.held,m::UP);assert_eq!(o.filtered,0);
 }
}

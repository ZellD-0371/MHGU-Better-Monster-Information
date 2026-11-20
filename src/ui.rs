use alloc::{format,string::String,ffi::CString,collections::BTreeMap,vec::Vec};
use forge::{imgui,sys::imgui::{self as sys,ImVec2,ImDrawFlags,ImGuiComboFlags,ImGuiSelectableFlags}};
use crate::{STATE,model::{State,Target},generated as g,resources as r};
static mut FONT:Option<imgui::ImFont>=None;
#[derive(Clone,Copy)]
struct Glyph{advance:f32,top:f32,bottom:f32}
static mut GLYPHS:Option<BTreeMap<(u32,u16),Glyph>>=None;
static mut WRAP:Option<((String,u32,u32,u32),Vec<String>)>=None;
static mut VIEWPORT_LOG:crate::layout::ViewportLog=crate::layout::ViewportLog::NEW;
fn glyph(pixels:f32,ch:char)->Glyph{unsafe{
 let key=(pixels as u32,u16::try_from(ch as u32).unwrap_or(0xfffd));
 let cache=&mut*core::ptr::addr_of_mut!(GLYPHS);let cache=cache.get_or_insert_with(BTreeMap::new);
 if !crate::perf::legacy(){if let Some(g)=cache.get(&key){return *g}}
 let mut g=Glyph{advance:pixels*0.5,top:0.,bottom:pixels};
 if let Some(font)=FONT{let baked=sys::ImFont_GetFontBaked(font.raw(),pixels,1.);if !baked.is_null(){let value=sys::ImFontBaked_FindGlyph(baked,key.1);if !value.is_null(){g=Glyph{advance:(*value).AdvanceX,top:(*value).Y0,bottom:(*value).Y1};}}}
 if !crate::perf::legacy(){if cache.len()>=8192{cache.clear();}cache.insert(key,g);}g
}}
const INK:u32=0xff282a2b;const PAPER:u32=0xffc0dce7;const LINE:u32=0xff91a8b6;const GOLD:u32=0xff50d9f1;const WHITE:u32=0xffe8e9e1;
mod skin { include!("skin_generated.rs"); }
static PENDING_LANGUAGE:core::sync::atomic::AtomicU32=core::sync::atomic::AtomicU32::new(0);
static mut LANGUAGE_FAILED:bool=false;
static mut LANGUAGE_CONTEXT:*mut sys::ImGuiContext=core::ptr::null_mut();
static mut LANGUAGE_HOOK:u32=0;
fn switch_language(lang:usize,save:bool)->bool{
 if crate::language::current()!=Some(lang){let Ok(pack)=crate::language::prepare(lang)else{unsafe{LANGUAGE_FAILED=true;}forge::log::prelude::error!("[monster-info] language load failed; previous language retained");return false};crate::language::commit(pack);unsafe{GLYPHS=None;WRAP=None;}}
 STATE.with(|s|{s.language=lang;s.clock=0.;});unsafe{LANGUAGE_FAILED=false;}
 if save{let _=crate::storage::save_preferences();}true
}
unsafe extern "C" fn before_language_frame(_ctx:*mut sys::ImGuiContext,_hook:*mut sys::ImGuiContextHook){
 let pending=PENDING_LANGUAGE.swap(0,core::sync::atomic::Ordering::AcqRel);
 let lang=if pending>0{pending as usize-1}else{STATE.with(|s|s.language)};
 if pending>0||crate::language::current()!=Some(lang){if !switch_language(lang,pending>0){if let Some(old)=crate::language::current(){STATE.with(|s|s.language=old);}}}
}
pub fn init(){unsafe{
 let ctx=sys::igGetCurrentContext();if ctx==LANGUAGE_CONTEXT&&LANGUAGE_HOOK!=0{return}LANGUAGE_CONTEXT=ctx;LANGUAGE_HOOK=0;
 crate::storage::poll();crate::language::reset();GLYPHS=None;WRAP=None;FONT=None;VIEWPORT_LOG=crate::layout::ViewportLog::NEW;
 let requested=STATE.with(|s|s.language);let language=switch_language(requested,false)||switch_language(0,false);
 if crate::language::font_valid(){FONT=imgui::ImFont::load(c"app:/nativeNX/plugins/better-monster-information/languages/font.shared.otf");}
 let hook=sys::ImGuiContextHook{HookId:0,Type:sys::ImGuiContextHookType::NewFramePre,Owner:0x424d4931,Callback:Some(before_language_frame),UserData:core::ptr::null_mut()};LANGUAGE_HOOK=sys::igAddContextHook(ctx,&hook);
 let font=FONT;let ready=language&&font.is_some()&&LANGUAGE_HOOK!=0&&STATE.with(|s|s.input_ready);
 STATE.with(|s|s.ready=ready);crate::diagnostic_info!("[monster-info] single-language resources and subset font ready={}",ready);
}}
// targets: mouse click areas of this frame in display pixels, shared with the right canvas
struct Canvas{draw:imgui::DrawList,scale:f32,ox:f32,oy:f32,right_ox:f32,view:crate::model::View,targets:alloc::rc::Rc<core::cell::RefCell<Vec<([f32;4],Target)>>>}
impl Canvas{
 fn new()->Option<Self>{
  let draw=imgui::DrawList::foreground();if draw.raw().is_null(){return None}
  let io=imgui::get_io();let sz=io.DisplaySize;let fb=io.DisplayFramebufferScale;
  let base=crate::BASE.load(core::sync::atomic::Ordering::Acquire);let hud=crate::native::hud_size(base);
  let resolved=crate::layout::resolve_viewport(sz.x,sz.y,hud);let(rw,rh,uw)=hud.unwrap_or((0,0,0));
  let key=[sz.x.to_bits(),sz.y.to_bits(),fb.x.to_bits(),fb.y.to_bits(),hud.is_some() as u32,rw,rh,uw];
  if unsafe{(&mut*core::ptr::addr_of_mut!(VIEWPORT_LOG)).record(STATE.with(|s|s.logging),key)}{
   if let Some((v,placement))=resolved{
    crate::diagnostic_info!("[monster-info-layout] display={}x{} framebuffer_scale_milli={},{} hud_present={} render={}x{} hud_width={} placement={} scale_milli={} left_x_milli={} right_x_milli={} top_y_milli={}",sz.x as i32,sz.y as i32,(fb.x*1000.) as i32,(fb.y*1000.) as i32,hud.is_some(),rw,rh,uw,placement.label(),(v.scale*1000.) as i32,(v.ox*1000.) as i32,(v.right_ox*1000.) as i32,(v.oy*1000.) as i32);
   }else{forge::log::prelude::warn!("[monster-info-layout] invalid display_bits={},{} framebuffer_scale_bits={},{} hud_present={} render={}x{} hud_width={}; drawing skipped",key[0],key[1],key[2],key[3],hud.is_some(),rw,rh,uw);}
  }
  let(v,_)=resolved?;Some(Self{draw,scale:v.scale,ox:v.ox,oy:v.oy,right_ox:v.right_ox,view:crate::model::View{w:sz.x,h:sz.y,hud},targets:Default::default()})
 }
 fn right_canvas(&self)->Self{Self{draw:self.draw,scale:self.scale,ox:self.right_ox,oy:self.oy,right_ox:self.right_ox,view:self.view,targets:self.targets.clone()}}
 /// A mouse click area; later areas win where they overlap
 fn target(&self,x:f32,y:f32,w:f32,h:f32,t:Target){let a=self.p(x,y);let b=self.p(x+w,y+h);self.targets.borrow_mut().push(([a.x,a.y,b.x,b.y],t));}
 fn p(&self,x:f32,y:f32)->ImVec2{ImVec2{x:self.ox+x*self.scale,y:self.oy+y*self.scale}}
 fn pixels(&self,size:f32)->f32{((size*self.scale+0.5) as u32).max(1) as f32}
 fn rect(&self,x:f32,y:f32,w:f32,h:f32,col:u32){self.draw.add_rect_filled(self.p(x,y),self.p(x+w,y+h),col,0.);}
 fn border(&self,x:f32,y:f32,w:f32,h:f32,col:u32){self.draw.add_rect(self.p(x,y),self.p(x+w,y+h),col,0.,self.scale,ImDrawFlags::None);}
 fn line(&self,x:f32,y:f32,x2:f32,y2:f32,col:u32,thickness:f32){self.draw.add_line(self.p(x,y),self.p(x2,y2),col,thickness*self.scale);}
 fn icon(&self,id:usize,x:f32,y:f32,size:f32,tint:u32){
  let mut id=id;
  if let Some((layout,action))=g::PROMPTS.iter().enumerate().find_map(|(l,row)|row.iter().position(|&v|v==id).map(|a|(l,a))){
   use crate::button_prompts::{self as bp,Prompt};
   match bp::get(action){
    Prompt::Original=>{},
    Prompt::Position(p)=>id=g::PROMPTS[layout][bp::icon_slot(p,layout)],
    Prompt::Pad(raw)=>{self.text_prompt(x,y,size,&bp::pad_label(&raw,layout));return}
    Prompt::Text(text)=>{self.text_prompt(x,y,size,&text);return}
    Prompt::Key(key)=>{
     if let Some(cell)=crate::generated_kbm::find(&bp::key_sprite(&key)){if r::kbm_icon(self.draw,cell,self.p(x,y),self.p(x+size,y+size),tint){return}}
     self.text_prompt(x,y,size,&bp::key_label(&key));return
    }
   }
  }
  let Some(sprite)=g::SPRITES.get(id)else{return};let ratio=size/(sprite.w.max(sprite.h) as f32);let(w,h)=(sprite.w as f32*ratio,sprite.h as f32*ratio);r::icon(self.draw,id,self.p(x+(size-w)*0.5,y+(size-h)*0.5),self.p(x+(size+w)*0.5,y+(size+h)*0.5),tint,false);}
 // A prompt without a picture: a light key cap with its text shrunk to fit (combos such as "LB+RB")
 fn text_prompt(&self,x:f32,y:f32,size:f32,text:&str){
  self.rect(x,y,size,size,0xffeeeeee);
  let font=size*0.65;let w=self.measure(font,text);let font=if w>size*0.92{(font*size*0.92/w).max(size*0.3)}else{font};
  self.label(x,y,size,size,font,INK,text,true);
 }
 fn measure(&self,size:f32,text:impl AsRef<str>)->f32{let text=text.as_ref();
  let pixels=self.pixels(size);let mut line=0_f32;let mut width=0_f32;if crate::perf::legacy(){if let Some(font)=unsafe{FONT}{unsafe{let baked=sys::ImFont_GetFontBaked(font.raw(),pixels,1.);if !baked.is_null(){for ch in text.chars(){if ch=='\n'{width=width.max(line);line=0.;continue}let g=sys::ImFontBaked_FindGlyph(baked,u16::try_from(ch as u32).unwrap_or(0xfffd));if !g.is_null(){line+=(*g).AdvanceX;}}return width.max(line)/self.scale}}}}
  for ch in text.chars(){if ch=='\n'{width=width.max(line);line=0.;continue}line+=glyph(pixels,ch).advance;}width.max(line)/self.scale
 }
 fn label(&self,x:f32,y:f32,w:f32,h:f32,size:f32,col:u32,text:impl AsRef<str>,center:bool){let text=text.as_ref();
  let mut clipped=String::new();let full=self.measure(size,text);
  let text=if full>w{let available=(w-self.measure(size,"…")).max(0.);let end=crate::layout::fitting_prefix(text,available,|ch|glyph(self.pixels(size),ch).advance/self.scale);clipped.push_str(&text[..end]);clipped.push('…');clipped.as_str()}else{text};
  self.raw_label(x,y,w,h,size,col,text,center,0.);
 }
 fn raw_label(&self,x:f32,y:f32,w:f32,h:f32,size:f32,col:u32,text:impl AsRef<str>,center:bool,offset:f32){let text=text.as_ref();
  let pixels=self.pixels(size);let mut top=f32::MAX;let mut bottom=f32::MIN;
  if crate::perf::legacy(){if let Some(font)=unsafe{FONT}{unsafe{let baked=sys::ImFont_GetFontBaked(font.raw(),pixels,1.);if !baked.is_null(){for ch in text.chars(){let g=sys::ImFontBaked_FindGlyph(baked,u16::try_from(ch as u32).unwrap_or(0xfffd));if !g.is_null()&&(*g).Y1>(*g).Y0{top=top.min((*g).Y0/self.scale);bottom=bottom.max((*g).Y1/self.scale);}}}}}}else{for ch in text.chars(){let g=glyph(pixels,ch);if g.bottom>g.top{top=top.min(g.top/self.scale);bottom=bottom.max(g.bottom/self.scale);}}}
  if top>bottom{top=0.;bottom=size;}
  let tx=if center{x+(w-self.measure(size,text))*0.5}else{x-offset};let ty=y+(h-top-bottom)*0.5;
  self.draw.push_clip_rect(self.p(x,y),self.p(x+w,y+h),true);
  if let Some(font)=unsafe{FONT}{self.draw.add_text_with_font(&font,pixels,self.p(tx,ty),col,text)}else{self.draw.add_text(self.p(tx,ty),col,text)}self.draw.pop_clip_rect();
 }
 fn right(&self,x:f32,y:f32,w:f32,h:f32,size:f32,col:u32,text:impl AsRef<str>){let text=text.as_ref();let mw=self.measure(size,text).min(w);self.label(x+w-mw,y,mw,h,size,col,text,false);}
 fn paragraph(&self,x:f32,y:f32,w:f32,h:f32,size:f32,col:u32,text:impl AsRef<str>){let text=text.as_ref();
  if crate::perf::legacy(){self.paragraph_uncached(x,y,w,h,size,col,text);return}
  let dims=(self.scale.to_bits(),size.to_bits(),w.to_bits());
  unsafe{let cache=&mut*core::ptr::addr_of_mut!(WRAP);
   if cache.as_ref().is_none_or(|((old,a,b,c),_)|old!=text||(*a,*b,*c)!=dims){
    let mut rows=Vec::new();let mut line=String::new();
    for paragraph in text.split('\n'){for word in paragraph.split_whitespace(){let candidate=if line.is_empty(){word.into()}else{format!("{} {}",line,word)};
     if self.measure(size,&candidate)<=w{line=candidate;continue}
     if !line.is_empty(){rows.push(core::mem::take(&mut line));}
     for ch in word.chars(){if self.measure(size,&format!("{}{}",line,ch))>w{rows.push(core::mem::take(&mut line));}line.push(ch);}
    }if !line.is_empty(){rows.push(core::mem::take(&mut line));}}
    *cache=Some(((String::from(text),dims.0,dims.1,dims.2),rows));
   }
   if let Some((_,rows))=cache.as_ref(){let step=size*0.93;for(i,line)in rows.iter().enumerate(){let py=y+i as f32*step;if py+step>y+h{break}self.raw_label(x,py,w,step,size,col,line,false,0.);}}
  }
 }
 fn paragraph_uncached(&self,x:f32,y:f32,w:f32,h:f32,size:f32,col:u32,text:impl AsRef<str>){let text=text.as_ref();
  let mut line=String::new();let mut py=y;let step=size*0.93;
  for paragraph in text.split('\n'){
   for word in paragraph.split_whitespace(){let candidate=if line.is_empty(){word.into()}else{format!("{} {}",line,word)};
    if self.measure(size,&candidate)<=w{line=candidate;continue}
    if !line.is_empty(){self.raw_label(x,py,w,step,size,col,&line,false,0.);line.clear();py+=step;if py+step>y+h{return}}
    for ch in word.chars(){if self.measure(size,&format!("{}{}",line,ch))>w{self.raw_label(x,py,w,step,size,col,&line,false,0.);line.clear();py+=step;if py+step>y+h{return}}line.push(ch);}
   }
   if !line.is_empty(){self.raw_label(x,py,w,step,size,col,&line,false,0.);line.clear();py+=step;if py+step>y+h{return}}
  }
 }
 fn cursor(&self,x:f32,y:f32,active:bool,clock:f32){let phase=(clock*2.2)%1.;let wobble=if active{(0.5-(phase-0.5).abs())*6.}else{0.};let x=x+wobble;
  self.draw.add_triangle_filled(self.p(x,y-10.),self.p(x+12.,y),self.p(x,y+10.),INK);
  self.draw.add_triangle_filled(self.p(x+2.,y-7.),self.p(x+9.,y),self.p(x+2.,y+7.),if active{GOLD}else{0xff8fa8bc});
 }
 fn bar(&self,x:f32,button:usize,label:impl AsRef<str>){let label=label.as_ref();self.rect(x+2.,674.,156.,28.,0xff000000);r::icon(self.draw,g::PROMPT_END,self.p(x,674.),self.p(x+12.,702.),0xffffffff,false);r::icon(self.draw,g::PROMPT_END,self.p(x+148.,674.),self.p(x+160.,702.),0xffffffff,true);self.icon(button,x+8.,679.,18.,WHITE);self.label(x+30.,674.,124.,28.,24.,WHITE,label,false);}
 fn form_bar(&self,s:&State){let label=t(g::UI_FORMSELECT[0],s);let w=(self.measure(24.,&label)+46.).max(160.);let x=569.-w;self.target(x,674.,w,28.,Target::FormOpen);self.rect(x+2.,674.,w-4.,28.,0xff000000);r::icon(self.draw,g::PROMPT_END,self.p(x,674.),self.p(x+12.,702.),0xffffffff,false);r::icon(self.draw,g::PROMPT_END,self.p(x+w-12.,674.),self.p(x+w,702.),0xffffffff,true);self.icon(g::PROMPTS[s.layout][5],x+8.,679.,18.,WHITE);self.label(x+30.,674.,w-36.,28.,24.,WHITE,label,false);}
 fn panel(&self,x:f32,y:f32,w:f32,h:f32){
  if crate::perf::legacy(){self.panel_legacy(x,y,w,h);return}
  self.rect(x+4.,y+4.,w-8.,h-8.,0xdc0c0e21);
  for px in [x,x+w-4.]{self.rect(px,y+8.,4.,h-16.,0xff000000);}
  r::sprite(self.draw,g::SKIN_EDGE,self.p(x+8.,y),self.p(x+w-8.,y+8.),0xffffffff,false,false);
  r::sprite(self.draw,g::SKIN_EDGE,self.p(x+8.,y+h-8.),self.p(x+w-8.,y+h),0xffffffff,true,true);
  for flip_y in [false,true]{for flip_x in [false,true]{let px=if flip_x{x+w-16.}else{x};let py=if flip_y{y+h-16.}else{y};r::sprite(self.draw,g::SKIN_CORNER,self.p(px,py),self.p(px+16.,py+16.),0xff6ce6f8,flip_x,flip_y);}}
 }
 fn panel_legacy(&self,x:f32,y:f32,w:f32,h:f32){
  // Original window_main Pol_Bg RGB and original red/green textured edges.
  // The native list's own GUI is hidden during this modal, so the original
  // translucent fill shows the world without duplicated list labels underneath.
  self.rect(x+4.,y+4.,w-8.,h-8.,0xdc0c0e21);
  for px in [x,x+w-4.]{self.rect(px,y+8.,4.,h-16.,0xff000000);}
  for ry in 0..8{let col=skin::RED_EDGE.iter().find(|&&(rx,y,_,_)|rx==3&&y==ry).map(|&(_,_,_,c)|c).unwrap_or(0xff000000);self.rect(x+8.,y+ry as f32,w-16.,1.,col);self.rect(x+8.,y+h-ry as f32-1.,w-16.,1.,col);}
  for flip_y in [false,true]{for flip_x in [false,true]{for &(rx,ry,rw,col)in skin::CORNER{
   let px=if flip_x{x+w-(rx+rw) as f32}else{x+rx as f32};let py=if flip_y{y+h-ry as f32-1.}else{y+ry as f32};
   self.rect(px,py,rw as f32,1.,if col==0xffffffff{0xff6ce6f8}else{col});
  }}}
 }
 fn scrollbar(&self,x:f32,y:f32,h:f32,total:usize,offset:usize){
  let Some((top,thumb))=crate::layout::scrollbar(total,crate::layout::PAGE_ROWS,offset,h)else{return};
  self.rect(x,y,7.,h,0xff9eafbd);self.border(x,y,7.,h,0xff607585);
  self.rect(x+1.,y+top,5.,thumb,0xff3c4b61);
  if offset>0{self.draw.add_triangle_filled(self.p(x-1.,y-5.),self.p(x+8.,y-5.),self.p(x+3.5,y-11.),INK);}
  if offset+crate::layout::PAGE_ROWS<total{self.draw.add_triangle_filled(self.p(x-1.,y+h+5.),self.p(x+8.,y+h+5.),self.p(x+3.5,y+h+11.),INK);}
 }
}
fn t(id:u32,s:&State)->alloc::rc::Rc<str>{r::text(id,s.language)}
use crate::numeric::{number,percent};
fn diagram_id(s:&State)->u16{if s.id==39&&g::WEAK.get(g::MONSTERS[39].weak as usize+s.row).is_some_and(|row|row.part>=8){130}else{s.id}}
fn weak_table(cv:&Canvas,s:&State){
 let m=&g::MONSTERS[s.id as usize];let diagram=diagram_id(s);let(w,h)=g::DIAGRAMS[diagram as usize];let dx=42.+(298.-w as f32)*0.5;let dy=250.+(215.-h as f32)*0.5;
 if !r::diagram(cv.draw,diagram,cv.p(dx,dy),cv.p(dx+w as f32,dy+h as f32)){if w==0{cv.label(42.,320.,298.,50.,24.,INK,t(g::UI_NODIAGRAM[0],s),true);}else{cv.label(42.,320.,298.,50.,24.,INK,"…",true);}}
 let row=g::WEAK.get(m.weak as usize+s.row);
 if s.mode==1{if let Some(row)=row{
  for i in row.ring as usize..row.ring as usize+row.ring_len as usize{
   let[cx,cy,rx,ry,_]=g::RINGS[i];let[cos,sin]=g::RING_TRIG[i];
   let point=|v:[f32;2]|{let x=v[0]*rx*w as f32;let y=v[1]*ry*h as f32;cv.p(dx+cx*w as f32+x*cos-y*sin,dy+cy*h as f32+x*sin+y*cos)};
   for points in g::CIRCLE.windows(2){cv.draw.add_line(point(points[0]),point(points[1]),0xff22231a,4.*cv.scale);cv.draw.add_line(point(points[0]),point(points[1]),GOLD,2.*cv.scale);}
  }
  if row.pending{cv.label(42.,500.,298.,48.,20.,INK,t(g::UI_PARTPENDING[0],s),true);}
 }}
 cv.rect(364.,179.,520.,37.,0xffb7cbd8);cv.border(364.,179.,520.,37.,LINE);
 for i in 0..3{let x=380.+i as f32*172.;cv.icon(g::ATTACKS[i],x,185.,25.,0xffffffff);cv.label(x+31.,179.,135.,37.,22.,INK,t(g::UI_DAMAGETYPES[i],s),false);}
 cv.rect(364.,226.,520.,37.,0xffa4b7c4);cv.label(368.,226.,146.,37.,20.,INK,t(g::UI_PART[0],s),true);
 for i in 0..8{cv.icon(if i<3{g::ATTACKS[i]}else{g::ELEMENTS[i-3]},518.+46.*i as f32+10.,233.,24.,0xffffffff);}
 for (visible,row) in g::WEAK[m.weak as usize..(m.weak+m.weak_len) as usize].iter().enumerate().skip(s.scroll).take(8){
  let y=268.+(visible-s.scroll) as f32*34.;let selected=visible==s.row;
  if selected{cv.rect(364.,y,520.,33.,if s.mode==1{0xff63cfe8}else{0xffaed0df});cv.cursor(348.,y+17.,s.mode==1,s.clock);}
  cv.line(364.,y+33.,884.,y+33.,LINE,1.);
  let text=t(row.name,s);let width=cv.measure(20.,&text);let available=143.;
  if selected&&width>available&&s.clock>1.{let travel=(width-available)/30.;let elapsed=(s.clock-1.)%(travel+3.);let offset=if elapsed<travel{elapsed*30.}else if elapsed<travel+2.{width-available}else{0.};cv.raw_label(372.,y,available,33.,20.,INK,text,false,offset);}
  else{cv.label(372.,y,available,33.,20.,INK,text,false);}
  for (i,value) in row.values.iter().enumerate(){cv.label(518.+i as f32*46.,y,46.,33.,24.,INK,&number(*value),true);}
 }
 cv.scrollbar(892.,268.,272.,s.rows(),s.scroll);
}
fn ordinary_table(cv:&Canvas,s:&State){
 let m=&g::MONSTERS[s.id as usize];let rows=if s.tab==1{&g::AILMENTS[m.ail as usize..(m.ail+m.ail_len) as usize]}else{&g::TRAPS[m.trap as usize..(m.trap+m.trap_len) as usize]};
 let widths:&[f32]=if s.tab==1{&[162.,130.,115.,130.,160.,119.]}else{&[212.,160.,148.,148.,148.]};
 let headers:&[u32]=if s.tab==1{&[g::UI_TABS[1],g::UI_INITIAL[0],g::UI_INCREASE[0],g::UI_MAXIMUM[0],g::UI_DURATION[0],g::UI_DAMAGE[0]]}else{&[g::UI_TABS[2],g::UI_EFFECTIVE[0],g::UI_NORMAL[0],g::UI_ENRAGED[0],g::UI_FATIGUED[0]]};
 if s.tab==2{cv.label(564.,193.,320.,27.,18.,INK,t(g::UI_TIMEUNIT[0],s),false);}
 cv.rect(68.,226.,816.,37.,0xffa4b7c4);let mut x=68.;for(i,width)in widths.iter().enumerate(){let header=t(headers[i],s);cv.label(x+4.,226.,width-8.,37.,20.,INK,&header,true);x+=width;}
 for(i,row)in rows.iter().enumerate(){let y=268.+i as f32*34.;cv.line(68.,y+33.,884.,y+33.,LINE,1.);cv.icon(row.icon as usize,76.,y+6.,22.,0xffffffff);cv.label(102.,y,widths[0]-39.,33.,20.,INK,t(row.name,s),false);let mut x=68.+widths[0];
  for col in 1..widths.len(){if s.tab==2&&col==1{cv.label(x+4.,y,widths[col]-8.,33.,24.,INK,/* same O / X in every language */if row.values[0]>0.{"O"}else{"X"},true);}else{cv.label(x+4.,y,widths[col]-8.,33.,24.,INK,number(row.values[col-1]),true);}x+=widths[col];}
 }
}
fn material_table(cv:&Canvas,s:&State){
 let other=s.materials().any(|m|m.cells[4]!=-1);let widths:&[f32]=if other{&[276.,108.,108.,108.,108.,108.]}else{&[312.,126.,126.,126.,126.]};
 cv.rect(68.,226.,816.,37.,0xffa4b7c4);cv.label(72.,226.,widths[0]-8.,37.,20.,INK,t(g::UI_MATERIAL[0],s),true);let mut x=68.+widths[0];
 for col in 1..widths.len(){let id=if col<=4{g::UI_GROUPS[col-1]}else{g::UI_OTHER[0]};cv.label(x+4.,226.,widths[col]-8.,37.,20.,INK,t(id,s),true);x+=widths[col];}
 for(i,m)in s.materials().enumerate().skip(s.scroll).take(8){let y=268.+(i-s.scroll) as f32*34.;let item=&g::ITEMS[m.item as usize];if i==s.row{cv.rect(68.,y,816.,33.,if s.mode==1{0xff63cfe8}else{0xffaed0df});cv.cursor(50.,y+17.,s.mode==1,s.clock);}
  cv.line(68.,y+33.,884.,y+33.,LINE,1.);cv.icon(item.icon as usize,76.,y+6.,22.,0xffffffff);cv.label(102.,y,widths[0]-39.,33.,20.,INK,t(item.name,s),false);let mut x=68.+widths[0];
  for col in 1..widths.len(){cv.label(x,y,widths[col],33.,24.,INK,&percent(m.cells[col-1]),true);x+=widths[col];}
 }
 if s.rows()==0{cv.label(68.,300.,816.,60.,24.,INK,t(g::UI_NOHUNT[0],s),true);}
 cv.scrollbar(892.,268.,272.,s.rows(),s.scroll);
}
fn material_detail(cv:&Canvas,s:&State){
 let Some(material)=s.material()else{return};let item=&g::ITEMS[material.item as usize];
 cv.panel(934.,106.,326.,257.);cv.label(948.,114.,298.,38.,25.,WHITE,t(item.name,s),false);
 cv.rect(948.,170.,58.,58.,0xff1c2022);cv.border(948.,170.,58.,58.,0xff626553);cv.icon(item.icon as usize,952.,174.,50.,0xffffffff);
 cv.paragraph(1016.,168.,228.,94.,20.,WHITE,t(item.description,s));
 let rarity=format!("{} {}",t(g::UI_RARITY[0],s),item.rarity);let rarity_width=cv.measure(20.,&rarity);
 cv.label(977.-rarity_width*0.5,235.,rarity_width+0.5,28.,20.,0xffdfdb8a,&rarity,true);
 cv.line(948.,296.,1246.,296.,0xff626553,1.);cv.label(948.,300.,160.,25.,20.,WHITE,t(g::UI_SELL[0],s),false);cv.right(1108.,300.,138.,25.,24.,GOLD,&format!("{} z",item.sell));
 cv.icon(g::CHEST,948.,329.,22.,0xffffffff);cv.right(980.,327.,266.,25.,22.,WHITE,&s.owned.map(|v|format!("x {}",v)).unwrap_or_else(||"—".into()));
 cv.panel(934.,375.,326.,281.);cv.label(948.,386.,298.,34.,25.,WHITE,t(g::UI_ACQUISITION[0],s),false);
 let rows=&g::ACQUISITIONS[material.entry as usize..(material.entry+material.entry_len) as usize];
 for(i,row)in rows.iter().enumerate().skip(s.detail_scroll).take(8){let y=424.+(i-s.detail_scroll) as f32*27.;cv.label(948.,y,190.,27.,20.,WHITE,&format!("◆ {}",t(row.name,s)),false);cv.right(1138.,y,47.,27.,20.,WHITE,&format!("×{}",row.quantity));cv.right(1188.,y,58.,27.,20.,WHITE,&percent(row.percent as i16));cv.line(948.,y+26.,1246.,y+26.,0xff626553,1.);}
 if rows.len()>8{cv.label(948.,638.,298.,14.,15.,WHITE,"R ↑ / ↓",true);}
}
fn ranks(cv:&Canvas,s:&State){
 if s.tab!=3||s.rank==0{return}
 cv.rect(680.,180.,206.,34.,0xff98b4c5);cv.border(680.,180.,206.,34.,0xff3d5069);
 if s.ranks.len()>1{cv.icon(g::PROMPTS[s.layout][2],689.,184.,26.,if s.mode==1{0x66ffffff}else{0xffffffff});cv.label(724.,180.,145.,34.,24.,INK,t(g::RANK_NAMES[s.rank as usize],s),true);cv.draw.add_triangle_filled(cv.p(869.,194.),cv.p(879.,194.),cv.p(874.,200.),INK);}
 else{cv.label(684.,180.,198.,34.,24.,INK,t(g::RANK_NAMES[s.rank as usize],s),true);}
 if s.mode!=2{return}let top=s.rank_cursor.saturating_sub(5);let count=s.ranks.len().min(6);let y=218.;cv.rect(598.,y,288.,count as f32*38.+8.,0xff292e31);cv.border(598.,y,288.,count as f32*38.+8.,GOLD);
 for(i,rank)in s.ranks.iter().enumerate().skip(top).take(6){let py=y+4.+(i-top) as f32*38.;cv.target(602.,py,280.,38.,Target::RankItem(i));if i==s.rank_cursor{cv.rect(602.,py,280.,38.,0xff1681a0);cv.cursor(608.,py+19.,false,0.);}cv.label(632.,py,238.,38.,24.,WHITE,t(g::RANK_NAMES[*rank as usize],s),false);}
}
fn forms(cv:&Canvas,s:&State){
 if !s.has_forms(){return}
 cv.rect(42.,180.,298.,34.,0xff98b4c5);cv.border(42.,180.,298.,34.,0xff3d5069);
 cv.icon(g::PROMPTS[s.layout][5],50.,185.,24.,if s.mode==1{0x66ffffff}else{0xffffffff});
 cv.label(80.,180.,240.,34.,20.,INK,t(g::MONSTERS[s.id as usize].name,s),false);
 cv.draw.add_triangle_filled(cv.p(325.,194.),cv.p(335.,194.),cv.p(330.,200.),INK);
 if s.mode!=3{return}let forms=s.forms();
 let w=forms.iter().map(|&id|cv.measure(24.,t(g::MONSTERS[id as usize].name,s))+64.).fold(298_f32,f32::max).min(600.);
 cv.rect(42.,218.,w,forms.len() as f32*38.+8.,0xff292e31);cv.border(42.,218.,w,forms.len() as f32*38.+8.,GOLD);
 for(i,&id)in forms.iter().enumerate(){let y=222.+i as f32*38.;cv.target(46.,y,w-8.,38.,Target::FormItem(i));if i==s.form_cursor{cv.rect(46.,y,w-8.,38.,0xff1681a0);cv.cursor(52.,y+19.,false,0.);}cv.label(76.,y,w-48.,38.,24.,WHITE,t(g::MONSTERS[id as usize].name,s),false);}
}
fn screen(cv:&Canvas,s:&State){
 // Draw only the requested left content area. The game's name/page header,
 // eight-row monster list and the world behind the book remain native.
 let detail=s.mode==1&&s.tab==3;
 cv.target(6.,94.,905.,562.,Target::Panel);
 cv.rect(6.,94.,905.,562.,PAPER);cv.border(6.,94.,905.,562.,0xff9fb7c5);cv.border(13.,101.,891.,548.,0xffa9c2ce);
 for i in 0..4{let x=76.+i as f32*196.;cv.target(x,130.,184.,38.,Target::Tab(i));cv.rect(x,130.,184.,38.,if i==s.tab{0xff292887}else{0xffb0c5d1});cv.label(x+5.,130.,174.,38.,25.,if i==s.tab{WHITE}else{INK},t(g::UI_TABS[i],s),true);}
 cv.target(33.,130.,38.,38.,Target::Prev);cv.target(856.,130.,38.,38.,Target::Next);
 cv.icon(g::PROMPTS[s.layout][3],39.,136.,26.,if s.mode!=0{0x66ffffff}else{0xffffffff});cv.icon(g::PROMPTS[s.layout][4],862.,136.,26.,if s.mode!=0{0x66ffffff}else{0xffffffff});
 match s.tab{0=>weak_table(cv,s),1|2=>ordinary_table(cv,s),_=>material_table(cv,s)}
 if detail{
  // Selecting a material owns input, so its existing panels may cover the native list.
  let rc=cv.right_canvas();rc.target(934.,44.,326.,612.,Target::Detail);rc.panel(934.,44.,326.,52.);rc.label(950.,44.,294.,52.,27.,GOLD,t(g::UI_DETAIL[0],s),false);material_detail(&rc,s);
 }
 // Rows of the selectable tables (weak points, materials)
 if matches!(s.tab,0|3)&&s.interactive(){let(x,w)=if s.tab==0{(364.,520.)}else{(68.,816.)};for k in 0..s.rows().saturating_sub(s.scroll).min(crate::layout::PAGE_ROWS){cv.target(x,268.+k as f32*34.,w,33.,Target::Row(s.scroll+k));}}
 // Scrollbar track (drawn at 892,268 7x272), a little wider so it is easy to grab
 if matches!(s.tab,0|3)&&s.rows()>crate::layout::PAGE_ROWS{cv.target(886.,268.,19.,272.,Target::Scrollbar);}
 if s.tab==3&&s.rank!=0&&s.ranks.len()>1{cv.target(680.,180.,206.,34.,Target::RankOpen);}
 if s.has_forms(){cv.target(42.,180.,298.,34.,Target::FormOpen);}
 cv.target(749.,674.,160.,28.,Target::Back);if matches!(s.mode,2|3)||s.mode==0&&s.interactive(){cv.target(579.,674.,160.,28.,Target::Enter);}
 ranks(cv,s);forms(cv,s);cv.bar(749.,g::PROMPTS[s.layout][1],t(g::UI_BACK[0],s));if matches!(s.mode,2|3)||s.mode==0&&s.interactive(){cv.bar(579.,g::PROMPTS[s.layout][0],t(g::UI_ENTER[0],s));}if s.mode==0&&s.has_forms(){cv.form_bar(s);}
 hover_frame(cv,s);
}
// Gold frame around the target under the Recomp pointer (dark rim first so it reads on paper and on black)
fn hover_frame(cv:&Canvas,s:&State){
 let Some(hover)=s.hover else{return};
 for([l,t,r,b],target)in cv.targets.borrow().iter(){
  if *target!=hover{continue}
  let e=cv.scale*2.;let(a,z)=(ImVec2{x:l-e,y:t-e},ImVec2{x:r+e,y:b+e});
  cv.draw.add_rect(a,z,0xc0181a1b,3.*cv.scale,4.*cv.scale,ImDrawFlags::None);cv.draw.add_rect(a,z,GOLD,3.*cv.scale,2.*cv.scale,ImDrawFlags::None);
 }
}
pub fn render(){
 let base=crate::BASE.load(core::sync::atomic::Ordering::Acquire);
 let notes_open=crate::native::notes_open(base);
 if notes_open||forge::graphics::Graphics::is_menu_visible(){crate::button_prompts::refresh(imgui::get_io().DeltaTime); }
 r::maintain_diagram(notes_open);
 // Preload on native notes pages too, before the additional page needs icons.
 r::maintain_icons(notes_open&&STATE.with(|s|s.ready&&s.enabled),imgui::get_io().DeltaTime);
 r::maintain_kbm(notes_open&&STATE.with(|s|s.ready&&s.enabled)&&crate::button_prompts::is_recomp(),imgui::get_io().DeltaTime);
 let visible=STATE.with(|s|s.visible&&s.ready);if !visible{unsafe{(&mut*core::ptr::addr_of_mut!(VIEWPORT_LOG)).reset();}return}if forge::graphics::Graphics::is_menu_visible(){return}
 let(id,tab)=STATE.with(|s|(diagram_id(s),s.tab));if tab==0{r::load_diagram(id);}let Some(cv)=Canvas::new()else{return};STATE.with(|s|{screen(&cv,s);s.targets=core::mem::take(&mut *cv.targets.borrow_mut());s.view=Some(cv.view);});
}
pub fn menu(){
 if !imgui::tree_node(c"Better Monster Information"){return}
 let font=unsafe{FONT};if let Some(f)=font{imgui::push_font(&f,Some(18.));}
 let names=["English","简体中文","繁體中文","日本語","한국어","Deutsch","Français","Italiano","Español"];let layouts=["Nintendo","Xbox","PlayStation"];let(lang,layout)=STATE.with(|s|(s.language,s.layout));let mut changed=false;
 for(label,options,current,is_language)in [(c"Language",&names[..],lang,true),(c"Button layout",&layouts[..],layout,false)]{
  let selected=CString::new(options[current]).unwrap();if imgui::begin_combo(label,&selected,ImGuiComboFlags::None){for(i,name)in options.iter().enumerate(){let text=CString::new(*name).unwrap();if imgui::selectable(&text,i==current,ImGuiSelectableFlags::None,ImVec2{x:0.,y:0.}){if is_language{PENDING_LANGUAGE.store(i as u32+1,core::sync::atomic::Ordering::Release);}else{STATE.with(|s|{s.layout=i;s.clock=0.;});changed=true;}}}imgui::end_combo();}
 }
 let mut logging=STATE.with(|s|s.logging);
 if imgui::checkbox(crate::settings_labels::LOGGING[lang],&mut logging){STATE.with(|s|s.logging=logging);crate::diagnostics::set_enabled(logging);unsafe{(&mut*core::ptr::addr_of_mut!(VIEWPORT_LOG)).reset();}changed=true;}
 if changed&&!crate::storage::save_preferences(){imgui::text("Settings could not be saved. See Forge log.");}
 imgui::text(&r::text(g::UI_NOTESINFO[0],STATE.with(|s|s.language)));
 if !STATE.with(|s|s.ready){imgui::text("Unavailable: resource, input, or signature check failed.");}
 if unsafe{LANGUAGE_FAILED}{imgui::text("Language resources could not be loaded. The previous language is retained.");}
 if font.is_some(){imgui::pop_font();}imgui::tree_pop();
}

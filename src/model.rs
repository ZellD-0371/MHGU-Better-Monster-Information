//! Input ownership and table/rank behaviour, independent of game pointers and drawing.
use alloc::vec::Vec;
use crate::{data::Material,generated as g};
pub const ACCEPT:u32=1;pub const BACK:u32=2;pub const RANK:u32=4;pub const PREV:u32=8;pub const NEXT:u32=16;pub const UP:u32=32;pub const DOWN:u32=64;
pub const SCROLL_UP:u32=128;pub const SCROLL_DOWN:u32=256;
pub const FORM:u32=512;
// Temporarily disabled by user request. Progression and individual-hunt gates remain active.
pub const REQUIRE_RANK_HUNT:bool=false;
#[derive(Clone,Copy)]
pub struct Progress{pub flags:[u32;64],pub cleared:[u8;256],pub hunted:[u16;130],pub hr:u16,pub hub:u16,pub special_hunted:u8,pub valid:bool}
impl Progress{
    pub const EMPTY:Self=Self{flags:[0;64],cleared:[0;256],hunted:[0;130],hr:0,hub:0,special_hunted:0,valid:false};
    pub fn hunted_special(&self,id:u16)->bool{self.valid&&g::SPECIAL_RECORDS.iter().position(|&(db,_,_)|db==id).is_some_and(|i|self.special_hunted&(1<<i)!=0)}
    pub fn flag(&self,id:usize)->bool{self.valid&&id<2048&&self.flags[id/32]&(1<<(id%32))!=0}
    pub fn cleared(&self,quest:u32)->bool{
        if !self.valid{return false}
        let Ok(i)=g::QUESTS.binary_search_by_key(&quest,|&(q,_)|q) else{return false};
        let bit=g::QUESTS[i].1 as usize;bit<2048&&self.cleared[bit/8]&(1<<(bit%8))!=0
    }
}
pub fn rule_open(rule:&str,p:&Progress)->bool{
    fn predicate(term:&str,p:&Progress)->bool{
        let (kind,arg)=term.split_once(':').unwrap_or((term,""));
        let number=|s:&str|s.parse::<u32>().unwrap_or(u32::MAX);
        match kind{
            "always"=>true,"flag"=>p.flag(number(arg) as usize),"cleared"=>p.cleared(number(arg)),
            "hr"=>p.valid&&p.hr as u32>=number(arg),"hub_star"=>p.valid&&p.hub as u32>=number(arg),
            "all"=>arg.split('|').all(|q|p.cleared(number(q))),
            "atleast"=>{let Some((count,quests))=arg.split_once(':') else{return false};quests.split('|').filter(|q|p.cleared(number(q))).count()>=number(count) as usize},
            _=>false,
        }
    }
    !rule.is_empty()&&rule.split(" OR ").any(|alt|alt.split(" AND ").all(|term|predicate(term,p)))
}
pub const RANK_RANGES:[(u32,u32);13]=[(0,0),(0,0),(0,0),(0,0),(1,3),(4,5),(6,8),(1,4),(5,8),(9,10),(11,12),(13,15),(16,16)];
pub fn rank_unlocked(id:u16,rank:u8,p:&Progress)->bool{
    let Some(monster)=g::MONSTERS.get(id as usize) else{return false};
    if monster.deviant<0{return rank==1||rank==2&&(p.flag(1008)||p.flag(113))||rank==3&&p.flag(1401)}
    let Some(&(lo,hi))=RANK_RANGES.get(rank as usize) else{return false};
    let first=if monster.deviant<12{1}else{11};
    for level in first..=16{
        let quest=40100+monster.deviant as u32*100+level;
        let Some(&(_,text))=g::RULES.iter().find(|&&(q,_)|q==quest) else{break};
        if !rule_open(&crate::resources::text(text,0),p){break}
        if level>=lo&&level<=hi{return true}
    }false
}
pub fn rank_open(id:u16,rank:u8,p:&Progress)->bool{
    if !rank_unlocked(id,rank,p)||!p.valid||rank>12{return false}
    if !REQUIRE_RANK_HUNT{return true}
    if p.hunted.get(id as usize).is_some_and(|mask|mask&(1<<rank)!=0){return true}
    let m=&g::MONSTERS[id as usize];if m.deviant<0{return false}
    let(lo,hi)=RANK_RANGES[rank as usize];
    (lo..=hi).any(|level|p.cleared(40100+m.deviant as u32*100+level))
}
pub struct State{
    pub enabled:bool,pub installed:bool,pub ready:bool,pub input_ready:bool,pub visible:bool,
    pub id:u16,pub native_id:u16,pub form_cursor:usize,pub native_note:u8,pub original_pages:u8,pub mode:u8,pub tab:usize,pub row:usize,pub scroll:usize,
    pub rank:u8,pub ranks:Vec<u8>,pub rank_cursor:usize,pub detail_scroll:usize,pub clock:f32,
    pub language:usize,pub layout:usize,pub logging:bool,pub progress:Progress,
    pub progress_age:f32,pub stock_age:f32,pub stock_item:Option<u16>,
    pub owned:Option<u32>,
    /// Mouse targets of the last drawn frame, in display pixels (render thread writes, game thread reads)
    pub targets:Vec<([f32;4],Target)>,
    /// Display size and HUD of the last drawn frame, for Recomp mouse positions
    pub view:Option<View>,
    /// Target under the Recomp pointer (gold frame); None without a keyboard / mouse player
    pub hover:Option<Target>,
    repeat_direction:i8,repeat_time:f32,
}
/// What a mouse click on our page does (Recomp only, mouse.rs)
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Target{Tab(usize),Prev,Next,Row(usize),RankOpen,RankItem(usize),FormOpen,FormItem(usize),Enter,Back,
    /// Our paper page and the material detail panels: clicks there are ours but do nothing
    Panel,Detail,
    /// The table scrollbar: press to jump, hold to drag (no hover frame)
    Scrollbar}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct View{pub w:f32,pub h:f32,pub hud:Option<(u32,u32,u32)>}
impl State{
    pub fn new()->Self{Self{enabled:true,installed:false,ready:false,input_ready:false,visible:false,id:48,native_id:48,form_cursor:0,native_note:34,original_pages:3,mode:0,tab:0,row:0,scroll:0,rank:1,ranks:Vec::new(),rank_cursor:0,detail_scroll:0,clock:0.,language:0,layout:0,logging:false,progress:Progress::EMPTY,progress_age:1.,stock_age:1.,stock_item:None,owned:None,targets:Vec::new(),view:None,hover:None,repeat_direction:0,repeat_time:0.}}
    pub fn load_preferences(&mut self,text:&str){
        for line in text.lines(){let Some((key,value))=line.split_once('=')else{continue};let Ok(n)=value.trim().parse::<usize>()else{continue};match key.trim(){"language" if n<9=>self.language=n,"layout" if n<3=>self.layout=n,"logging" if n<=1=>self.logging=n==1,_=>{}}}
    }
    pub fn encode_preferences(&self)->alloc::string::String{alloc::format!("[BetterMonsterInformation]\nlanguage={}\nlayout={}\nlogging={}\n",self.language,self.layout,self.logging as u8)}
    pub fn form_ids(&self)->impl Iterator<Item=u16>+'_ {
        g::FAMILIES.iter().filter_map(|&(parent,id)|{
            if parent!=self.native_id{return None}
            let m=&g::MONSTERS[id as usize];
            (id==parent||self.progress.hunted_special(id)&&g::MATERIALS[m.material as usize..(m.material+m.material_len) as usize].iter().any(|r|rank_unlocked(id,r.rank,&self.progress))).then_some(id)
        })
    }
    pub fn forms(&self)->Vec<u16>{self.form_ids().collect()}
    pub fn has_forms(&self)->bool{self.form_ids().nth(1).is_some()}
    pub fn reset_form(&mut self){self.id=self.native_id;self.form_cursor=0;self.recompute(true);}
    pub fn materials(&self)->impl Iterator<Item=&'static Material>{let m=&g::MONSTERS[self.id as usize];g::MATERIALS[m.material as usize..(m.material+m.material_len) as usize].iter().filter(|r|r.rank==self.rank)}
    pub fn material(&self)->Option<&'static Material>{self.materials().nth(self.row)}
    pub fn rows(&self)->usize{let m=&g::MONSTERS[self.id as usize];match self.tab{0=>m.weak_len as usize,1=>m.ail_len as usize,2=>m.trap_len as usize,_=>self.materials().count()}}
    pub fn interactive(&self)->bool{matches!(self.tab,0|3)&&self.rows()>0}
    pub fn recompute(&mut self,reset_rank:bool){
        let m=&g::MONSTERS[self.id as usize];let old=self.rank;self.ranks.clear();
        for r in &g::MATERIALS[m.material as usize..(m.material+m.material_len) as usize]{
            if !self.ranks.contains(&r.rank)&&rank_open(self.id,r.rank,&self.progress){self.ranks.push(r.rank)}
        }
        self.ranks.sort_unstable();self.rank=if !reset_rank&&self.ranks.contains(&old){old}else{self.ranks.last().copied().unwrap_or(0)};
        self.row=self.row.min(self.rows().saturating_sub(1));self.scroll=self.scroll.min(self.row);
        self.rank_cursor=self.ranks.iter().position(|&r|r==self.rank).unwrap_or(0);
    }
    pub fn select(&mut self,id:u16,note:u8,pages:u8){
        if self.native_id!=id{self.native_id=id;self.id=id;self.form_cursor=0;self.mode=0;self.row=0;self.scroll=0;self.detail_scroll=0;self.clock=0.;self.recompute(true)}
        self.native_note=note;self.original_pages=pages;
    }
    pub fn action(&mut self,event:u32){
        if self.mode==3{
            let forms=self.forms();
            if event&(BACK|FORM)!=0{self.mode=0}
            else if event&ACCEPT!=0{if let Some(&id)=forms.get(self.form_cursor){if id!=self.id{self.id=id;self.row=0;self.scroll=0;self.detail_scroll=0;self.clock=0.;self.owned=None;self.recompute(true);}}self.mode=0}
            else if event&(UP|DOWN)!=0&&!forms.is_empty(){let n=forms.len();self.form_cursor=(self.form_cursor+n+if event&DOWN!=0{1}else{n-1})%n}
            return
        }
        if self.mode==2{
            if event&(BACK|RANK)!=0{self.mode=0}
            else if event&ACCEPT!=0{if let Some(&rank)=self.ranks.get(self.rank_cursor){if rank!=self.rank{self.rank=rank;self.row=0;self.scroll=0;self.detail_scroll=0;self.clock=0.;}}self.mode=0}
            else if event&(UP|DOWN)!=0&&!self.ranks.is_empty(){let n=self.ranks.len();self.rank_cursor=(self.rank_cursor+n+if event&DOWN!=0{1}else{n-1})%n}
            return
        }
        if event&BACK!=0{if self.mode==1{self.mode=0;}return}
        if self.mode==1&&self.tab==3&&event&(SCROLL_UP|SCROLL_DOWN)!=0{
            let max=self.material().map_or(0,|m|m.entry_len as usize).saturating_sub(8);
            self.detail_scroll=if event&SCROLL_UP!=0{self.detail_scroll.saturating_sub(1)}else{(self.detail_scroll+1).min(max)};return
        }
        if self.mode==0{
            if event&FORM!=0&&self.has_forms(){self.mode=3;self.form_cursor=self.forms().iter().position(|&id|id==self.id).unwrap_or(0)}
            else if event&(PREV|NEXT)!=0{self.tab=(self.tab+if event&NEXT!=0{1}else{3})%4;self.row=0;self.scroll=0;self.detail_scroll=0;self.clock=0.;self.recompute(true)}
            else if event&RANK!=0&&self.tab==3&&self.ranks.len()>1{self.mode=2;self.rank_cursor=self.ranks.iter().position(|&r|r==self.rank).unwrap_or(0)}
            else if event&ACCEPT!=0&&self.interactive(){self.mode=1;}
        }else if event&(UP|DOWN)!=0{
            let n=self.rows();if n>0{self.row=(self.row+n+if event&DOWN!=0{1}else{n-1})%n;self.detail_scroll=0;self.clock=0.;}
        }
        if self.row<self.scroll{self.scroll=self.row}if self.row>=self.scroll+8{self.scroll=self.row-7}
    }
    /// Scroll the table to a first visible row (scrollbar drag); the selected row is kept inside the visible rows
    pub fn scroll_to(&mut self,offset:usize){
        let rows=self.rows();let page=crate::layout::PAGE_ROWS;if rows<=page{return}
        self.scroll=offset.min(rows-page);
        let row=self.row.clamp(self.scroll,self.scroll+page-1);
        if row!=self.row{self.row=row;self.detail_scroll=0;self.clock=0.;}
    }
    fn set_tab(&mut self,tab:usize){if tab!=self.tab&&tab<4{self.tab=tab;self.row=0;self.scroll=0;self.detail_scroll=0;self.clock=0.;self.recompute(true)}}
    /// A left click on one of our targets. Returns true when the game itself should get a cancel (the back bar while
    /// the game still owns cancel, mode 0). An open list closes on any click that is not one of its rows.
    pub fn click(&mut self,target:Target)->bool{
        if matches!(self.mode,2|3){
            match(self.mode,target){
                (2,Target::RankItem(i))if i<self.ranks.len()=>{self.rank_cursor=i;self.action(ACCEPT)}
                (3,Target::FormItem(i))if i<self.forms().len()=>{self.form_cursor=i;self.action(ACCEPT)}
                _=>self.mode=0,
            }
            return false
        }
        match target{
            Target::Tab(i)=>{self.mode=0;self.set_tab(i)}
            Target::Prev|Target::Next=>{self.mode=0;self.action(if target==Target::Next{NEXT}else{PREV})}
            Target::Row(i)=>if self.interactive()&&i<self.rows(){
                if i!=self.row{self.row=i;self.detail_scroll=0;self.clock=0.;}self.mode=1;
                if self.row<self.scroll{self.scroll=self.row}if self.row>=self.scroll+8{self.scroll=self.row-7}
            },
            Target::RankOpen=>{self.mode=0;self.action(RANK)}
            Target::FormOpen=>{self.mode=0;self.action(FORM)}
            Target::Enter=>self.action(ACCEPT),
            Target::Back=>{if self.mode==0{return true}self.action(BACK)}
            Target::RankItem(_)|Target::FormItem(_)|Target::Panel|Target::Detail|Target::Scrollbar=>{}
        }
        false
    }
    pub fn tick(&mut self,dt:f32,held:u32){
        self.clock+=dt.clamp(0.,0.1);
        let direction=if held&UP!=0{-1}else if held&DOWN!=0{1}else{0};
        if direction==0||self.mode==0{self.repeat_direction=0;self.repeat_time=0.;return}
        if direction!=self.repeat_direction{self.repeat_direction=direction;self.repeat_time=0.36;return}
        self.repeat_time-=dt.clamp(0.,0.1);if self.repeat_time<=0.{self.repeat_time+=0.09;self.action(if direction>0{DOWN}else{UP})}
    }
}

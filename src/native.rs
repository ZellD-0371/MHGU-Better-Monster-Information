//! MHGU global 1.4.0 Hunter's Notes adapter; character and inventory data are read only.
use core::{ptr,sync::atomic::{AtomicU32,Ordering}};
use forge::log::prelude::info;
static OWNER:AtomicU32=AtomicU32::new(0);
static LAST:AtomicU32=AtomicU32::new(u32::MAX);
static COUNT:AtomicU32=AtomicU32::new(0);
static mut MASKED_GUI:u32=0;static mut MASKED_COUNT:usize=0;
static mut MASKED_GROUPS:[(u32,bool);128]=[(0,false);128];
pub fn valid(p:u32)->bool{(0x10000..0xeffff000).contains(&p)&&p&3==0}
pub unsafe fn rd(p:u32)->u32{unsafe{ptr::read_unaligned(p as *const u32)}}
pub unsafe fn byte(p:u32)->u8{unsafe{ptr::read(p as *const u8)}}
pub fn hud_size(base:u32)->Option<(u32,u32,u32)>{
    let render=unsafe{rd(base+0x2126998)};
    if !valid(render)||unsafe{rd(render)}!=base+0x1754a58{return None}
    Some(unsafe{(rd(render+0x44bac0),rd(render+0x44bac4),rd(render+0x44bac8))})
}

#[forge::hook(offset=0x5b20b0)]
fn detail_refresh(owner:u32)->u32{
    let base=crate::BASE.load(Ordering::Acquire);
    if let Some((_,pager,id,note,_))=selected(base,owner){
        let target=crate::STATE.with(|s|crate::layout::extra_page_on_monster_change(s.enabled&&s.ready&&s.visible&&crate::layout::supports_monster(id),s.native_note,note,COUNT.load(Ordering::Acquire) as u8,unsafe{byte(owner+0x50)}));
        if let Some(page)=target{if unsafe{virtual_get(base,pager,0x24)}!=Some(page){unsafe{virtual_set(base,pager,0x3c,page);}}}
    }
    let result=original!(owner);
    OWNER.store(owner,Ordering::Release);
    refresh_extra_title(crate::BASE.load(Ordering::Acquire),owner);result
}
#[forge::hook(offset=0x5b2be0)]
fn detail_count(owner:u32)->u32{
    let result=original!(owner);
    OWNER.store(owner,Ordering::Release);
    let base=crate::BASE.load(Ordering::Acquire);
    let count=unsafe{byte(owner+0x50)};
    COUNT.store(count as u32,Ordering::Release);
    let ready=crate::STATE.with(|s|s.installed&&s.ready&&s.enabled);
    if ready&&selected(base,owner).is_some_and(|(_,_,id,_,_)|crate::layout::supports_monster(id))&&(1..=3).contains(&count){
        unsafe{ptr::write((owner+0x50) as *mut u8,count+1)};
    }
    result
}
#[forge::hook(offset=0x5b2080)]
fn detail_close(owner:u32)->u32{
    restore_list();
    if OWNER.load(Ordering::Acquire)==owner{OWNER.store(0,Ordering::Release);LAST.store(u32::MAX,Ordering::Release);}
    original!(owner)
}
#[forge::hook(offset=0x5b1f94)]
fn detail_destroy(owner:u32)->u32{
    restore_list();
    if OWNER.load(Ordering::Acquire)==owner{OWNER.store(0,Ordering::Release);LAST.store(u32::MAX,Ordering::Release);}
    original!(owner)
}
pub fn restore_list(){unsafe{
    for i in 0..MASKED_COUNT{let(p,visible)=MASKED_GROUPS[i];if valid(p){let flags=rd(p+0x54);ptr::write_unaligned((p+0x54) as *mut u32,(flags&!1)|visible as u32);}}
    MASKED_COUNT=0;MASKED_GUI=0;
}}
pub fn material_focus(base:u32,focus:bool){unsafe{
    if !focus{if MASKED_GUI!=0{restore_list()}return}
    let owner=OWNER.load(Ordering::Acquire);let Some((list,_,_,_,_))=selected(base,owner)else{restore_list();return};
    let gui=rd(list+0x24);if !valid(gui)||gui==rd(owner+0x24){return}
    if gui!=MASKED_GUI{
        restore_list();let count=rd(gui+0x298) as usize;let groups=rd(gui+0xf8);
        if count>128||!valid(groups){return}
        for i in 0..count{let p=rd(groups+i as u32*4);if valid(p){MASKED_GROUPS[i]=(p,rd(p+0x54)&1!=0)}else{MASKED_GROUPS[i]=(0,false)}}
        MASKED_COUNT=count;MASKED_GUI=gui;
    }
    for i in 0..MASKED_COUNT{let(p,_)=MASKED_GROUPS[i];if valid(p){ptr::write_unaligned((p+0x54) as *mut u32,rd(p+0x54)&!1);}}
}}
pub fn install(base:u32)->bool{
    for (off,word,second) in [(0x5b20b0,0xe92d4ff0,0xe24dd01c),(0x5b2be0,0xe92d4bf0,0xe1a04000),(0x5b2080,0xe92d4010,0xe1a04000),(0x5b1f94,0xe92d4010,0xe1a04000)]{
        if unsafe{rd(base+off)}!=word||unsafe{rd(base+off+4)}!=second{return false}
    }
    forge::install_hook!(base,detail_refresh);forge::install_hook!(base,detail_count);
    forge::install_hook!(base,detail_close);forge::install_hook!(base,detail_destroy);true
}
unsafe fn virtual_get(base:u32,object:u32,slot:u32)->Option<u32>{
    if !valid(object){return None}let vt=unsafe{rd(object)};
    if !(base+0x1400000..base+0x1830000).contains(&vt){return None}
    let address=unsafe{rd(vt+slot)};if !(base..base+0x1400000).contains(&address){return None}
    let get:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(address)};
    Some(unsafe{get(object)})
}
unsafe fn virtual_set(base:u32,object:u32,slot:u32,value:u32){
    if !valid(object){return}let vt=unsafe{rd(object)};
    if !(base+0x1400000..base+0x1830000).contains(&vt){return}
    let address=unsafe{rd(vt+slot)};if !(base..base+0x1400000).contains(&address){return}
    let set:unsafe extern "C" fn(u32,u32)->u32=unsafe{core::mem::transmute(address)};
    unsafe{set(object,value)};
}
fn selected(base:u32,owner:u32)->Option<(u32,u32,u16,u8,usize)>{
    if !valid(owner)||base==0{return None}
    if unsafe{rd(owner)}!=base+0x177062c{return None}
    let list=unsafe{rd(owner+0x58)};let pager=unsafe{rd(owner+0x4c)};
    if !valid(list)||unsafe{rd(list)}!=base+0x1770010{return None}
    let selected=unsafe{virtual_get(base,rd(list+0x4c),0x38)};
    let selected=selected?;if selected>=unsafe{byte(list+0x50)} as u32{return None}
    let entries=unsafe{rd(list+0x58)};if !valid(entries){return None}
    let note=unsafe{byte(entries+selected)};
    let id=crate::generated::NATIVE_MAP.get(note as usize).copied()?;
    Some((list,pager,id,note,selected as usize))
}
fn refresh_extra_title(base:u32,owner:u32){
    let Some((_,pager,id,note,_))=selected(base,owner)else{return};
    // The original refresh returns before updating its native title on page 4.
    // Replay only the same name-lookup and native text-node update used on pages 1-3.
    if !crate::layout::supports_monster(id)||unsafe{virtual_get(base,pager,0x24)}.unwrap_or(0)<=2{return}
    let manager=unsafe{rd(base+0x1899220)};if !valid(manager){return}
    let name_index=unsafe{rd(base+0x162d9f0+note as u32*16)};
    let lookup:unsafe extern "C" fn(u32,u32,u32)->u32=unsafe{core::mem::transmute(base+0x55bc4c)};
    let set_text:unsafe extern "C" fn(u32,u32,u32,u32)->u32=unsafe{core::mem::transmute(base+0x56e808)};
    let text=unsafe{lookup(manager,3,name_index)};
    if text>=0x10000{unsafe{set_text(owner,1,2,text)};}
}
pub struct Snapshot{pub id:u16,pub note:u8,pub row:usize,pub pages:u8,pub extra:bool,pub notes:[u16;123],pub len:usize}
pub fn snapshot(base:u32)->Option<Snapshot>{
    let owner=OWNER.load(Ordering::Acquire);
    if !valid(owner)||unsafe{byte(owner+0x18)}!=1{return None}
    let (list,pager,id,note,row)=selected(base,owner)?;
    let page=unsafe{virtual_get(base,pager,0x24)}.unwrap_or(u32::MAX);
    let count=unsafe{byte(owner+0x50)};let key=(note as u32)<<16|page;
    if LAST.swap(key,Ordering::AcqRel)!=key{
        info!("[monster-info] note={} db={} page={}/{} owner={:08x}",note,id,page,count,owner);
    }
    let pages=COUNT.load(Ordering::Acquire) as u8;
    let len=unsafe{byte(list+0x50)} as usize;if len>123{return None}
    let entries=unsafe{rd(list+0x58)};let mut notes=[0;123];
    for (i,item) in notes.iter_mut().enumerate().take(len){*item=crate::generated::NATIVE_MAP.get(unsafe{byte(entries+i as u32)} as usize).copied().unwrap_or(0);}
    Some(Snapshot{id,note,row,pages,extra:crate::layout::supports_monster(id)&&count==pages+1&&page==pages as u32,notes,len})
}
pub fn progress(base:u32)->crate::model::Progress{
    let mut result=crate::model::Progress::EMPTY;
    let flags=unsafe{rd(base+0x1884c58)};let quests=unsafe{rd(base+0x1897f78)};
    let player=unsafe{rd(base+0x1885b58)};let village=unsafe{rd(base+0x18979f0)};
    if ![flags,quests,player,village].iter().all(|&p|valid(p)){return result}
    for (i,word) in result.flags.iter_mut().enumerate(){*word=unsafe{rd(flags+0x500+i as u32*4)};}
    for (i,bit) in result.cleared.iter_mut().enumerate(){*bit=unsafe{byte(quests+0x42c+i as u32)};}
    result.hr=unsafe{ptr::read_unaligned((player+0x5d0) as *const u16)};
    result.hub=unsafe{ptr::read_unaligned((village+0x3e2) as *const u16)};
    // sQuest + 0x72c: 137 independent saved u16 kill counts, then capture
    // counts at +0x112 (0x526d94 / 0x526e00; code -> index at 0x1597ea4).
    // These six entries are distinct from their parents; never use total parent hunts.
    for(i,&(_,_,index))in crate::generated::SPECIAL_RECORDS.iter().enumerate(){
        let slot=quests+0x72c+index as u32*2;
        let killed=unsafe{ptr::read_unaligned(slot as *const u16)};
        let captured=unsafe{ptr::read_unaligned((slot+0x112) as *const u16)};
        if killed<=9999&&captured<=9999&&(killed>0||captured>0){result.special_hunted|=1<<i;}
    }
    result.valid=result.hr<=999&&result.hub<=20;result
}
pub fn hunt_progress(base:u32,p:&mut crate::model::Progress){
    if !p.valid{return}
    let manager=unsafe{rd(base+0x185a270)};
    if valid(manager){let data=manager+0xb8;if valid(data){
        let banks:[[u32;3];3]=core::array::from_fn(|rank|core::array::from_fn(|word|unsafe{rd(data+0x108+rank as u32*12+word as u32*4)}));
        for(id,&code)in crate::generated::HUNT_CODES.iter().enumerate(){if code>0&&code<=88{for(rank,bank)in banks.iter().enumerate(){if bank[code as usize/32]&(1<<(code%32))!=0{p.hunted[id]|=1<<(rank+1);}}}}
    }}
    for &(id,rank,index)in &crate::generated::HUNT_PROOFS{let bit=index as usize;if bit<2048&&p.cleared[bit/8]&(1<<(bit%8))!=0{p.hunted[id as usize]|=1<<rank;}}
    // A separately recorded individual with exactly one possible material grade
    // has unambiguous historical grade evidence even without its quest clear.
    for &(id,_,_)in &crate::generated::SPECIAL_RECORDS{if p.hunted_special(id){let m=&crate::generated::MONSTERS[id as usize];let rows=&crate::generated::MATERIALS[m.material as usize..(m.material+m.material_len) as usize];if let Some(first)=rows.first(){if rows.iter().all(|r|r.rank==first.rank){p.hunted[id as usize]|=1<<first.rank;}}}}
    crate::history::merge(p);
}
pub fn character_key(base:u32)->Option<u64>{
    // sGuildCard's saved owner ID, data offset +0x8b0 plus its +0x20 header.
    let card=unsafe{rd(base+0x1857668)};if !valid(card){return None}
    let id=unsafe{rd(card+0x8d0)} as u64|((unsafe{rd(card+0x8d4)} as u64)<<32);
    (id!=0&&id!=u64::MAX).then_some(id)
}
pub fn stock(base:u32,item:u16)->Option<u32>{
    let box_ptr=unsafe{rd(base+0x1858438)};let pouch=unsafe{rd(base+0x18584b8)};
    if item==0||!valid(box_ptr)||!valid(pouch){return None}
    let mut total=0;
    for (p,count) in [(box_ptr,2300),(pouch,50)]{for i in 0..count{
        let id=unsafe{ptr::read_unaligned((p+0x1a+i*4) as *const u16)};
        let quantity=unsafe{ptr::read_unaligned((p+0x1c+i*4) as *const u16)};
        if id>=2992||quantity>99{return None}if id==item{total+=quantity as u32}
    }}Some(total)
}
pub fn play_ui(base:u32,id:u32){unsafe{
    let entry=base+0x4eeb78;let words:[u32;4]=core::array::from_fn(|i|rd(entry+i as u32*4));
    let original=[0xe1a02000,0xe59200c0,0xe3500000,0x012fff1e];
    let chained=words[0]==0xe51ff004&&valid(words[1])&&words[1]!=entry&&words[2..]==original[2..];
    if words!=original&&!chained{return}
    let manager=rd(base+0x1896f5c);if !valid(manager){return}
    let play:unsafe extern "C" fn(u32,u32)=core::mem::transmute(entry);play(manager,id);
}}

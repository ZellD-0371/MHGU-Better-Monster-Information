//! Owned, validated single-language packs. Only commit can replace the active bytes.
use alloc::{vec::Vec,string::String,rc::Rc};
mod manifest{include!("language_assets.rs");}
pub use manifest::{COUNT,FILES,FONT};
pub fn crc_step(mut crc:u32,data:&[u8])->u32{const T:[u32;16]=[0,0x1db71064,0x3b6e20c8,0x26d930ac,0x76dc4190,0x6b6b51f4,0x4db26158,0x5005713c,0xedb88320,0xf00f9344,0xd6d6a3e8,0xcb61b38c,0x9b64c2b0,0x86d3d2d4,0xa00ae278,0xbdbdf21c];for b in data{crc^=*b as u32;crc=(crc>>4)^T[(crc&15) as usize];crc=(crc>>4)^T[(crc&15) as usize];}crc}
fn word(b:&[u8],at:usize)->Option<u32>{Some(u32::from_le_bytes(b.get(at..at+4)?.try_into().ok()?))}
pub struct Pack{pub language:usize,bytes:Vec<u8>}
impl Pack{
 pub fn prepare(language:usize,bytes:Vec<u8>)->Result<Self,()>{
  let(size,crc)=*FILES.get(language).ok_or(())?;
  if bytes.len()!=size||!crc_step(!0,&bytes)!=crc||bytes.get(..8)!=Some(b"BMILANG1")||word(&bytes,8)!=Some(COUNT as u32)||word(&bytes,12)!=Some(language as u32){return Err(())}
  let start=16+COUNT*4;let pool=bytes.get(start..).ok_or(())?;let pool_text=core::str::from_utf8(pool).map_err(|_|())?;
  for i in 0..COUNT{let at=word(&bytes,16+i*4).ok_or(())? as usize;if at<start||at>=bytes.len()||!pool_text.is_char_boundary(at-start)||!bytes[at..].contains(&0){return Err(())}}
  Ok(Self{language,bytes})
 }
 pub fn len(&self)->usize{self.bytes.len()}
 pub fn view(&self,id:u32)->&str{if id as usize>=COUNT{return "—"}let at=word(&self.bytes,16+id as usize*4).unwrap() as usize;let rest=&self.bytes[at..];let end=rest.iter().position(|&b|b==0).unwrap();core::str::from_utf8(&rest[..end]).unwrap()}
 pub fn text(&self,id:u32)->String{self.view(id).into()}
}
/// Bounded owned strings: safe across pack swaps, no repeat allocation on cache hits.
#[derive(Default)]pub struct TextCache{rows:Vec<(u32,Rc<str>)>,bytes:usize}
impl TextCache{
 pub const EMPTY:Self=Self{rows:Vec::new(),bytes:0};
 pub fn get(&mut self,pack:&Pack,id:u32)->Rc<str>{
  if let Some((_,text))=self.rows.iter().find(|(key,_)|*key==id){return Rc::clone(text)}
  let text:Rc<str>=Rc::from(pack.view(id));
  if text.len()<=16384{while self.rows.len()>=64||self.bytes+text.len()>16384{let(_,old)=self.rows.remove(0);self.bytes-=old.len();}self.bytes+=text.len();self.rows.push((id,Rc::clone(&text)));}text
 }
 #[cfg(test)]pub fn usage(&self)->(usize,usize){(self.rows.len(),self.bytes)}
}
#[derive(Default)]pub struct Store{pub active:Option<Pack>,cache:TextCache}
impl Store{
 pub const EMPTY:Self=Self{active:None,cache:TextCache::EMPTY};
 pub fn commit(&mut self,next:Pack)->Option<Pack>{self.cache=TextCache::EMPTY;self.active.replace(next)}
 pub fn reset(&mut self)->Option<Pack>{self.cache=TextCache::EMPTY;self.active.take()}
 pub fn text(&mut self,id:u32)->Rc<str>{self.active.as_ref().map_or_else(||Rc::from("—"),|p|self.cache.get(p,id))}
}

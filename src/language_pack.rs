//! Owned, validated single-language packs. Only commit can replace the active bytes.
use alloc::{vec::Vec,string::String};
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
 pub fn text(&self,id:u32)->String{if id as usize>=COUNT{return "—".into()}let at=word(&self.bytes,16+id as usize*4).unwrap() as usize;let rest=&self.bytes[at..];let end=rest.iter().position(|&b|b==0).unwrap();core::str::from_utf8(&rest[..end]).unwrap().into()}
}
#[derive(Default)]pub struct Store{pub active:Option<Pack>}
impl Store{pub fn commit(&mut self,next:Pack)->Option<Pack>{self.active.replace(next)}}

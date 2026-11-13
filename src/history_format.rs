use alloc::{format,string::String};
pub fn decode(text:&str,key:u64)->Option<[u16;130]>{
 let mut result=[0;130];let mut identity=false;
 for line in text.lines(){let Some((k,v))=line.split_once('=')else{continue};
  if k=="identity"{identity=u64::from_str_radix(v,16).ok()==Some(key);continue}
  if let(Ok(id),Ok(mask))=(k.parse::<usize>(),v.parse::<u16>()){if id>0&&id<130&&mask&!0x1ffe==0{result[id]=mask}else{return None}}
 }
 identity.then_some(result)
}
pub fn encode(key:u64,masks:&[u16;130])->String{
 let mut text=format!("[BetterMonsterInformationHunts1]\nidentity={:016x}\n",key);
 for(id,mask)in masks.iter().enumerate().skip(1){if *mask!=0{text.push_str(&format!("{}={}\n",id,mask));}}text
}
pub fn event_mask(id:u16,rank:u8,quest:u32)->u16{
 let Some(m)=crate::generated::MONSTERS.get(id as usize)else{return 0};
 if m.deviant<0{return if (1..=3).contains(&rank){1<<rank}else{0}}
 let Some(level)=quest.checked_sub(40100+m.deviant as u32*100)else{return 0};
 if !(1..=16).contains(&level){return 0}
 let mut mask=0;for row in &crate::generated::MATERIALS[m.material as usize..(m.material+m.material_len) as usize]{let(lo,hi)=crate::model::RANK_RANGES[row.rank as usize];if level>=lo&&level<=hi{mask|=1<<row.rank;}}mask
}

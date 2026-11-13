//! Borrowed static labels for known database values; formatting only for new values.
use alloc::{borrow::Cow,string::String,format};
mod tables{include!("numeric_generated.rs");}
pub fn number(v:f32)->Cow<'static,str>{match tables::NUMBERS.binary_search_by_key(&v.to_bits(),|&(bits,_)|bits){Ok(i)=>Cow::Borrowed(tables::NUMBERS[i].1),Err(_)=>Cow::Owned(format_number(v))}}
pub fn percent(v:i16)->Cow<'static,str>{match tables::PERCENTS.binary_search_by_key(&v,|&(value,_)|value){Ok(i)=>Cow::Borrowed(tables::PERCENTS[i].1),Err(_)=>Cow::Owned(format_percent(v))}}
pub fn format_number(v:f32)->String{if v<0.{String::from("—")}else if v==v as u32 as f32{format!("{}",v as u32)}else{let scaled=(v*100000.+0.5) as u32;let fraction=scaled%100000;let mut tail=format!("{}",fraction);while tail.len()<5{tail.insert(0,'0');}let mut out=format!("{}.{}",scaled/100000,tail);while out.ends_with('0'){out.pop();}out}}
pub fn format_percent(v:i16)->String{if v==-1{String::from("—")}else if v==-2{String::from("…")}else if v%100==0{format!("{}%",v/100)}else{format!("{}.{}{}%",v/100,(v%100)/10,v%10)}}

extern crate alloc;
#[path="../src/numeric.rs"]mod numeric;
#[path="../src/retry.rs"]mod retry;
#[path="../src/diagram_lifetime.rs"]mod diagram_lifetime;
#[path="../src/settings_labels.rs"]mod settings_labels;

#[test]fn logging_defaults_off_and_old_settings_remain_compatible(){
 let mut s=State::new();assert!(!s.logging);s.load_preferences("[BetterMonsterInformation]\nlanguage=4\nlayout=2\n");
 assert_eq!((s.language,s.layout,s.logging),(4,2,false));
 s.load_preferences("logging=2\nlogging=-1\nlogging=true\nlanguage=9\nlayout=3\n");assert_eq!((s.language,s.layout,s.logging),(4,2,false));
 for lang in 0..9{for layout in 0..3{for enabled in [false,true]{s.language=lang;s.layout=layout;s.logging=enabled;let mut restored=State::new();restored.load_preferences(&s.encode_preferences());assert_eq!((restored.language,restored.layout,restored.logging),(lang,layout,enabled));}}}
}
#[test]fn layout_logging_is_opt_in_and_resumes_with_current_values(){
 let mut log=layout::ViewportLog::NEW;let key=[1280,720,1,1,1,1280,720,1280];
 for _ in 0..600{assert!(!log.record(false,key));}
 assert!(log.record(true,key));assert!(!log.record(true,key));assert!(!log.record(false,key));assert!(log.record(true,key));
 let mut next=key;next[0]=1920;assert!(!log.record(false,next));assert!(log.record(true,next));
}
#[test]fn logging_checkbox_has_all_nine_translations(){
 let labels=settings_labels::LOGGING.map(|s|s.to_str().unwrap());assert_eq!(labels[0],"Enable logging");assert_eq!(labels[4],"로그 기록 활성화");
 for(i,label)in labels.iter().enumerate(){assert!(!label.is_empty());assert!(!labels[..i].contains(label));}
}

#[test]fn handheld_and_docked_offscreen_hud_fall_back_to_screen(){
 use layout::{resolve_viewport,viewport_fits,Placement};
 for(w,h)in [(1280.,720.),(1920.,1080.)]{
  let(normal,p)=resolve_viewport(w,h,Some((w as u32,h as u32,1280))).unwrap();assert_eq!(p,Placement::Hud);
  for uw in [1920,2560,7200]{
   assert!(layout::hud_viewport(w,h,w as u32,h as u32,uw).is_none());
   let(v,p)=resolve_viewport(w,h,Some((w as u32,h as u32,uw))).unwrap();assert_eq!(p,Placement::InvalidHud);assert!(viewport_fits(v,w,h));
   assert_eq!((v.scale,v.ox,v.right_ox,v.oy),(normal.scale,normal.ox,normal.right_ox,normal.oy));
  }
 }
}
#[test]fn viewport_safety_preserves_center_edges_and_fixed_aspect(){
 use layout::{resolve_viewport,viewport_fits,Placement};
 for(w,h,rw,rh,uw,scale,left,right,top)in [
  (1680.,720.,1680,720,1680,1.,0.,400.,0.),
  (1680.,720.,1280,720,1280,1.,200.,200.,0.),
  (3440.,1440.,3440,1440,1720,2.,0.,880.,0.),
  (1792.,504.,1792,504,2560,0.7,0.,896.,0.),
  (1440.,900.,1440,900,1280,1.125,0.,0.,0.),
  (1920.,1200.,1920,1200,1280,1.5,0.,0.,0.),
  (1920.,1080.,1280,1024,1280,1.0546875,285.,285.,0.),
 ]{
  let(v,p)=resolve_viewport(w,h,Some((rw,rh,uw))).unwrap();assert_eq!(p,Placement::Hud);assert!(viewport_fits(v,w,h));
  for(a,b)in [(v.scale,scale),(v.ox,left),(v.right_ox,right),(v.oy,top)]{assert!((a-b).abs()<0.01,"{a} != {b}");}
 }
}
#[test]fn viewport_safety_handles_invalid_inputs_and_all_fallbacks(){
 use layout::{resolve_viewport,viewport_fits,Placement};
 for(w,h)in [(1280.,720.),(1920.,1080.),(3840.,2160.),(3440.,1440.),(1440.,900.),(1024.,768.),(640.,360.)]{
  for hud in [None,Some((0,720,1280)),Some((1280,0,1280)),Some((u32::MAX,720,1280)),Some((1280,720,u32::MAX)),Some((1280,720,0))]{
   let(v,p)=resolve_viewport(w,h,hud).unwrap();assert_ne!(p,Placement::Hud);assert!(viewport_fits(v,w,h));
  }
 }
 for(w,h)in [(0.,720.),(1280.,0.),(-1280.,720.),(f32::NAN,720.),(1280.,f32::INFINITY)]{assert!(resolve_viewport(w,h,Some((1280,720,1280))).is_none());}
}
#[test]fn viewport_logs_once_per_input_change_and_visible_session(){
 let mut log=layout::ViewportLog::NEW;let key=[1280f32.to_bits(),720f32.to_bits(),1f32.to_bits(),1f32.to_bits(),1,1280,720,1280];
 assert!(log.changed(key));for _ in 0..600{assert!(!log.changed(key));}
 let mut bad=key;bad[7]=1920;assert!(log.changed(bad));assert!(!log.changed(bad));
 log.reset();assert!(log.changed(bad));assert!(!log.changed(bad));
}

#[test]fn diagram_releases_on_close_only_after_backend_completion(){
 use diagram_lifetime::{Lifetime,Status as S,Action as A};let mut life=Lifetime::NEW;
 assert_eq!(life.advance(false,None),A::Keep);
 for _ in 0..300{assert_eq!(life.advance(true,Some(S::Ready)),A::Keep);}
 assert_eq!(life.advance(false,Some(S::Ready)),A::RequestDestroy);
 assert!(life.retiring());
 // Reopening cannot cancel an in-flight destruction or reuse that texture.
 assert_eq!(life.advance(true,Some(S::Busy)),A::WaitForDestroy(2));
 assert_eq!(life.advance(true,Some(S::Destroyed)),A::Free);assert!(!life.retiring());
 assert_eq!(life.advance(true,Some(S::Ready)),A::Keep);
}
#[test]fn diagram_waits_for_upload_and_handles_repeated_closes(){
 use diagram_lifetime::{Lifetime,Status as S,Action as A};let mut life=Lifetime::NEW;
 for _ in 0..300{assert_eq!(life.advance(false,Some(S::Busy)),A::Keep);}
 // An upload completing after a quick reopen must remain usable.
 assert_eq!(life.advance(true,Some(S::Ready)),A::Keep);
 for _ in 0..1000{
  assert_eq!(life.advance(false,Some(S::Ready)),A::RequestDestroy);
  assert_eq!(life.advance(false,Some(S::Busy)),A::WaitForDestroy(2));
  assert_eq!(life.advance(false,Some(S::Destroyed)),A::Free);
  assert_eq!(life.advance(false,None),A::Keep);
 }
}
#[path="../src/language_pack.rs"]mod language_pack;
#[path="../src/layout.rs"]mod layout;
#[path="../src/data.rs"]mod data;
#[cfg(not(bmi_low_memory))]#[path="../src/generated.rs"]mod generated;
#[cfg(bmi_low_memory)]#[path="../src/generated_low.rs"]mod generated;
#[path="../src/model.rs"]mod model;
#[path="../src/history_format.rs"]mod history_format;
mod resources{
 pub fn text(id:u32,lang:usize)->&'static str{
  let bytes=include_bytes!("../assets/native/monster-text.bin");let off=12+id as usize*36+lang.min(8)*4;
  let start=u32::from_le_bytes(bytes[off..off+4].try_into().unwrap()) as usize;
  let rest=&bytes[start..];let len=rest.iter().position(|&c|c==0).unwrap();core::str::from_utf8(&rest[..len]).unwrap()
 }
}
use model::*;
fn high()->Progress{let mut p=Progress::EMPTY;p.valid=true;p.hr=2;p.hub=2;p.flags[1008/32]|=1<<(1008%32);p.hunted.fill(0x1ffe);p}
#[test]fn ordinary_unlocks(){let p=high();assert!(rank_open(48,1,&p));assert!(rank_open(48,2,&p));assert!(!rank_open(48,3,&p));let mut p=p;p.flags[1401/32]|=1<<(1401%32);assert!(rank_open(48,3,&p));}
#[test]fn highest_rank_and_reset(){let mut s=State::new();s.progress=high();s.recompute(true);assert_eq!(s.rank,2);s.tab=3;s.action(RANK);assert_eq!(s.mode,2);s.action(UP);s.action(ACCEPT);assert_eq!(s.rank,1);assert_eq!(s.row,0);assert_eq!(s.mode,0);}
#[test]fn table_owns_navigation_and_back(){let mut s=State::new();s.progress=high();s.recompute(true);s.action(ACCEPT);assert_eq!(s.mode,1);s.action(NEXT|RANK);assert_eq!(s.tab,0);assert_eq!(s.mode,1);s.action(UP);assert_eq!(s.row,s.rows()-1);s.action(DOWN);assert_eq!(s.row,0);s.action(BACK);assert_eq!(s.mode,0);}
#[test]fn material_one_accept(){let mut s=State::new();s.progress=high();s.recompute(true);s.tab=3;s.action(ACCEPT);assert_eq!(s.mode,1);s.action(ACCEPT);assert_eq!(s.mode,1);s.action(BACK);assert_eq!(s.mode,0);}
#[test]fn readonly_tabs(){let mut s=State::new();for tab in [1,2]{s.tab=tab;s.action(ACCEPT|DOWN);assert_eq!(s.mode,0);assert_eq!(s.row,0);}}
#[test]fn a_short_direction_press_does_not_repeat(){let mut s=State::new();s.action(ACCEPT);s.action(UP);let row=s.row;for _ in 0..6{s.tick(1./60.,UP);}assert_eq!(s.row,row);for _ in 0..20{s.tick(1./60.,UP);}assert_eq!(s.row,(row+s.rows()-1)%s.rows());}
#[test]fn rank_cancel_keeps_selection(){let mut s=State::new();s.progress=high();s.recompute(true);s.tab=3;s.action(RANK);s.action(UP);s.action(BACK);assert_eq!(s.rank,2);assert_eq!(s.mode,0);}
#[test]fn data_references_valid(){use generated as g;for m in &g::MONSTERS{assert!(m.weak as usize+m.weak_len as usize<=g::WEAK.len());assert!(m.material as usize+m.material_len as usize<=g::MATERIALS.len());}for r in &g::WEAK{assert!(r.ring as usize+r.ring_len as usize<=g::RINGS.len());}for m in &g::MATERIALS{assert!((m.item as usize)<g::ITEMS.len());assert!(m.entry as usize+m.entry_len as usize<=g::ACQUISITIONS.len());}assert_eq!(g::NATIVE_MAP.iter().filter(|&&id|id>=37).count(),87);}
#[test]fn all_localized_strings_present(){let bytes=include_bytes!("../assets/native/monster-text.bin");let count=u32::from_le_bytes(bytes[8..12].try_into().unwrap());for id in 0..count{for lang in 0..9{assert!(!resources::text(id,lang).is_empty());}}}
#[test]fn every_native_monster_including_small_has_a_page(){for &id in &generated::NATIVE_MAP{assert!(layout::supports_monster(id));let m=generated::MONSTERS[id as usize];assert!(m.weak_len>0&&m.material_len>0);}assert_eq!(generated::NATIVE_MAP.iter().filter(|&&id|id<37).count(),36);assert!(!layout::supports_monster(0));assert!(!layout::supports_monster(130));}
#[test]fn every_db_monster_is_reachable_from_native_notes_or_family(){let mut covered=generated::NATIVE_MAP.to_vec();for &(parent,id)in &generated::FAMILIES{assert!(generated::NATIVE_MAP.contains(&parent));covered.push(id);}for id in 1..130{assert!(covered.contains(&id),"unreachable DB {id}");}}
#[test]fn form_select_cancel_commit_and_native_refresh(){let mut s=State::new();s.progress=high();s.progress.special_hunted=63;s.select(64,0,3);s.action(FORM);assert_eq!(s.mode,3);s.action(DOWN);s.action(BACK);assert_eq!(s.id,64);s.action(FORM);s.action(DOWN);s.action(ACCEPT);assert_eq!(s.id,65);assert_eq!(s.mode,0);s.select(64,0,3);assert_eq!(s.id,65);s.action(ACCEPT);s.action(FORM|NEXT|RANK);assert_eq!(s.id,65);assert_eq!(s.mode,1);s.select(83,1,3);assert_eq!(s.id,83);assert_eq!(s.mode,0);s.reset_form();assert_eq!(s.id,s.native_id);}
#[test]fn form_list_hides_unopened_ranks_and_owns_all_navigation(){let mut s=State::new();s.select(127,0,3);assert_eq!(s.forms(),[127]);s.progress=high();s.progress.flags[1401/32]|=1<<(1401%32);assert_eq!(s.forms(),[127]);s.progress.special_hunted=63;assert_eq!(s.forms(),[127,128,129]);s.action(FORM);s.action(NEXT|RANK);assert_eq!(s.mode,3);assert_eq!(s.tab,0);s.action(UP);s.action(ACCEPT);assert_eq!(s.id,129);assert_eq!(s.rank,3);s.row=2;s.scroll=1;s.reset_form();assert_eq!(s.id,127);}
#[test]fn special_hunts_are_independent_and_missing_records_fail_closed(){let mut s=State::new();s.progress=high();s.progress.flags[1401/32]|=1<<(1401%32);for(i,&(id,_,_))in generated::SPECIAL_RECORDS.iter().enumerate(){s.progress.special_hunted=1<<i;for(j,&(other,_,_))in generated::SPECIAL_RECORDS.iter().enumerate(){assert_eq!(s.progress.hunted_special(other),i==j);}let parent=generated::FAMILIES.iter().find(|&&(_,child)|child==id).unwrap().0;s.select(parent,0,3);assert!(s.forms().contains(&id));s.progress.valid=false;assert_eq!(s.forms(),[parent]);s.progress.valid=true;}s.progress.special_hunted=0;s.select(64,0,3);s.action(FORM);assert_eq!(s.mode,0);}
#[test]fn resolution_matrix_matches_native_hud_without_distorting(){for(w,h)in [(1280.,720.),(1920.,1080.),(2560.,1440.),(3840.,2160.),(1440.,900.),(1920.,1200.),(2560.,1600.),(1680.,720.),(3440.,1440.),(1792.,504.),(5120.,1440.)]{let v=layout::native_viewport(w,h).unwrap();assert!(v.scale>0.);assert!(v.ox>=0.&&v.oy>=0.);assert!(v.ox+1270.*v.scale<=w+0.01);assert!(v.oy+702.*v.scale<=h+0.01);if w/h<16./9.{assert_eq!(v.oy,0.);assert_eq!(v.scale,w/1280.);}else{assert!((v.ox+640.*v.scale-w*0.5).abs()<0.01);assert!((v.oy+360.*v.scale-h*0.5).abs()<0.01);}}for(w,h)in [(-1.,720.),(0.,720.),(1280.,0.),(f32::NAN,720.),(f32::INFINITY,720.)]{assert!(layout::native_viewport(w,h).is_none());}}
#[test]fn cleared_bitmap_uses_quest_index(){let mut p=high();let(q,index)=generated::QUESTS[20];assert!(!p.cleared(q));p.cleared[index as usize/8]|=1<<(index as usize%8);assert!(p.cleared(q));assert!(rule_open(&format!("flag:1008 AND cleared:{}",q),&p));assert!(!rule_open("flag:1401 AND hr:8",&p));}
#[test]fn deviant_all_access_when_all_conditions_met(){let mut p=high();p.flags.fill(u32::MAX);p.cleared.fill(255);p.hr=999;p.hub=20;for (id,m)in generated::MONSTERS.iter().enumerate(){if m.deviant>=0{for rank in generated::MATERIALS[m.material as usize..(m.material+m.material_len) as usize].iter().map(|r|r.rank){assert!(rank_open(id as u16,rank,&p),"monster {id}, rank {rank}");}}}}
#[test]fn scrollbars_show_overflow_and_reach_both_ends(){assert!(layout::scrollbar(8,8,0,272.).is_none());let(first,h)=layout::scrollbar(24,8,0,272.).unwrap();let(last,last_h)=layout::scrollbar(24,8,16,272.).unwrap();assert_eq!(first,0.);assert_eq!(h,last_h);assert!((last+h-272.).abs()<0.001);assert!(layout::scrollbar(24,8,8,272.).unwrap().0>first);assert!(layout::scrollbar(240,8,0,272.).unwrap().1>=18.);}
#[test]fn native_diagrams_preserve_full_source_resolution(){use generated as g;assert_eq!(g::DIAGRAM_SOURCES.iter().filter(|&&(w,_)|w>0).count(),130);for(i,&(w,h))in g::DIAGRAM_SOURCES.iter().enumerate(){if w==0{continue}assert!(w as usize<=g::DIAGRAM_TEX_W&&h as usize<=g::DIAGRAM_TEX_H);assert!(w>=g::DIAGRAMS[i].0&&h>=g::DIAGRAMS[i].1);}if cfg!(bmi_low_memory){assert_eq!((g::DIAGRAM_TEX_W,g::DIAGRAM_TEX_H),(384,256));}else{assert!(g::DIAGRAM_TEX_W>=1000);}}

#[test]fn rank_history_is_character_isolated_and_events_keep_actual_rank(){let mut masks=[0;130];masks[48]=1<<2;let text=history_format::encode(0x12345678,&masks);assert_eq!(history_format::decode(&text,0x12345678),Some(masks));assert!(history_format::decode(&text,0x87654321).is_none());assert_eq!(history_format::event_mask(48,3,11401),1<<3);assert_eq!(history_format::event_mask(48,0,11401),0);}
#[test]fn unlocked_ranks_no_longer_require_a_hunt(){let mut p=high();p.hunted.fill(0);assert!(!REQUIRE_RANK_HUNT);assert!(rank_open(48,1,&p));assert!(rank_open(48,2,&p));assert!(!rank_open(48,3,&p));let mut s=State::new();s.progress=p;s.recompute(true);assert_eq!(s.rank,2);assert_eq!(s.ranks,[1,2]);s.progress.flags[1401/32]|=1<<(1401%32);s.recompute(true);assert_eq!(s.rank,3);assert_eq!(s.ranks,[1,2,3]);s.progress.valid=false;s.recompute(true);assert!(s.ranks.is_empty());}
#[test]fn edge_hud_windows_anchor_independently_with_uniform_scale(){for(w,h,uw)in [(1680.,720.,1680),(1792.,504.,2560),(5120.,1440.,2560)]{let v=layout::hud_viewport(w,h,w as u32,h as u32,uw).unwrap();assert!(v.ox.abs()<0.01);assert!((v.right_ox+1280.*v.scale-w).abs()<0.01);}let v=layout::hud_viewport(1680.,720.,1280,720,1280).unwrap();assert_eq!(v.ox,200.);assert_eq!(v.ox,v.right_ox);}

#[test]fn deviant_interval_hunts_disabled_but_progression_kept(){let mut p=high();p.hunted.fill(0);p.flags.fill(u32::MAX);p.cleared.fill(255);for q in 40101..=40103{let index=generated::QUESTS.iter().find(|&&(id,_)|id==q).unwrap().1 as usize;p.cleared[index/8]&=!(1<<(index%8));}assert!(rank_open(60,4,&p));for rank in 4..=12{assert_eq!(rank_open(60,rank,&p),rank_unlocked(60,rank,&p));}assert!(!rank_open(60,12,&Progress::EMPTY));}
#[test]fn monster_change_preserves_added_page_across_different_page_counts(){assert_eq!(layout::extra_page_on_monster_change(true,0,34,3,4),Some(3));assert_eq!(layout::extra_page_on_monster_change(true,34,0,2,3),Some(2));assert_eq!(layout::extra_page_on_monster_change(true,34,35,3,4),Some(3));assert_eq!(layout::extra_page_on_monster_change(false,0,34,3,4),None);assert_eq!(layout::extra_page_on_monster_change(true,34,34,3,4),None);assert_eq!(layout::extra_page_on_monster_change(true,0,34,3,3),None);}

#[test]fn atlas_variant_size_and_sprite_bounds(){let p=if cfg!(bmi_low_memory){"assets/native-low/monster-icons.rgba"}else{"assets/native/monster-icons.rgba"};assert_eq!(std::fs::metadata(p).unwrap().len(),(2048*generated::ATLAS_HEIGHT*4) as u64);if cfg!(bmi_low_memory){assert!(generated::ATLAS_HEIGHT<=256);}for s in &generated::SPRITES{assert!(s.w>0&&s.h>0&&s.x as usize+s.w as usize<=2048&&s.y as usize+s.h as usize<=generated::ATLAS_HEIGHT);if cfg!(bmi_low_memory){assert!(s.w<=32&&s.h<=32);}}}

#[test]fn single_language_packs_match_every_multilingual_record(){
 let codes=["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"];
 for(l,code)in codes.iter().enumerate(){let bytes=std::fs::read(format!("assets/languages/text.{}.bin",code)).unwrap();let pack=language_pack::Pack::prepare(l,bytes).ok().unwrap();assert_eq!(pack.language,l);for id in 0..language_pack::COUNT as u32{assert_eq!(pack.text(id),resources::text(id,l));}}
}
#[test]fn language_swap_releases_old_pack_and_preserves_owned_strings(){
 let en=std::fs::read("assets/languages/text.en.bin").unwrap();let ko=std::fs::read("assets/languages/text.ko.bin").unwrap();let mut store=language_pack::Store::default();
 store.commit(language_pack::Pack::prepare(0,en).ok().unwrap());let text=store.active.as_ref().unwrap().text(generated::UI_NOTESINFO[0]);
 let old=store.commit(language_pack::Pack::prepare(4,ko.clone()).ok().unwrap()).unwrap();assert_eq!(old.language,0);drop(old);assert_eq!(store.active.as_ref().unwrap().language,4);assert!(text.contains("Hunter"));
 let mut broken=ko;broken[20]^=1;assert!(language_pack::Pack::prepare(4,broken).is_err());assert_eq!(store.active.as_ref().unwrap().language,4);assert!(language_pack::Pack::prepare(0,Vec::new()).is_err());
}
#[test]fn shared_subset_font_integrity(){let f=std::fs::read("assets/languages/font.shared.otf").unwrap();assert_eq!(f.len(),language_pack::FONT.0);assert_eq!(!language_pack::crc_step(!0,&f),language_pack::FONT.1);assert!(f.len()<2*1024*1024);assert_eq!(&f[..4],b"OTTO");}

#[test]fn every_diagram_file_matches_variant_dimensions(){let dir=if cfg!(bmi_low_memory){"assets/native-low/parts"}else{"assets/native/parts"};let mut count=0;for(id,&(w,h))in generated::DIAGRAM_SOURCES.iter().enumerate(){if w==0{continue}count+=1;assert_eq!(std::fs::metadata(format!("{}/{}.rgba",dir,id)).unwrap().len(),w as u64*h as u64*4);assert!(w as usize<=generated::DIAGRAM_TEX_W&&h as usize<=generated::DIAGRAM_TEX_H);}assert_eq!(count,130);}

#[test]fn fitting_prefix_is_linear_and_keeps_unicode_boundaries(){let s="머리 · Broken";let mut calls=0;let n=layout::fitting_prefix(s,3.,|_|{calls+=1;1.});assert_eq!(&s[..n],"머리 ");assert_eq!(calls,4);assert_eq!(layout::fitting_prefix(s,0.,|_|1.),0);assert_eq!(layout::fitting_prefix(s,100.,|_|1.),s.len());}

#[test]fn text_cache_reuses_allocations_stays_bounded_and_survives_swap(){use alloc::rc::Rc;let bytes=std::fs::read("assets/languages/text.en.bin").unwrap();let pack=language_pack::Pack::prepare(0,bytes.clone()).ok().unwrap();let mut cache=language_pack::TextCache::EMPTY;let a=cache.get(&pack,generated::UI_NOTESINFO[0]);let b=cache.get(&pack,generated::UI_NOTESINFO[0]);assert!(Rc::ptr_eq(&a,&b));for i in 0..language_pack::COUNT as u32{cache.get(&pack,i);let(n,bytes)=cache.usage();assert!(n<=64&&bytes<=16384);}let mut store=language_pack::Store::default();store.commit(language_pack::Pack::prepare(0,bytes).ok().unwrap());let old=store.text(generated::UI_NOTESINFO[0]);store.commit(language_pack::Pack::prepare(4,std::fs::read("assets/languages/text.ko.bin").unwrap()).ok().unwrap());assert!(old.contains("Hunter"));assert!(store.text(generated::UI_NOTESINFO[0]).contains("헌터"));}

#[test]fn numeric_labels_match_old_format_without_allocating_for_all_db_values(){use alloc::borrow::Cow;for row in &generated::WEAK{for v in row.values{let text=numeric::number(v);assert!(matches!(text,Cow::Borrowed(_)));assert_eq!(text,numeric::format_number(v));}}for row in generated::AILMENTS.iter().chain(generated::TRAPS.iter()){for v in row.values{let text=numeric::number(v);assert!(matches!(text,Cow::Borrowed(_)));assert_eq!(text,numeric::format_number(v));}}for row in &generated::MATERIALS{for v in row.cells{let text=numeric::percent(v);assert!(matches!(text,Cow::Borrowed(_)));assert_eq!(text,numeric::format_percent(v));}}for row in &generated::ACQUISITIONS{let text=numeric::percent(row.percent as i16);assert!(matches!(text,Cow::Borrowed(_)));assert_eq!(text,numeric::format_percent(row.percent as i16));}}
#[test]fn unknown_numeric_values_keep_legacy_fallback(){assert_eq!(numeric::number(12345.125),numeric::format_number(12345.125));assert_eq!(numeric::percent(12345),numeric::format_percent(12345));assert_eq!(numeric::percent(-1),"—");assert_eq!(numeric::percent(-2),"…");}
#[test]fn failed_diagram_attempts_stop_and_can_be_reset(){let mut gate=retry::RetryGate::NEW;gate.select(41);let mut attempts=0;for _ in 0..3000{if gate.allows(){attempts+=1;gate.failed();}gate.tick(1./60.,true);}assert_eq!(attempts,3);assert!(!gate.allows());gate.select(45);assert!(gate.allows());gate.failed();assert!(!gate.allows());gate.tick(4.,true);assert!(!gate.allows());gate.tick(1.1,true);assert!(gate.allows());gate.failed();gate.succeeded();assert!(gate.allows());gate.failed();gate.tick(0.,false);gate.select(45);assert!(gate.allows());}

// Reported in game: Gypceros takes pitfall traps but not shock traps (DB Trap_ID 2 = shock, 3 = pitfall).
#[test]fn trap_rows_are_flash_pitfall_shock_with_known_immunities(){
 let rows=|id:usize|{let m=&generated::MONSTERS[id];&generated::TRAPS[m.trap as usize..(m.trap+m.trap_len) as usize]};
 let(pit,shock)=(rows(98)[1].name,rows(98)[2].name);assert_ne!(pit,shock);
 for id in 37..130{let r=rows(id);if r.len()>=3{assert_eq!((r[1].name,r[2].name),(pit,shock),"monster {id}");}}
 let g=rows(50);assert_eq!(g[1].values[0],1.);assert_ne!(g[2].values[0],1.); // Gypceros
 let n=rows(77);assert_ne!(n[1].values[0],1.);assert_eq!(n[2].values[0],1.); // Nibelsnarf
 // Stonefist Hermitaur pitfall: trapped only while enraged, 8 s (reviewed override of the DB's 12 / 2.5 / 25; not measured yet)
 assert_eq!(rows(72)[1].values[..4],[1.,-1.,8.,-1.]);assert_eq!(rows(71)[1].values[..4],[1.,12.,12.,25.]);
}
#[test]fn neset_parts_use_reviewed_rings_and_separate_picture(){let m=&generated::MONSTERS[39];for row in &generated::WEAK[m.weak as usize..(m.weak+m.weak_len) as usize]{if row.part>=8{assert!(row.ring_len>0);assert!(!row.pending);}}assert!(generated::DIAGRAM_SOURCES[130].0>0);}


#[path="../src/menu_input.rs"]mod menu_input;
#[path="../src/hook_guard.rs"]mod hook_guard;

#[path="../src/button_prompts.rs"]mod button_prompts;

#[path="../src/mouse.rs"]mod mouse;
#[test]fn mouse_maps_into_the_picture_and_hud(){
 use mouse::to_display;
 let close=|a:(f32,f32),b:(f32,f32)|(a.0-b.0).abs()<0.01&&(a.1-b.1).abs()<0.01;
 // 16:9 window, 16:9 picture: plain scaling
 let v=View{w:1920.,h:1080.,hud:Some((1920,1080,1280))};assert!(close(to_display(640.,360.,v),(960.,540.)));assert!(close(to_display(0.,0.,v),(0.,0.)));
 // 16:10 forced in a 16:9 window: picture 1728x1080 centered, bars of 96 px
 let v=View{w:1920.,h:1080.,hud:Some((1728,1080,1280))};assert!(close(to_display(0.,0.,v),(96.,0.)));assert!(close(to_display(1280.,720.,v),(1824.,1080.)));
 // 21:9 picture with a 16:9 HUD: x covers the HUD's middle part only
 let v=View{w:2560.,h:1080.,hud:Some((2560,1080,1280))};assert!(close(to_display(0.,360.,v),(320.,540.)));assert!(close(to_display(1280.,360.,v),(2240.,540.)));
 // Without renderer data: the whole window
 let v=View{w:1280.,h:800.,hud:None};assert!(close(to_display(1280.,720.,v),(1280.,800.)));
}
#[test]fn mouse_hit_prefers_the_last_drawn_target(){
 let t=[([0.,0.,100.,100.],Target::Panel),([10.,10.,50.,50.],Target::Tab(1)),([20.,20.,30.,30.],Target::RankItem(2))];
 assert_eq!(mouse::hit(&t,25.,25.),Some(Target::RankItem(2)));assert_eq!(mouse::hit(&t,12.,12.),Some(Target::Tab(1)));assert_eq!(mouse::hit(&t,90.,90.),Some(Target::Panel));assert_eq!(mouse::hit(&t,100.,5.),None);
}
#[test]fn wheel_moves_rows_while_selected_and_scrolls_the_detail(){
 use model::{SCROLL_UP,SCROLL_DOWN,UP,DOWN};use mouse::wheel_events as w;
 assert_eq!(w(SCROLL_DOWN,1,Some(Target::Row(3))),DOWN);assert_eq!(w(SCROLL_UP,1,None),UP);assert_eq!(w(SCROLL_UP,1,Some(Target::Panel)),UP);
 assert_eq!(w(SCROLL_DOWN,1,Some(Target::Detail)),SCROLL_DOWN);
 for mode in [2,3]{assert_eq!(w(SCROLL_UP,mode,Some(Target::Detail)),UP);}
 assert_eq!(w(SCROLL_DOWN,0,Some(Target::Row(0))),SCROLL_DOWN);assert_eq!(w(0,1,None),0);
 // Through the model: a selected material moves to the next row
 let mut s=State::new();s.select(48,34,3);s.progress.valid=true;s.tab=3;s.recompute(true);
 if s.rows()>1{s.click(Target::Row(0));s.action(w(SCROLL_DOWN,s.mode,Some(Target::Row(0))));assert_eq!((s.mode,s.row),(1,1));}
 assert!(mouse::hoverable(Target::Row(0))&&mouse::hoverable(Target::Tab(1))&&!mouse::hoverable(Target::Panel)&&!mouse::hoverable(Target::Detail));
}
#[test]fn scrollbar_drag_follows_the_pointer_and_keeps_the_row_visible(){
 use mouse::scroll_offset as o;
 assert_eq!(o(0.,100.,372.,20,8),0);assert_eq!(o(1000.,100.,372.,20,8),12);assert_eq!(o(236.,100.,372.,20,8),6);assert_eq!(o(200.,100.,372.,5,8),0);
 let mut s=State::new();s.select(48,34,3);s.progress.valid=true;s.tab=0;s.recompute(true);
 let rows=s.rows();if rows>8{s.scroll_to(99);assert_eq!(s.scroll,rows-8);assert_eq!(s.row,rows-8);s.row=rows-1;s.scroll_to(0);assert_eq!((s.scroll,s.row),(0,7));}
 assert!(!mouse::hoverable(Target::Scrollbar));
}
#[test]fn mouse_click_waits_for_release_after_opening(){
 let mut a=mouse::Arm::NEW;assert!(!a.step(true,1,1));assert!(!a.step(true,1,0));assert!(!a.step(true,0,0));assert!(a.step(true,1,1));
 assert!(!a.step(false,0,0));assert!(!a.step(true,1,1));
}
#[test]fn mouse_clicks_drive_tabs_rows_lists_and_back(){
 let mut s=State::new();s.progress=high();s.recompute(true);
 assert!(!s.click(Target::Tab(3)));assert_eq!((s.tab,s.mode),(3,0));
 assert!(!s.click(Target::Prev));assert_eq!(s.tab,2);assert!(!s.click(Target::Next));assert_eq!(s.tab,3);
 let rows=s.rows();assert!(rows>2);assert!(!s.click(Target::Row(2)));assert_eq!((s.mode,s.row),(1,2));
 assert!(!s.click(Target::Row(rows)));assert_eq!(s.row,2);
 // The back bar: ours while a row is selected, the game's in mode 0
 assert!(!s.click(Target::Back));assert_eq!(s.mode,0);assert!(s.click(Target::Back));assert_eq!(s.mode,0);
 // A tab click while a row is selected leaves the table
 s.click(Target::Row(1));assert_eq!(s.mode,1);s.click(Target::Tab(0));assert_eq!((s.tab,s.mode),(0,0));
 s.tab=3;s.recompute(true);
 if s.ranks.len()>1{
  s.click(Target::RankOpen);assert_eq!(s.mode,2);let pick=s.ranks[0];s.click(Target::RankItem(0));assert_eq!((s.mode,s.rank),(0,pick));
  s.click(Target::RankOpen);s.click(Target::Panel);assert_eq!(s.mode,0);
  // A click elsewhere on the page only closes the open list
  s.click(Target::RankOpen);s.click(Target::Tab(1));assert_eq!((s.mode,s.tab),(0,3));
 }
 // Read-only tabs have no selectable rows
 s.click(Target::Tab(1));s.click(Target::Row(0));assert_eq!(s.mode,0);
 s.click(Target::Tab(0));s.click(Target::Enter);assert_eq!(s.mode,1);
}

// Player report 2026-11-22: Royal Ludroth mane, Najarala resonating organs (not body), Seregios legs, Nibelsnarf mouth
#[test]fn reported_break_parts_are_marked(){
 let bits=|id:usize,part:u8|{let m=&generated::MONSTERS[id];generated::WEAK[m.weak as usize..(m.weak+m.weak_len) as usize].iter().filter(|r|r.part==part).fold(0,|a,r|a|r.capability)};
 // Break counts from the game files (dtbparts): Rathalos wings left + right, Seregios legs left + right, Barroth claws once
 let times=|id:usize,part:u8|{let m=&generated::MONSTERS[id];generated::WEAK[m.weak as usize..(m.weak+m.weak_len) as usize].iter().filter(|r|r.part==part).map(|r|r.breaks).max().unwrap_or(0)};
 assert_eq!(times(98,6),2);assert_eq!(times(106,7),2);assert_eq!(times(82,3),1);assert_eq!(times(98,1),1);
 assert_eq!(bits(75,3)&1,1);assert_eq!(bits(81,4)&1,1);assert_eq!(bits(81,3)&1,0);assert_eq!(bits(106,6)&1,1);assert_eq!(bits(106,7)&1,1);assert_eq!(bits(77,8)&1,1);
}
#[path="../src/part_info.rs"]mod part_info;
#[test]fn shared_break_and_tail_groups_cover_their_hitzones(){
 let rows=|id:usize|{let m=&generated::MONSTERS[id];&generated::WEAK[m.weak as usize..(m.weak+m.weak_len) as usize]};
 for part in [10,12]{assert!(rows(39).iter().filter(|r|r.part==part).all(|r|r.capability&1!=0));}
 let n=rows(40);assert_eq!(n[4].capability&1,1);assert_eq!(n[5].capability,0);
 let a=rows(107);assert_eq!(a[11].capability,2);assert_eq!(a[12].capability,0);assert_eq!(a[13].capability,2);
 assert_eq!(a[10].capability,2);assert_eq!(rows(108)[9].capability,2);
 for r in rows(108).iter().filter(|r|r.part==7){assert_eq!(r.capability,2);}
 let b=rows(88);assert_eq!(b[5].capability&2,2);assert_eq!(b[6].capability&2,2);
 assert_eq!(b[5].description,b[6].description);
 // One verified attack location stays exclusive (Rathian tail).
 assert!(rows(95).iter().filter(|r|r.capability&2!=0).all(|r|r.part==5));
}
#[test]fn carveable_detached_heads_are_marked_without_marking_broken_states(){
 let rows=|id:usize|{let m=&generated::MONSTERS[id];&generated::WEAK[m.weak as usize..(m.weak+m.weak_len) as usize]};
 let barroth=rows(82);assert_eq!(barroth[0].capability&1,1);assert_eq!(barroth[1].capability&1,1);
 let crystalbeard=rows(86);assert_eq!(crystalbeard[0].capability&1,1);assert_eq!(crystalbeard[1].capability&1,0);
 assert!(generated::WEAK.iter().all(|r|(r.description!=0)==(r.capability!=0)));
}
#[test]fn reward_markers_are_positive_only_and_localized(){
 for label in part_info::BREAK.iter().chain(part_info::SEVER.iter()){assert!(!label.is_empty());}
 let r=&generated::MONSTERS[95];let rows=&generated::WEAK[r.weak as usize..(r.weak+r.weak_len) as usize];
 assert_eq!(rows[0].capability,1);assert_eq!(rows[4].capability,2);assert_eq!(rows[3].capability,0);
 assert_eq!(rows[0].stun,100);assert_eq!(rows[4].stun,0);
 assert!(generated::WEAK.iter().all(|r|r.capability<=3));
}

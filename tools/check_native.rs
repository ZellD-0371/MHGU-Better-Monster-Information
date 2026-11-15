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

#[test]fn neset_parts_use_reviewed_rings_and_separate_picture(){let m=&generated::MONSTERS[39];for row in &generated::WEAK[m.weak as usize..(m.weak+m.weak_len) as usize]{if row.part>=8{assert!(row.ring_len>0);assert!(!row.pending);}}assert!(generated::DIAGRAM_SOURCES[130].0>0);}


#[path="../src/menu_input.rs"]mod menu_input;
#[path="../src/hook_guard.rs"]mod hook_guard;

#[path="../src/button_prompts.rs"]mod button_prompts;

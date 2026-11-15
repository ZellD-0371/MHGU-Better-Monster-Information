//! Validate the menu translator without rejecting an existing Forge hook.
pub fn menu_entry(words:[u32;4],site:u32)->bool{
 const ORIGINAL:[u32;4]=[0xe3a02010,0xe3a00040,0xe0022821,0xe0000821];
 if words==ORIGINAL{return true}
 // Forge replaces only the first eight bytes with ldr pc,[pc,#-4]
 // and an aligned destination. Require the untouched instructions too.
 words[0]==0xe51ff004&&(0x03200000..0xeffff000).contains(&words[1])&&words[1]&3==0&&words[1]!=site&&words[2..]==ORIGINAL[2..]
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn intact_and_chained_menu_entries(){
  let original=[0xe3a02010,0xe3a00040,0xe0022821,0xe0000821];
  assert!(menu_entry(original,0x5de4f8));
  let hooked=[0xe51ff004,0x03951000,original[2],original[3]];
  assert!(menu_entry(hooked,0x5de4f8));
  for (index,value) in [(0,0xea000000),(1,0),(1,0x03951001),(1,0x5de4f8),(2,0),(3,0)]{
   let mut invalid=hooked;invalid[index]=value;assert!(!menu_entry(invalid,0x5de4f8));
  }
 }
}

//! Compact immutable records shared with the resource packer.
#[derive(Clone,Copy)]
pub struct Monster{pub name:u32,pub weak:u16,pub weak_len:u16,pub ail:u16,pub ail_len:u16,pub trap:u16,pub trap_len:u16,pub material:u16,pub material_len:u16,pub deviant:i8}
#[derive(Clone,Copy)]
pub struct Weak{pub capability:u8,pub breaks:u8,pub stun:i16,pub name:u32,pub description:u32,pub part:u8,pub values:[f32;8],pub ring:u16,pub ring_len:u8,pub pending:bool}
#[derive(Clone,Copy)]
pub struct Row{pub name:u32,pub icon:u16,pub values:[f32;5]}
#[derive(Clone,Copy)]
pub struct Item{pub name:u32,pub description:u32,pub native:u16,pub rarity:u8,pub sell:u32,pub icon:u16}
#[derive(Clone,Copy)]
pub struct Material{pub item:u16,pub rank:u8,pub cells:[i16;5],pub entry:u16,pub entry_len:u16}
#[derive(Clone,Copy)]
pub struct Acquisition{pub name:u32,pub quantity:u16,pub percent:u16}
#[derive(Clone,Copy)]
pub struct Sprite{pub x:u16,pub y:u16,pub w:u16,pub h:u16}

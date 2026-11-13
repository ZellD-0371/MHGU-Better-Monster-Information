//! Three attempts per selection, at least five active seconds apart.
pub struct RetryGate{id:Option<u16>,failures:u32,remaining:f32}
impl RetryGate{
 pub const NEW:Self=Self{id:None,failures:0,remaining:0.};
 pub fn select(&mut self,id:u16){if self.id!=Some(id){*self=Self{id:Some(id),..Self::NEW};}}
 pub fn allows(&self)->bool{self.failures<3&&self.remaining<=0.}
 pub fn failed(&mut self){if self.failures<3{self.failures+=1;}self.remaining=5.;}
 pub fn succeeded(&mut self){self.failures=0;self.remaining=0.;}
 pub fn tick(&mut self,dt:f32,visible:bool){if !visible{*self=Self::NEW;}else if dt.is_finite()&&dt>0.{self.remaining=(self.remaining-dt).max(0.);}}
}

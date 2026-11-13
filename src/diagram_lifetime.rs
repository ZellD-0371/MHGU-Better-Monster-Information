//! User textures require explicit GPU retirement before unregistering/freeing pixels.
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum Status{Busy,Ready,Destroyed}
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum Action{Keep,RequestDestroy,WaitForDestroy(i32),Free}
pub struct Lifetime{retiring_frames:i32}
impl Lifetime{
 pub const NEW:Self=Self{retiring_frames:0};
 pub fn retiring(&self)->bool{self.retiring_frames>0}
 pub fn advance(&mut self,notes_open:bool,status:Option<Status>)->Action{
  let Some(status)=status else{*self=Self::NEW;return Action::Keep};
  // A committed destruction cannot be cancelled: finish it before loading again.
  if self.retiring(){
   if status==Status::Destroyed{*self=Self::NEW;return Action::Free}
   if self.retiring_frames<1024{self.retiring_frames+=1;}
   return Action::WaitForDestroy(self.retiring_frames)
  }
  if !notes_open&&status==Status::Ready{self.retiring_frames=1;Action::RequestDestroy}else{Action::Keep}
 }
}

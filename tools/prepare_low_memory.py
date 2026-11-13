"""Generate the small atlas; never replace or resize standard resources."""
from pathlib import Path
import re
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
def main():
 code=(ROOT/'src/generated.rs').read_text(encoding='utf-8')
 rects=[tuple(map(int,r)) for r in re.findall(r'Sprite\{x:(\d+),y:(\d+),w:(\d+),h:(\d+)\}',code)]
 height=int(re.search(r'ATLAS_HEIGHT:usize=(\d+)',code)[1])
 source=Image.frombytes('RGBA',(2048,height),(ROOT/'assets/native/monster-icons.rgba').read_bytes())
 images=[];packed=[];x=y=row=0
 for sx,sy,w,h in rects:
  im=source.crop((sx,sy,sx+w,sy+h));im.thumbnail((32,32),Image.Resampling.LANCZOS)
  if x+im.width+2>2048:x=0;y+=row+2;row=0
  packed.append((x+1,y+1,*im.size));images.append(im);x+=im.width+2;row=max(row,im.height)
 ah=1<<max(1,(y+row+1).bit_length());atlas=Image.new('RGBA',(2048,ah))
 for im,(x,y,w,h) in zip(images,packed):atlas.paste(im,(x,y))
 out=ROOT/'assets/native-low';out.mkdir(exist_ok=True)
 (out/'monster-icons.rgba').write_bytes(atlas.tobytes())
 new='pub static SPRITES:[Sprite;'+str(len(packed))+']=['+','.join(f'Sprite{{x:{x},y:{y},w:{w},h:{h}}}' for x,y,w,h in packed)+'];'
 code=re.sub(r'pub static SPRITES:[^\n]+',lambda _:new,code)
 code=re.sub(r'ATLAS_HEIGHT:usize=\d+',f'ATLAS_HEIGHT:usize={ah}',code)
 (ROOT/'src/generated_low.rs').write_text(code,encoding='utf-8')
 assert len(packed)==240 and ah<=256
 assert all(0<x and 0<y and x+w<=2048 and y+h<=ah for x,y,w,h in packed)
 print(f'Low-memory atlas: 2048x{ah}, {2048*ah*4} bytes, {len(packed)} sprites')
if __name__=='__main__':main()

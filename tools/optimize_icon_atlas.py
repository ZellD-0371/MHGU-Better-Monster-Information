"""Repack original-size icons into 2048x512 without changing any sprite pixels."""
from pathlib import Path
from PIL import Image
import re,hashlib,json
ROOT=Path(__file__).resolve().parents[1]
def main():
 p=ROOT/'src/generated.rs';code=p.read_text(encoding='utf-8');oldh=int(re.search(r'ATLAS_HEIGHT:usize=(\d+)',code)[1]);src=Image.frombytes('RGBA',(2048,oldh),(ROOT/'assets/native/monster-icons.rgba').read_bytes())
 rects=[tuple(map(int,x)) for x in re.findall(r'Sprite\{x:(\d+),y:(\d+),w:(\d+),h:(\d+)\}',code)];images=[src.crop((x,y,x+w,y+h)) for x,y,w,h in rects]
 tall=[i for i,im in enumerate(images) if im.height>64];large=[i for i,im in enumerate(images) if im.height<=64 and max(im.size)>48];small=[i for i,im in enumerate(images) if max(im.size)<=48]
 assert len(tall)==1 and images[tall[0]].width<=64 and images[tall[0]].height<=130 and len(large)<=215
 pos={tall[0]:(1981,1)};slots=[(x*66+1,y*66+1) for y in range(7) for x in range(31) if not(x==30 and y<2)]
 for i,xy in zip(large,slots):pos[i]=xy
 x=1;y=463
 for i in small:
  assert x+images[i].width<2048 and images[i].height<=48;pos[i]=(x,y);x+=images[i].width+2
 assert len(pos)==len(rects)
 dst=Image.new('RGBA',(2048,512));new=[]
 for i,im in enumerate(images):
  x,y=pos[i];dst.paste(im,(x,y));new.append((x,y,*im.size))
 for i,(x,y,w,h) in enumerate(new):assert dst.crop((x,y,x+w,y+h)).tobytes()==images[i].tobytes()
 # All padded sprite allocations must remain separate, avoiding filter bleed.
 for i,(x,y,w,h) in enumerate(new):
  assert 0<x and 0<y and x+w<2048 and y+h<512
  for xx,yy,ww,hh in new[:i]:assert x+w+1<=xx-1 or xx+ww+1<=x-1 or y+h+1<=yy-1 or yy+hh+1<=y-1
 line='pub static SPRITES:[Sprite;'+str(len(new))+']=['+','.join(f'Sprite{{x:{x},y:{y},w:{w},h:{h}}}' for x,y,w,h in new)+'];'
 code=re.sub(r'pub static SPRITES:[^\n]+',lambda _:line,code);code=re.sub(r'ATLAS_HEIGHT:usize=\d+','ATLAS_HEIGHT:usize=512',code)
 p.write_text(code,encoding='utf-8');(ROOT/'assets/native/monster-icons.rgba').write_bytes(dst.tobytes());dst.save(ROOT/'assets/native/monster-icons-preview.png')
 print(json.dumps({'oldBytes':2048*oldh*4,'newBytes':2048*512*4,'spritesVerifiedPixelIdentical':len(new)}))
if __name__=='__main__':main()

"""Create a self-contained Forge mod, without changing any other installed mod."""
from pathlib import Path
import shutil,zipfile,json,re
ROOT=Path(__file__).resolve().parents[1]
def main():
    target=ROOT/'dist/native/Better-Monster-Information/romfs/nativeNX/plugins'
    target.mkdir(parents=True,exist_ok=True)
    shutil.copy2(ROOT/'better_monster_information.nro',target)
    assets=target/'better-monster-information';assets.mkdir(exist_ok=True)
    for name in ['monster-text.bin','monster-icons.rgba']:
        shutil.copy2(ROOT/'assets/native'/name,assets/name)
    shutil.copy2(ROOT/'assets/NotoSansCJKkr-Regular.otf',assets/'NotoSansCJKkr-Regular.otf')
    (assets/'NOTO_LICENSE.txt').unlink(missing_ok=True)
    shutil.copy2(ROOT/'assets/NOTO_LICENSE.txt',assets/'OFL.md')
    shutil.copy2(ROOT/'conf/FORGE_LICENSE.txt',assets/'FORGE_LICENSE.md')
    shutil.copy2(ROOT/'CREDITS.md',assets/'CREDITS.md')
    shutil.copytree(ROOT/'assets/native/parts',assets/'parts',dirs_exist_ok=True)
    mod=target.parents[2]
    for name in ['README.txt','설치와 조작.txt','DISTRIBUTION_NOTICE.txt','THIRD_PARTY_NOTICES.txt','FORGE_LICENSE.txt']:
        (mod/name).unlink(missing_ok=True)
    manifest=(ROOT/'Cargo.toml').read_text(encoding='utf-8')
    package=manifest.split('[package]',1)[1].split('[',1)[0]
    version=re.search(r'^version\s*=\s*"([^"]+)"',package,re.M).group(1)
    archive=ROOT/f'dist/Better-Monster-Information_v{version}.zip'
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=6) as z:
        for file in sorted(mod.rglob('*')):
            if file.is_file():z.write(file,file.relative_to(mod.parent))
    print(json.dumps({'directory':str(mod),'zip':str(archive),'files':sum(f.is_file() for f in mod.rglob('*'))}))
if __name__=='__main__':main()

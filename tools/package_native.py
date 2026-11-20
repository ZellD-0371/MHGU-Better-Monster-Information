"""Create a self-contained Forge mod, without changing any other installed mod."""
from pathlib import Path
import shutil,zipfile,json,re,argparse,hashlib
ROOT=Path(__file__).resolve().parents[1]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--variant',choices=['standard','low-memory'],default='standard');variant=parser.parse_args().variant
    built=ROOT/'dist/build'/variant
    audit=json.loads((built/'build.json').read_text(encoding='utf-8-sig'))
    binary=built/'better_monster_information.nro'
    assert audit['variant']==variant and audit['sha256'].lower()==hashlib.sha256(binary.read_bytes()).hexdigest()
    target=ROOT/('dist/native' if variant=='standard' else 'dist/native-low')/'Better-Monster-Information/romfs/nativeNX/plugins'
    target.mkdir(parents=True,exist_ok=True)
    shutil.copy2(binary,target)
    assets=target/'better-monster-information';assets.mkdir(exist_ok=True)
    for name in ['monster-icons.rgba','kbm-icons.rgba']:
        source=ROOT/('assets/native-low' if variant=='low-memory' else 'assets/native')/name
        shutil.copy2(source,assets/name)
    for obsolete in ['monster-text.bin','NotoSansCJKkr-Regular.otf']:(assets/obsolete).unlink(missing_ok=True)
    shutil.copytree(ROOT/'assets/languages',assets/'languages',dirs_exist_ok=True)
    # License files next to the plugin: the mod's own, the third-party notices, and the full texts they refer to
    for obsolete in ['OFL.md','CREDITS.md']:(assets/obsolete).unlink(missing_ok=True)
    shutil.copy2(ROOT/'LICENSE',assets/'LICENSE.txt')
    shutil.copy2(ROOT/'THIRD_PARTY_NOTICES.md',assets/'THIRD_PARTY_NOTICES.md')
    shutil.copy2(ROOT/'assets/NOTO_LICENSE.txt',assets/'NOTO_LICENSE.txt')
    shutil.copy2(ROOT/'conf/FORGE_LICENSE.txt',assets/'FORGE_LICENSE.md')
    shutil.copytree(ROOT/('assets/native-low' if variant=='low-memory' else 'assets/native')/'parts',assets/'parts',dirs_exist_ok=True)
    mod=target.parents[2]
    for name in ['README.txt','설치와 조작.txt','DISTRIBUTION_NOTICE.txt','THIRD_PARTY_NOTICES.txt','FORGE_LICENSE.txt']:
        (mod/name).unlink(missing_ok=True)
    manifest=(ROOT/'Cargo.toml').read_text(encoding='utf-8')
    package=manifest.split('[package]',1)[1].split('[',1)[0]
    version=re.search(r'^version\s*=\s*"([^"]+)"',package,re.M).group(1)
    suffix='_LowMemory' if variant=='low-memory' else ''
    archive=ROOT/f'dist/Better-Monster-Information{suffix}_v{version}.zip'
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=6) as z:
        for file in sorted(mod.rglob('*')):
            if file.is_file():z.write(file,file.relative_to(mod.parent))
    print(json.dumps({'directory':str(mod),'zip':str(archive),'files':sum(f.is_file() for f in mod.rglob('*'))}))
if __name__=='__main__':main()

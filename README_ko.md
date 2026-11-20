<p align="center">
  <a href="README.md">English</a> | <strong>한국어</strong>
</p>

<h3 align="center">스크린샷</h3>

<p align="center">
  <a href="docs/screenshots.md">스크린샷 보기</a>
</p>


# MHGU Better Monster Information

Monster Hunter Generations Ultimate의 헌터 노트에 몬스터 정보 페이지를 추가하는 Forge 모드입니다. 육질과 부위 그림, 상태 이상 내성, 함정 효과, 소재 획득 정보를 게임 안에서 확인할 수 있습니다.

## 지원 환경

- MHGU 글로벌 버전 1.4.0
- MHGU Recomp 혹은 `Forge 1.5.1` 이 설치된 에뮬레이터

## 주요 기능

- 등록된 소형·대형 몬스터의 도감 마지막에 정보 페이지 추가
- 육질 표에서 선택한 부위를 그림의 원·타원으로 강조
- 상태 이상 내성 및 함정 효과를 DB 수치로 표시
- 하위·상위·G급 및 특수허가 구간별 소재 확률과 상세 획득 조건 확인

## 설치

1. 게임을 종료합니다.
2. `Better-Monster-Information_vX.X.X.zip`을 압축 해제합니다.
3. `Better-Monster-Information` 폴더를 Recomp 실행 파일 옆의 `mods` 폴더에 넣습니다.
4. 게임을 실행합니다.

```text
mods/
└─ Better-Monster-Information/
   └─ romfs/
      └─ nativeNX/
         └─ plugins/
            ├─ better_monster_information.nro
            └─ better-monster-information/
               ├─ monster-icons.rgba
               ├─ languages/
               │  ├─ text.en.bin … text.es.bin
               │  └─ font.shared.otf
               └─ parts/
```

### 일반용 / 저사양용

- 일반용: `Better-Monster-Information_vX.X.X.zip` — 원본 아이콘 화질을 유지한 4MiB 아틀라스와 원본 해상도 부위 그림.
- 저사양용: `Better-Monster-Information_LowMemory_vX.X.X.zip` — 아이콘 크기를 최대 32px로 제한하고, 약 1.06MiB 크기의 텍스처 아틀라스를 사용합니다. 몬스터 부위 도식은 384×256픽셀 이내에 맞도록 미리 크기를 조정했습니다.

> [!IMPORTANT]
> 에뮬레이터 유저는 **저사양용**을 사용해주세요.

두 버전 모두 몬스터 도감을 열 때 아이콘 아틀라스를, 필요할 때 부위 그림을 불러옵니다. 도감을 닫으면 진행 중인 화면 처리가 끝나는 대로 두 텍스처와 픽셀 버퍼를 해제하며, 다시 열면 필요한 리소스를 다시 불러옵니다.

두 버전은 같은 모드 폴더와 설정을 사용합니다. 하나만 설치하고, 변경 시 게임을 종료한 뒤 해당 폴더를 교체하세요. 

## 사용 방법

### 목록 확인

게임 메뉴에서 헌터 노트의 몬스터 목록을 열고 등록된 몬스터를 선택합니다. R로 마지막 페이지에 들어가면 추가 정보가 표시됩니다.

## 설정

Insert로 Forge 설정을 열고 `Plugin UI > Better Monster Information`에서 변경합니다.

| 항목 | 내용 |
| --- | --- |
| Language | 지원 언어 9개 중 선택. 기본값 English |
| Button layout | Nintendo / Xbox / PlayStation. 기본값 Nintendo |
| 로그 기록 활성화 | 몬스터 이동·리소스·화면 배치 진단 로그 기록. 기본값 꺼짐. 경고·오류는 계속 기록. 선택한 언어로 표시 |

지원 언어: English, 简体中文, 繁體中文, 日本語, 한국어, Deutsch, Français, Italiano, Español

## 저장 데이터

| 파일 | 내용 |
| --- | --- |
| `better-monster-information.ini` | 공통 언어, 버튼 표시 및 진단 로그 설정 |

Windows Recomp 기본 경로는 `%APPDATA%\MHGURecomp\save\save_0\`입니다. 실행 환경에서 저장 경로를 따로 지정했다면 해당 경로를 사용합니다. 

## 소스 빌드

필요 도구: Rust nightly와 rust-src, Forge Cargo 빌드 도구(`cargo forge`), devkitARM, Python 3.10 이상. Forge SDK 커밋은 `Cargo.toml`에 지정되어 있습니다.

```powershell
./tools/Build.ps1
./tools/Test.ps1
py -3.10 -X utf8 tools/package_native.py
```

결과는 `better_monster_information.nro`, `dist/native/Better-Monster-Information`, `dist/Better-Monster-Information_vX.X.X.zip`에 생성됩니다. 빌드 스크립트의 devkitARM 경로는 사용 환경에 맞춰 확인하세요.

자동 검사는 실제 게임의 조작·화면 검수를 대신하지 않습니다.


```powershell
# Regenerate changed text/font inputs with fontTools.
py -3.10 -X utf8 tools/prepare_languages.py
# Generate the small atlas only when icon inputs change (requires Pillow).
py -3.10 -X utf8 tools/optimize_icon_atlas.py
py -3.10 -X utf8 tools/prepare_low_memory.py
./tools/Build.ps1 -Variant low-memory
./tools/Test.ps1 -Variant low-memory
py -3.10 -X utf8 tools/package_native.py --variant low-memory
```

## 저장소 구성

`src`, `.cargo`, `link`, `conf`, `assets/native`, `tools`는 네이티브 모드 빌드·검사·패키지 생성에 필요한 파일입니다. `Cargo.lock`과 생성된 Rust 데이터 및 네이티브 리소스도 함께 보관합니다.

## 크레딧

**Ping**(원본 MHXX Dex), **fabric_soul**(MHXX Dex 웹 포트)과 웹 포트가 밝힌 기여자 breakds, 掌机迷, MHVuze, svanheulen, ACG汉化组, dasding and team, 팀 갈무리, Zeff and team께 감사드립니다. 출처와 이용 조건은 `THIRD_PARTY_NOTICES.md`에 정리했습니다.

## 라이선스

MIT License. `LICENSE`와, 외부 구성 요소·게임 데이터에 관한 `THIRD_PARTY_NOTICES.md`를 참고하세요.

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
- MHGU Recomp

현재 에뮬레이터는 지원하지 않습니다.

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
               ├─ monster-text.bin
               ├─ monster-icons.rgba
               ├─ NotoSansCJKkr-Regular.otf
               └─ parts/
```

## 사용 방법

### 목록 확인

게임 메뉴에서 헌터 노트의 몬스터 목록을 열고 등록된 몬스터를 선택합니다. R로 마지막 페이지에 들어가면 추가 정보가 표시됩니다.

## 설정

Insert로 Forge 설정을 열고 `Plugin UI > Better Monster Information`에서 변경합니다.

| 항목 | 내용 |
| --- | --- |
| Language | 지원 언어 9개 중 선택. 기본값 English |
| Button layout | Nintendo / Xbox / PlayStation. 기본값 Nintendo |

지원 언어: English, 简体中文, 繁體中文, 日本語, 한국어, Deutsch, Français, Italiano, Español

## 저장 데이터

| 파일 | 내용 |
| --- | --- |
| `better-monster-information.ini` | 공통 언어 및 버튼 표시 설정 |

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

## 저장소 구성

`src`, `.cargo`, `link`, `conf`, `assets/native`, `tools`는 네이티브 모드 빌드·검사·패키지 생성에 필요한 파일입니다. `Cargo.lock`과 생성된 Rust 데이터 및 네이티브 리소스도 함께 보관합니다.

## 출처 및 크레딧

몬스터 부위 도식과 육질 및 소재 데이터는 Ping의 MHXX Dex를 기반으로 제작된 MHXX Dex 웹 포트에서 가져왔습니다.

- **Ping** — 원본 MHXX Dex
- **fabric_soul** — MHXX Dex 웹 포트
- 웹 포트가 명시한 기여자: breakds, 掌机迷, MHVuze, svanheulen, ACG汉化组, dasding and team, 팀 갈무리, Zeff and team. 이 목록이 각 부위 그림의 개별 작성자를 확정하는 것은 아닙니다.
- **mhgu-forge / forge-rs** — Fexty, MIT License
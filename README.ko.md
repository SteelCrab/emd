[🇺🇸 English](README.md) | [🇰🇷 한국어](README.ko.md)

# AWS CLI 설치 도구 (TUI)

Rust로 만든 크로스 플랫폼 AWS CLI v2 터미널 설치 도구입니다.

## 기능

- 🖥️ **크로스 플랫폼**: macOS, Windows, Linux (x86_64, arm64)
- 📥 **설치**: AWS CLI v2 다운로드 및 설치
- 🗑️ **삭제**: 시스템에서 AWS CLI 제거
- 📊 **진행률 바**: 실시간 다운로드 진행 상황

## 설치

[Releases](../../releases)에서 다운로드하거나 소스에서 빌드:

```bash
cargo build --release
./target/release/t-aws
```

## 사용법

1. 애플리케이션 실행
2. 환영 화면에서 **Enter** 누름
3. 액션 선택:
   - **[1]** AWS CLI 설치
   - **[2]** AWS CLI 삭제
4. 안내에 따라 진행

## 지원 플랫폼

| OS | 아키텍처 |
|----|----------|
| macOS | x86_64, arm64 |
| Windows | x86_64 |
| Linux | x86_64, arm64 |

## 라이선스

MIT

# 품질 스크립트 안내

[English guide](README.md)

품질 게이트를 위한 스크립트 모음입니다.

커버리지 기준은 **85%** (`MIN_LINES=85`) 입니다.
CI 업로드용 커버리지 출력은 `target/llvm-cov-target/lcov.info` 입니다.

## 파일 구성
- `rust-lint-cleanup.sh`: clippy 기반 정리 스크립트(`--fix`, 일반 검사, dead-code 검사)
- `rust-coverage.sh`: 시나리오 게이트 + `cargo llvm-cov` 라인 커버리지 게이트
- `test-scenarios.csv`: 시나리오 카탈로그 데이터

## 사용 순서
저장소 루트에서 순서대로 실행:

```bash
./tools/rust-lint-cleanup.sh
./tools/rust-coverage.sh
```

## 주요 환경 변수
- `MIN_LINES` (기본값: `85`)
- `SCENARIO_CATALOG` (기본값: `tools/test-scenarios.csv`)
- `MIN_TOTAL` (기본값: `20`)
- `MIN_AUTOMATED` (기본값: `17`)

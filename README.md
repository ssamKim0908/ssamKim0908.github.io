# ssamKim0908.github.io

자료구조 학습 정리 사이트. [mdBook](https://rust-lang.github.io/mdBook/) 기반.

향후 각 단원에 Rust + WebAssembly 로 작성된 인터랙티브 렌더러를 임베드할 예정.

## 로컬 실행

```sh
mdbook serve --open
```

## 빌드

```sh
mdbook build
```

빌드 산출물은 `book/` 에 생성되며 gitignore 됩니다.
`main` 브랜치 push 시 GitHub Actions 가 자동 빌드 후 Pages 에 배포합니다.

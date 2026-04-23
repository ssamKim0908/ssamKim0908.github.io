# 소개

자료구조 학습 정리. 각 단원은 개념 설명과 함께 Rust + WebAssembly 로 작성된
인터랙티브 렌더러를 임베드합니다. 입력을 넣으면 해당 자료구조가 그 자리에서
그려집니다.

## 렌더러 미리보기

아래 구는 WebGL2 로 그려지며, 로직은 Rust 로 작성되어 WebAssembly 로
컴파일된 뒤 브라우저에서 돌아갑니다. 각 단원의 자료구조 시각화도 같은
파이프라인 위에 올라갑니다.

<canvas id="intro-sphere" style="width:100%;max-width:640px;height:360px;display:block;margin:1.5em auto;background:#11151c;border-radius:8px;"></canvas>

<script type="module">
  import init, { start } from './wasm/renderer.js';
  await init();
  start('intro-sphere');
</script>

## 구성

- **선형 자료구조** — 배열, 연결 리스트, 스택, 큐
- **트리** — 이진 트리, 이진 탐색 트리, 힙
- **그래프** — 기초, BFS, DFS

좌측 사이드바에서 원하는 단원을 선택하세요.

# 顔の横回転モデル（Rust + Leptos）

「アニメ顔の横回転モデル ブラウザ実装仕様」の楕円断面モデルを、Rust / Leptos (CSR) で実装したものです。

- `src/model.rs` … 描画に依存しない幾何計算（仕様 第3〜8節・第11節の移植）とテスト
- `src/app.rs` … Leptos の UI と SVG 描画（顔・y=88 断面図・数値）
- `src/texture.rs` … 目の画像素材（`assets/eyes.png`）を楕円表面に貼って回転させる処理
- `style.css` … スタイル（ライト／ダーク対応）

### 目の画像を差し替えるとき

背景透過の PNG を `assets/eyes.png` に置き、`src/texture.rs` の `EYES` を合わせます。

- `width` / `height`：画像のサイズ（px）
- `left_pupil` / `right_pupil`：左右の瞳の中心（px）。モデルの瞳の位置 (x=±28, y=88) に合わせて配置され、両瞳の間隔から縮尺が決まります
- `left_region` / `right_region`：まゆ毛を含む各目の範囲（px）

## 必要なもの

```sh
brew install rustup trunk wasm-bindgen binaryen
rustup default stable
rustup target add wasm32-unknown-unknown
export PATH=/opt/homebrew/opt/rustup/bin:$PATH   # rustup は keg-only のため
```

`wasm-bindgen` CLI のバージョンは `Cargo.lock` の `wasm-bindgen` クレートと一致させてください。

## 実行

```sh
trunk serve --offline --open     # 開発サーバー http://localhost:8080
trunk build --release --offline  # dist/ に静的ファイルを出力
cargo test --lib                 # 仕様 第13節の検証条件
```

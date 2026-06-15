# nrun

`package.json` の `scripts` を対話的に選んで実行できる、シンプルな CLI ツールです。

## 特徴

- 現在のディレクトリから親ディレクトリへ遡って `package.json`（`scripts` が定義されているもの）を自動検出
- `bun.lockb` / `pnpm-lock.yaml` / `yarn.lock` の有無からパッケージマネージャ（bun / pnpm / yarn / npm）を自動判定
- スクリプト一覧を対話的に選択して実行
- スクリプト名を直接指定して即実行することも可能

## インストール

Rust のツールチェイン（cargo）が必要です。

```sh
cargo install --path .
```

または、リリースビルドを作成して任意の場所に配置します。

```sh
cargo build --release
# 生成物: target/release/nrun
```

## 使い方

### 対話モード

`package.json` が存在するプロジェクト内（サブディレクトリでも可）で実行します。

```sh
nrun
```

`scripts` の一覧が表示されるので、矢印キーで選択して実行します。

### スクリプト名を直接指定

```sh
nrun <script名>
```

例:

```sh
nrun dev
```

指定したスクリプトが `scripts` に存在しない場合はエラーになります。

## 動作の仕組み

1. カレントディレクトリから上位ディレクトリへ向かって `scripts` を含む `package.json` を探索します。
2. 見つかった `package.json` のあるディレクトリ内のロックファイルから、使用するパッケージマネージャを判定します。
   - `bun.lockb` があれば `bun`
   - `pnpm-lock.yaml` があれば `pnpm`
   - `yarn.lock` があれば `yarn`
   - それ以外は `npm`
3. 判定したパッケージマネージャで `<pm> run <script>` を実行します。

## 必要環境

- Rust（edition 2024 をサポートするバージョン）

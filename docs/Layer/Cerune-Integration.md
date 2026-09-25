# Cerune Integration

## 目的

Ceruneが持つVM実行経路と複数のBackend Artifact生成経路を、Whitebaseの独立したBackendとして接続し、同一のCerune処理を異なる実行経路で測定・比較できるようにします。

Ceruneとの統合はL3 Backend Integrationの責務として扱います。

Whitebase CoreへCerune固有の処理や外部toolchain依存を追加しません。

---

## 責務境界

CeruneとWhitebaseの責務を次のように分けます。

```text
Cerune
├── Language / Frontend
├── Cerune IR
├── Bytecode / VM
└── Emitters
     ├── C
     ├── LLVM
     ├── QBE
     ├── WAT
     ├── Assembly
     └── Native Object

             ↓

Whitebase Backend Integration
├── VM integration
├── external toolchain integration
├── execution preparation
└── ComputeBackend adaptation

             ↓

Whitebase Core
             ↓
Whitebase Runner
             ↓
Measure / Compare / Observe
```

Ceruneは言語の意味、VM実行、Artifact生成までを担当します。

Whitebaseは、Ceruneから受け取ったVMまたはArtifactをWhitebase Backendとして成立させる処理を担当します。

---

## 2つの統合経路

Cerune統合は、性質の異なる2経路に分けます。

### Route 1: VM

CeruneをRustライブラリとして利用し、生成済みbytecodeをCerune VMで直接実行します。

```text
Cerune Source
      ↓
Cerune
      ↓
BytecodeProgram
      ↓
Cerune VM Host-call API
      ↓
whitebase-cerune-vm-adapter
      ↓
ComputeBackend
```

この経路では外部compilerやruntimeを必要としません。

Cerune側に必要なのは、生成済みbytecodeの関数をホストから呼び出すための最小限のVM APIです。

---

### Route 2: Artifact

CeruneのEmitterから成果物を受け取り、Whitebase側で対応するtoolchainまたはruntimeを利用して実行可能な形へ変換します。

```text
Cerune Source
      ↓
Cerune Emitter
      ↓
Backend Artifact
      ↓
Whitebase Adapter
      ↓
External Toolchain / Runtime
      ↓
Executable / Callable Target
      ↓
ComputeBackend
```

外部toolchainの探索、availability判定、build、link、loadなどはWhitebase側の責務です。

---

## Adapterの分離

Ceruneの各実行経路は独立したadapter crateとして扱います。

想定する構成:

```text
crates/backend/integration/
├── whitebase-backend-bridge
├── whitebase-cpp-adapter
├── whitebase-asm-adapter
├── whitebase-windows-gnu-adapter
│
├── whitebase-cerune-vm-adapter
├── whitebase-cerune-c-adapter
├── whitebase-cerune-llvm-adapter
├── whitebase-cerune-qbe-adapter
├── whitebase-cerune-wat-adapter
├── whitebase-cerune-asm-adapter
└── whitebase-cerune-native-adapter
```

最初からすべてを作る必要はありません。

各経路を実装するときに、その経路専用のcrateを追加します。

---

## 依存関係

各adapterは、その経路を成立させるために必要な依存だけを持ちます。

```text
whitebase-cerune-vm-adapter
└── cerune-lang


whitebase-cerune-c-adapter
├── cerune-lang
└── C toolchain integration


whitebase-cerune-qbe-adapter
├── cerune-lang
├── QBE integration
└── assembler / linker integration
```

ある経路の外部依存を、別のCerune Backendの必須依存にはしません。

たとえばQBEが利用できない環境でも、Cerune VM Backendのavailabilityには影響させません。

---

## Coreとの境界

`whitebase-core`はCeruneや外部toolchainへ直接依存しません。

既存の依存方向を維持します。

```text
whitebase-core
      │
      ├── whitebase-backend-contract
      │
      └── whitebase-backend-bridge
                         │
                         ├── existing adapters
                         │
                         ├── whitebase-cerune-vm-adapter
                         ├── whitebase-cerune-c-adapter
                         ├── whitebase-cerune-llvm-adapter
                         ├── whitebase-cerune-qbe-adapter
                         ├── whitebase-cerune-wat-adapter
                         ├── whitebase-cerune-asm-adapter
                         └── whitebase-cerune-native-adapter
```

`whitebase-core`から見ると、Cerune Backendも既存Backendと同じ`ComputeBackend`です。

Cerune固有のコンパイル、Artifact、toolchain、VM値などをCoreへ漏らしません。

---

## Backend Bridge

`whitebase-backend-bridge`は、利用可能なCerune adapterを既存Backendと同じ形でCoreへ公開します。

Bridgeは各Cerune実行方式そのものを実装しません。

実行方式固有の処理は各adapterに置き、Bridgeは統合のみを担当します。

---

## Availability

「Ceruneが成果物を生成できること」と「現在のWhitebase環境で成果物を実行できること」を分けます。

たとえば:

```text
Cerune VM
  Cerune support: yes
  External dependency: none
  Available: yes

Cerune C
  Cerune support: yes
  C compiler: clang
  Available: yes

Cerune QBE
  Cerune support: yes
  qbe: missing
  Available: no
```

外部toolchainが存在しない場合、暗黙に別の経路へfallbackしません。

そのBackendのみをUnavailableとして扱います。

---

## PreparationとMeasurement

コンパイルや外部toolchain起動を通常の演算時間へ混ぜません。

VM経路:

```text
Preparation

source
  ↓
compile to bytecode
  ↓
resolve function


Measurement

arguments
  ↓
VM invocation
  ↓
result
```

Artifact経路:

```text
Preparation

source
  ↓
emit artifact
  ↓
external build / link / load
  ↓
callable target


Measurement

arguments
  ↓
target invocation
  ↓
result
```

Whitebase Runnerが`ComputeBackend`呼び出しを計測する場合、adapterは可能な限りその呼び出し以前にPreparationを完了させます。

コンパイル時間やlink時間そのものを観測する場合は、演算benchmarkとは別の測定対象として扱います。

---

## Artifact Adapterの実行形式

Emitter Artifactを既存`ComputeBackend`へ接続する具体的な方法は、各経路を実装するときに決定します。

候補には次があります。

```text
Artifact
   ↓
Whitebase-generated harness
   ↓
build
   ↓
callable function
```

または、

```text
Artifact
   ↓
build
   ↓
process
   ↓
input / output
```

があります。

既存`ComputeBackend`のOperationを公平に比較する場合は、実際の演算呼び出しだけを測定できるcallable形式を優先します。

ただし、まだ実装していない経路について共通ABIや共通loaderを先に仮定しません。

複数adapterで同じ処理が実際に必要になった時点で共通化します。

---

## Cerune側へ持ち込まない責務

Whitebase統合のためだけに、Ceruneへ以下を追加しません。

* Whitebase専用Backend API
* clang / gcc管理
* LLVM toolchain管理
* QBE管理
* Wasm Runtime管理
* assembler / linker管理
* WhitebaseのCapability判定
* Whitebaseのmeasurement処理
* Whitebase用の新しい言語構文

Cerune側への変更は、Cerune単体でも自然な機能である場合に限ります。

VM Host-call APIは、Cerune VMを一般のホストアプリケーションへ埋め込む機能としてCerune側に置きます。

---

## 最初の対象

最初はCerune VM経路だけを実装します。

```text
Cerune
      ↓
Bytecode
      ↓
Cerune VM
      ↓
whitebase-cerune-vm-adapter
      ↓
whitebase-backend-bridge
      ↓
whitebase-core
      ↓
whitebase-runner
```

最初のOperationは、動的配列ABIを必要としない`AddScalarF64`を候補とします。

この経路が成立してからArtifact経路を一つずつ追加します。

想定順は固定しませんが、各追加では以下を確認します。

* 対応target
* 必要toolchain
* availability
* Preparation
* execution boundary
* measurement boundary
* error mapping
* platform差

---

## 非目標

この設計では次を行いません。

* CeruneをWhitebase repositoryへ取り込む
* CeruneにWhitebaseの外部toolchain依存を持たせる
* `whitebase-core`からCeruneへ直接依存する
* すべてのCerune経路を一つのadapter crateへまとめる
* すべての外部toolchainをWhitebase利用時の必須依存にする
* 未実装の経路まで先に共通抽象化する
* unavailableな経路を黙って別Backendへfallbackする

---

## 将来

同じCeruneソースについて、最終的には次のような実行経路を独立したBackendとして比較できる構成を目指します。

```text
Cerune Source
   │
   ├── Cerune VM
   ├── C
   ├── LLVM
   ├── QBE
   ├── WAT
   ├── Assembly
   └── Native Object
           │
           ▼
       Whitebase
           │
           ▼
   Measure / Compare / Observe
```

Whitebaseは、Ceruneの言語実装を変更する場所ではなく、Ceruneが提供する複数の実行表現を同じ観測基盤へ接続する場所として扱います。

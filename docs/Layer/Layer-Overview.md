# Layer-Overview

## 目的

Whitebaseを構成する各Layerの責務と関係を整理するためのドキュメントです。

各Layerについて、次の内容を明確にします。

- どのような役割を持つか
- 何を目的とするか
- 他のLayerとどのように接続するか
- どのように利用されるか
- どのような利用方法を想定しているか
- 何ができ、どのような制約があるか

[Overview.ja.md](/docs/Overview.ja.md) / [Overview.en.md](/docs/Overview.en.md) にも
アーキテクチャ図、モジュール構成図、利用構成図がありますが、
そちらはWhitebase全体の概要を把握するためのものです。

このドキュメントでは、各Layerの責務と設計意図をより正確に定義し、
実装と設計の間にずれが生じないようにします。

また、実装の変化によって責務や依存方向にずれが生じた場合に、
実装と設計のどちらを修正するべきか判断できる基準として使用します。

ここで定義するのは、各Layerの論理的な責務と関係です。
実際のビルド・リンク依存関係については、
各CrateおよびNative実装のビルド定義を参照してください。

---

## Layers

現在のWhitebaseを構成している部品を、
それぞれの責務から整理すると以下のLayerに分類できます。

Layer番号は単純な実行順序を示すものではありません。

各LayerがWhitebaseの中で担当する責務と、
その責務の位置関係を整理するための番号です。

<table>
  <thead>
    <tr>
      <th>Layer</th>
      <th>位置づけ</th>
      <th>Component</th>
      <th>Role</th>
      <th>Relation</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>L1<br>Backend Contract</td>
      <td>共通契約のベース</td>
      <td><code>whitebase-backend-contract</code></td>
      <td>Backend共通の型・Capability・Operation・Backend Contractを定義する</td>
      <td>L3 Backend Integration / L4 Coreから参照される</td>
    </tr>
    <tr>
      <td rowspan="3">L2<br>Backend Implementation</td>
      <td rowspan="3">実計算</td>
      <td><code>whitebase-rust-backend</code></td>
      <td>RustによるScalar / SIMD計算を実装する</td>
      <td rowspan="3">L3 Backend Integrationから利用される</td>
    </tr>
    <tr>
      <td>C++ Native</td>
      <td>C++によるScalar / AVX計算を実装する</td>
    </tr>
    <tr>
      <td>Assembly Native</td>
      <td>AssemblyによるScalar / AVX計算を実装する</td>
    </tr>
    <tr>
      <td rowspan="5">L3<br>Backend Integration</td>
      <td rowspan="5">Backend接続・統合</td>
      <td><code>whitebase-cpp-adapter</code></td>
      <td>C++ Native実装をRust側から利用可能にする</td>
      <td rowspan="5">
        L1のContractを利用し、
        L2または外部実行環境の実装をL4 Coreへ接続する
      </td>
    </tr>
    <tr>
      <td><code>whitebase-asm-adapter</code></td>
      <td>Assembly Native実装をRust側から利用可能にする</td>
    </tr>
    <tr>
      <td><code>whitebase-windows-gnu-adapter</code></td>
      <td>Windows GNU Native実装をRust側へ接続する</td>
    </tr>
    <tr>
      <td><code>whitebase-cerune-vm-adapter</code></td>
      <td>Cerune bytecode / VMをWhitebase Backendとして利用可能にする</td>
    </tr>
    <tr>
      <td><code>whitebase-backend-bridge</code></td>
      <td>各Backendを共通の<code>ComputeBackend</code>としてCoreへ統合する</td>
    </tr>
    <tr>
      <td>L4<br>Core</td>
      <td>Pure Compute</td>
      <td><code>whitebase-core</code></td>
      <td>Backendの登録・選択・Capability確認・availability確認・dispatchを行い、Whitebaseの基礎計算を提供する</td>
      <td>L1 / L3を利用し、L5 Runner / L6 Interfaceから利用される</td>
    </tr>
    <tr>
      <td>L5<br>Runner</td>
      <td>Applied Compute</td>
      <td><code>whitebase-runner</code></td>
      <td>CoreのPure Computeを利用し、比較・計測・観測・検証・実験などの応用処理を構成する</td>
      <td>L4 Coreを利用し、L6 Interfaceから利用される</td>
    </tr>
    <tr>
      <td rowspan="4">L6<br>Interface</td>
      <td rowspan="4">Application Boundary / Interface Adapter</td>
      <td><code>whitebase-interface</code></td>
      <td>
        Core / Runnerを利用するtransport非依存のApplication API、
        Request / Response、Interface Errorを提供する
      </td>
      <td rowspan="4">
        <code>whitebase-interface</code>を共通Application Boundaryとし、
        transport固有のInterface AdapterからWhitebaseの機能を公開する
      </td>
    </tr>
    <tr>
      <td>HTTP API<br><code>whitebase-http-api</code></td>
      <td>
        <code>whitebase-interface</code>をHTTP / JSONへ変換するAxum Interface Adapter
      </td>
    </tr>
    <tr>
      <td>Tauri Command API<br><code>whitebase-tauri-api</code></td>
      <td>
        <code>whitebase-interface</code>をTauri IPCへ変換するInterface Adapter
      </td>
    </tr>
    <tr>
      <td>WASM API<br><code>whitebase-wasm</code></td>
      <td>
        <code>whitebase-interface</code>をWebAssembly / JavaScriptへ変換するInterface Adapter
      </td>
    </tr>
    <tr>
      <td>L7<br>Presentation</td>
      <td>User Facing</td>
      <td>Whitebase App / Browser UI</td>
      <td>Interfaceを利用し、操作・表示・Visualizationを提供する</td>
      <td>L6 Interfaceを利用する</td>
    </tr>
  </tbody>
</table>

---

## Layerの関係

上記のLayerは、単純にL1からL7まで順番に呼び出される構造ではありません。

特にBackend Contractは実行経路そのものではなく、
Backend IntegrationとCoreが共有する契約を定義する基盤です。

そのため、Layerの関係は「実行経路」と「共通契約」を分けて考えます。

### Backend Contract

Backend Contractは、Backend IntegrationとCoreの双方から利用される共通契約です。

![Backend Contract](/docs/diagrams/Layer/Backend-Contract.svg)

### Backend Integration

Backend Integrationは、Backend固有の実装や外部実行環境を
Whitebaseの共通`ComputeBackend`としてCoreへ接続します。

現在の主な統合経路は以下です。

- Rust Scalar / SIMD
- C++ Native
- Assembly Native
- Windows GNU Native
- Cerune VM

Cerune VM統合もL3 Backend Integrationの責務として扱います。

`whitebase-cerune-vm-adapter`は、Ceruneのbytecode / VMを
Whitebaseの`ComputeBackend`として利用可能な形へ適合します。

Cerune固有のbytecode、VM値、function handleなどは
`whitebase-core`へ公開しません。

将来CeruneのC / LLVM / QBE / WAT / Assembly / Native Objectなどの
Artifact経路を追加する場合も、
各経路をL3の独立したAdapterとして接続します。

詳細は [Cerune Integration](Cerune-Integration.md) を参照してください。

### Pure Compute

Pure Computeを直接利用する場合の基本的な経路です。

![Pure Compute](/docs/diagrams/Layer/Pure-Compute.svg)

### Applied Compute

Runnerを利用する場合は、CoreのPure Computeを応用処理として利用します。

![Applied Compute](/docs/diagrams/Layer/Applied-Compute.svg)

### Interfaceからの接続

`whitebase-interface`は、公開する機能によってCoreまたはRunnerを利用します。

- Pure Computeを公開する場合はCoreを利用する
- Applied Computeを公開する場合はRunnerを利用する

HTTP、Tauri IPC、WebAssembly / JavaScriptなどのtransport固有処理は、
それぞれのInterface Adapterが担当します。

現在の主な構成は以下です。

- `whitebase-http-api`: HTTP / JSON / Axum Interface Adapter
- `whitebase-tauri-api`: Tauri IPC Interface Adapter
- `whitebase-wasm`: WebAssembly / JavaScript Interface Adapter

各Interface Adapterは`whitebase-interface`を共通Application Boundaryとして利用し、
Core / Runner固有のApplication処理をtransport側へ重複させない構成とします。

`whitebase-server`などの実行HostはInterface Adapterを起動する役割のみを持ち、
Application Boundaryそのものには含めません。

---

## Layer外の構成要素

7 Layerとは別に、FFI BoundaryとOperations Planeがあります。

| 区分 | 主な構成要素 | 役割 |
| --- | --- | --- |
| FFI Boundary | `whitebase-c-api` | Native consumer向けにWhitebase CoreをC ABIとして公開する |
| Operations Plane | Control Center、scripts、CI / GitHub Actions | 各Layerのbuild・test・run・release・検証・管理を行う |

### FFI Boundary

`whitebase-c-api`はApplication Interfaceとは分けて扱います。

Native consumerがWhitebase Coreをライブラリとして利用するための
FFI Boundaryです。

![FFI Boundary](/docs/diagrams/Layer/FFI-Boundary.svg)

### Operations Plane

Operations Planeは8番目のLayerではありません。

Whitebase全体を横断し、
各Layerを構築・実行・検証・管理するための領域として扱います。

対象には以下が含まれます。

- Control Center
- scripts
- build
- test
- lint
- run
- package
- release
- CI / GitHub Actions
- environment diagnosis

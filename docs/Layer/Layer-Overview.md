# Layer-Overview

## 目的

WhitebaseのLayerを整理するためのドキュメントです。
上記を一目でわかるようにすることが目的になります。

以下を明記して、定義します。

- この層の役割はなにか
- この層の目的はなにか
- どのようなつながりを持っているか
- どのように使っていくか
- 想定される使用方法
- なにが出来て、どんな制約があるのか

注意点として、[Overview.md](/docs/Overview.md) / [Overview.ja.md](/docs/Overview.ja.md) にも図と文書はありますが、
アーキテクチャ、モジュール構成図、利用構成図等の概要を掴むための図とはわけて考えます。

このドキュメントでは、より正確な実装意図を記載し、
実際の構成と設計意図がずれないようにします。

また、実装の変化によって責務や方向性にずれが生じた場合、
どちらを修正するべきか判断できるよう、
Whitebaseの構成を一段深いところで定義します。

ここは各Layerの責務と論理的な関係を定義するものです。実際のビルド・リンク依存関係については、各CrateおよびNative実装のビルド定義を参照してください。

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
      <td><code>whitebase-interface</code></td>
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
      <td rowspan="4">L3<br>Backend Integration</td>
      <td rowspan="4">Backend接続・統合</td>
      <td><code>whitebase-cpp-adapter</code></td>
      <td>C++ Native実装をRust側から利用可能にする</td>
      <td rowspan="4">L1のContractを利用し、L2の実装をL4 Coreへ接続する</td>
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
      <td><code>whitebase-backend-bridge</code></td>
      <td>各Backendを共通のBackendとしてCoreへ統合する</td>
    </tr>
    <tr>
      <td>L4<br>Core</td>
      <td>Pure Compute</td>
      <td><code>whitebase-core</code></td>
      <td>Backendの登録・選択・Capability確認・dispatchを行い、Whitebaseの基礎計算を提供する</td>
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
      <td rowspan="3">L6<br>Interface</td>
      <td rowspan="3">Application Boundary</td>
      <td>HTTP API<br><code>whitebase-server</code></td>
      <td>Whitebaseの機能をHTTP / JSONとして公開する</td>
      <td rowspan="3">Pure ComputeはL4、Applied ComputeはL5を利用し、L7 Presentationへ機能を公開する</td>
    </tr>
    <tr>
      <td>Tauri Command API</td>
      <td>Whitebaseの機能をTauri IPCとして公開する</td>
    </tr>
    <tr>
      <td>WASM API<br><code>whitebase-wasm</code></td>
      <td>Whitebaseの機能をWebAssembly / JavaScriptへ公開する</td>
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

### Pure Compute

Pure Computeを直接利用する場合の基本的な経路です。

![Pure Compute](/docs/diagrams/Layer/Pure-Compute.svg)

### Applied Compute

Runnerを利用する場合は、CoreのPure Computeを応用処理として利用します。

![Applied Compute](/docs/diagrams/Layer/Applied-Compute.svg)

### Interfaceからの接続

Interfaceは、公開する機能によってCoreまたはRunnerを利用します。

- Pure Computeを公開する場合はCoreを利用する
- Applied Computeを公開する場合はRunnerを利用する

HTTP API、Tauri Command API、WASM APIは、
それぞれ異なる実行環境へWhitebaseの機能を公開しますが、
Whitebase内部で担当する責務は同じInterface Layerとして扱います。

## Layer外の構成要素

7 Layerとは別に、FFI BoundaryとOperationsがあります。

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

Operationsは8番目のLayerではありません。

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
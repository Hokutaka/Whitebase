# Whitebase HTTP API

[English](HTTP-API.en.md) | **日本語**

Whitebase Serverが、Browserや他のLocal ClientからWhitebaseの演算を実行するために公開するLocal HTTP/JSON APIです。

> [!IMPORTANT]
> Whitebaseは実験用Repositoryです。OperationやBackendの追加に伴い、APIやResponse形式は変更される可能性があります。

## Base URL

```text
http://127.0.0.1:1430
```

現在のServerはloopback interfaceのみにbindします。

Repository rootから起動します。

```powershell
cargo run -p whitebase-server
```

## Endpoint一覧

| Method | Path | 用途 |
| --- | --- | --- |
| `GET` | `/api/health` | Whitebase Serverが起動しているか確認。 |
| `POST` | `/api/observations/add-scalar-f64` | 対応Backendを横断してScalar `f64`加算を観察。 |
| `POST` | `/api/benchmarks/run` | OperationとPrecisionを指定してBenchmarkを実行。 |
| `POST` | `/api/benchmarks/add-array` | 配列加算用の互換Endpoint。 |
| `POST` | `/api/benchmarks/add-f32` | `f32`配列加算用のLegacy Endpoint。 |

## CORS

現在、Browser requestは次のOriginから許可します。

- `http://localhost:1420`
- `http://127.0.0.1:1420`
- `https://hokutaka.github.io`

許可Methodは`GET`と`POST`で、request headerとして`Content-Type`を許可します。

## Health Check

### `GET /api/health`

例:

```powershell
Invoke-RestMethod `
  -Uri "http://127.0.0.1:1430/api/health"
```

Response:

```json
{
  "status": "ok",
  "service": "whitebase-server"
}
```

## Benchmark API

### `POST /api/benchmarks/run`

Benchmark用の主要Endpointです。

Server側で入力を生成し、指定OperationをWhitebase Runner経由で実行します。利用可能なBackendを参照結果と比較し、計測結果と比較結果を返します。

### Request

```json
{
  "operation": "sum-f64",
  "precision": "f64",
  "inputLength": 1000000,
  "warmupIterations": 10,
  "measuredIterations": 100
}
```

| Field | Type | 必須 | 値・上限 | 説明 |
| --- | --- | ---: | --- | --- |
| `operation` | string | No | `add-array`, `sum-f64` | Benchmark対象。省略時は`add-array`。 |
| `precision` | string | Yes | `f32`, `f64` | 浮動小数点Precision。 |
| `inputLength` | integer | Yes | `1..=10,000,000` | 生成する入力要素数。 |
| `warmupIterations` | integer | Yes | `0..=10,000` | 計測前のwarm-up回数。 |
| `measuredIterations` | integer | Yes | `1..=10,000` | 計測回数。 |

対応するOperation / Precisionの組み合わせ:

| Operation | `f32` | `f64` |
| --- | ---: | ---: |
| `add-array` | 対応 | 対応 |
| `sum-f64` | 非対応 | 対応 |

`sum-f64`に`precision: "f32"`を指定すると、HTTP `400` / `invalid_benchmark_precision`を返します。

### 入力生成

Benchmark配列はrequestで受け取らず、Server側で生成します。

配列加算では左辺・右辺の両配列を生成します。`sum-f64`では`f64`配列Benchmarkと同じ左辺入力を生成し、その配列を1つの合計値へreduceします。

### Response

時間はnanosecond単位です。

実際に含まれるBackendはPlatformとBackend availabilityによって変わります。

```json
{
  "operation": "sum-f64",
  "precision": "f64",
  "inputLength": 1000000,
  "referenceBackend": "<reference backend>",
  "warmupIterations": 10,
  "measuredIterations": 100,
  "absoluteTolerance": 1e-12,
  "results": [
    {
      "backend": "<backend display name>",
      "status": "completed",
      "timingStatus": "measured",
      "iterations": 100,
      "totalNanoseconds": 1234567,
      "minimumNanoseconds": 12000,
      "maximumNanoseconds": 13000,
      "meanNanoseconds": 12345.67,
      "matchesReference": true,
      "mismatchCount": 0,
      "maximumAbsoluteError": 0.0,
      "error": null
    }
  ]
}
```

上記の数値は例です。実際の結果はBuild設定、CPU、Cache、Memory bandwidth、OS schedulingなどの実行環境に依存します。

### Backend Result Status

`results`の各要素は次のいずれかのStatusを持ちます。

| Status | 意味 |
| --- | --- |
| `completed` | Backendの実行に成功。比較Fieldが設定され、Timing Fieldは`timingStatus`に従う。 |
| `unavailable` | 現在のPlatformまたはCPUでは利用不可。Timing / Comparison Fieldは`null`。 |
| `failed` | Backendは利用可能だが実行に失敗。`error`に失敗内容を格納。 |

### Timing Status

実行に成功したBackendには次のいずれかの`timingStatus`が入ります。

| Timing status | 意味 |
| --- | --- |
| `measured` | 全ての計測iterationで観測可能な実行時間を取得できた。 |
| `too-fast-to-measure` | 1つ以上のiterationが現在のTimer分解能未満だった。 |

`timingStatus`が`too-fast-to-measure`でもBackend statusは`completed`のままです。演算と参照結果との比較は有効ですが、時間値は報告せず、fastest / speedup計算から除外します。

この場合、`iterations`、`totalNanoseconds`、`minimumNanoseconds`、`maximumNanoseconds`、`meanNanoseconds`は`null`です。

実行成功時:

- `matchesReference`は、設定されたTolerance内で参照結果と一致したかを示します。
- `mismatchCount`はRunnerが報告した不一致要素または値の数です。
- `maximumAbsoluteError`は観測した最大絶対誤差です。有限値を報告できない場合は`null`になることがあります。

### PowerShell: `sum-f64`

```powershell
$body = @{
  operation = "sum-f64"
  precision = "f64"
  inputLength = 1000000
  warmupIterations = 10
  measuredIterations = 100
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri "http://127.0.0.1:1430/api/benchmarks/run" `
  -ContentType "application/json" `
  -Body $body
```

### PowerShell: `add-array` / `f32`

```powershell
$body = @{
  operation = "add-array"
  precision = "f32"
  inputLength = 1000000
  warmupIterations = 10
  measuredIterations = 100
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri "http://127.0.0.1:1430/api/benchmarks/run" `
  -ContentType "application/json" `
  -Body $body
```

### curl: `sum-f64`

```bash
curl -X POST http://127.0.0.1:1430/api/benchmarks/run \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "sum-f64",
    "precision": "f64",
    "inputLength": 1000000,
    "warmupIterations": 10,
    "measuredIterations": 100
  }'
```

## Compatibility Benchmark Endpoints

### `POST /api/benchmarks/add-array`

`/api/benchmarks/run`と同じrequest形式を使いますが、Server側では常に`add-array`として実行します。

`operation`を指定しても無視し、`add-array`へ置き換えます。

例:

```json
{
  "precision": "f64",
  "inputLength": 1000000,
  "warmupIterations": 10,
  "measuredIterations": 100
}
```

### `POST /api/benchmarks/add-f32`

`f32`配列加算用のLegacy Endpointです。

Benchmarkのサイズ指定だけを受け取ります。

```json
{
  "inputLength": 1000000,
  "warmupIterations": 10,
  "measuredIterations": 100
}
```

Server側では次の指定として実行します。

```text
operation = add-array
precision = f32
```

新しくAPIを利用する場合は`/api/benchmarks/run`を推奨します。

## Scalar `f64` Observation API

### `POST /api/observations/add-scalar-f64`

`AddScalarF64` capabilityを宣言する登録済みBackendを横断して、Scalar IEEE 754 `f64`加算を観察するEndpointです。

Benchmark APIとは異なり、`lhs`と`rhs`はClientが10進文字列として指定します。RunnerはBackend実行とは独立して正確な10進参照値を計算し、実行に成功した各Backendの結果を、その参照値を正しく丸めた`f64`とbit単位で比較します。

Backendが利用不可または実行失敗しても、観察全体は中断しません。状態を`results`に保持したまま、残りの対応Backendの観察を継続します。

### Request

```json
{
  "lhs": "0.1",
  "rhs": "0.2"
}
```

### Response Field

| Field | 説明 |
| --- | --- |
| `lhsInput` | 入力した左辺の10進文字列。 |
| `rhsInput` | 入力した右辺の10進文字列。 |
| `lhs` | Parse後の`f64`値、10進表示、bit pattern。 |
| `rhs` | Parse後の`f64`値、10進表示、bit pattern。 |
| `decimalReference` | Runnerが計算した正確な10進加算結果。 |
| `reference` | 正確な10進参照値を最も近い`f64`へ丸めた値、10進表示、bit pattern。 |
| `results` | Backendごとの実行状態、結果、bit単位の比較、Error情報。 |
| `allBackendsMatch` | 実行に成功した全Backendの結果bitが互いに一致しているか。成功したBackendがない場合は`false`。 |

`lhs`、`rhs`、`reference`、実行成功したBackendの`result`は次の形式です。

```json
{
  "value": 0.3,
  "decimal": "0.29999999999999999",
  "bits": "0x3fd3333333333333"
}
```

`0.1 + 0.2`では、正確な10進参照値は`0.3`です。これを正しく丸めた参照`f64`のbitは`0x3fd3333333333333`ですが、通常のbinary `f64`加算結果は`0x3fd3333333333334`になります。そのためBackend実行自体が成功していても、`matchesReferenceBits`が`false`になる場合があります。

### Scalar Backend Status

各Backendの観察結果には、次のいずれかのStatusが入ります。

| Status | `result` | `matchesReferenceBits` | `error` | 意味 |
| --- | --- | --- | --- | --- |
| `completed` | object | boolean | `null` | Backendの実行に成功。 |
| `unavailable` | `null` | `null` | `null` | Scalar `f64`をサポートするが、現在の環境では利用不可。 |
| `failed` | `null` | `null` | string | Backendは利用可能だが実行に失敗。残りのBackend観察は継続。 |

実行成功時:

```json
{
  "backend": "<backend display name>",
  "status": "completed",
  "result": {
    "value": 0.30000000000000004,
    "decimal": "0.30000000000000004",
    "bits": "0x3fd3333333333334"
  },
  "matchesReferenceBits": false,
  "error": null
}
```

利用不可の場合:

```json
{
  "backend": "<backend display name>",
  "status": "unavailable",
  "result": null,
  "matchesReferenceBits": null,
  "error": null
}
```

実行失敗時:

```json
{
  "backend": "<backend display name>",
  "status": "failed",
  "result": null,
  "matchesReferenceBits": null,
  "error": "<backend failure message>"
}
```

`matchesReferenceBits`は実行に成功したBackendでのみ設定され、そのBackendの結果が、正確な10進参照値を丸めた`f64`とbit単位で一致するかを表します。

`allBackendsMatch`は別の比較です。実行に成功したBackend同士の結果bitを比較する値であり、全Backendが`reference`と一致したことを意味しません。

### PowerShell例

```powershell
$body = @{
  lhs = "0.1"
  rhs = "0.2"
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri "http://127.0.0.1:1430/api/observations/add-scalar-f64" `
  -ContentType "application/json" `
  -Body $body
```

## Error

API Errorは次のJSON形式です。

```json
{
  "code": "input_length_zero",
  "message": "input length must be greater than zero"
}
```

現在のBenchmark validation error:

| HTTP status | Code | 条件 |
| ---: | --- | --- |
| `400` | `input_length_zero` | `inputLength`が`0`。 |
| `400` | `input_length_too_large` | `inputLength`が`10,000,000`を超える。 |
| `400` | `measured_iterations_zero` | `measuredIterations`が`0`。 |
| `400` | `warmup_iterations_too_large` | `warmupIterations`が`10,000`を超える。 |
| `400` | `measured_iterations_too_large` | `measuredIterations`が`10,000`を超える。 |
| `400` | `invalid_benchmark_precision` | `sum-f64`に`f64`以外のPrecisionを指定。 |

Scalar Observationでは、不正なScalar入力または範囲外の正確な10進参照値に対してHTTP `400` / `invalid_scalar_f64_request`を返します。

Scalar Observation中のBackend実行失敗は、Top-levelのHTTP `500` Errorにはしません。`results`内の`status: "failed"`として保持し、残りの対応Backendの観察を継続します。

Interface内部の予期しない失敗やbackground taskの失敗は、`code`と`message`を含むHTTP `500` Errorとして返る場合があります。

## Notes

- 現在のServerは`127.0.0.1:1430`でlistenし、全network interfaceには公開しません。
- 現在のServerには認証Layerを実装していません。
- 新しいBenchmark Clientでは`/api/benchmarks/run`の利用を推奨します。
- 性能比較を目的とするBenchmarkではRelease buildを推奨します。

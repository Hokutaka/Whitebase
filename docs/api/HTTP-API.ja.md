# Whitebase HTTP API

Whitebase Serverは、ブラウザやローカルクライアントからWhitebaseの計算処理を実行するためのHTTP/JSON APIを提供します。

> [!IMPORTANT]
> Whitebaseは学習・実験用のリポジトリです。OperationやBackendの追加に伴い、APIやレスポンス形式が変更される可能性があります。

## Base URL

```text
http://127.0.0.1:1430
```

現在のServerはloopback interfaceのみにbindします。

リポジトリルートから起動します。

```powershell
cargo run -p whitebase-server
```

## Endpoint一覧

| Method | Path | 用途 |
| --- | --- | --- |
| `GET` | `/api/health` | Whitebase Serverの起動確認。 |
| `POST` | `/api/observations/add-scalar-f64` | Scalar `f64`加算の10進表現・bit表現を観察。 |
| `POST` | `/api/benchmarks/run` | OperationとPrecisionを指定してベンチマークを実行。 |
| `POST` | `/api/benchmarks/add-array` | 配列加算の互換Endpoint。 |
| `POST` | `/api/benchmarks/add-f32` | `f32`配列加算のLegacy Endpoint。 |

## CORS

現在、ブラウザからのリクエストは次のOriginを許可しています。

- `http://localhost:1420`
- `http://127.0.0.1:1420`
- `https://hokutaka.github.io`

許可Methodは`GET`と`POST`、許可Request Headerは`Content-Type`です。

## Health Check

### `GET /api/health`

実行例:

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

現在の主要Benchmark Endpointです。

Server側で入力を生成し、指定されたOperationをWhitebase Runnerで実行します。利用可能な各Backendの結果を参照結果と比較し、実行時間と比較結果を返します。

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

| Field | 型 | 必須 | 値・制限 | 説明 |
| --- | --- | ---: | --- | --- |
| `operation` | string | いいえ | `add-array`, `sum-f64` | 実行するOperation。省略時は`add-array`。 |
| `precision` | string | はい | `f32`, `f64` | 浮動小数点Precision。 |
| `inputLength` | integer | はい | `1..=10,000,000` | Serverが生成する入力要素数。 |
| `warmupIterations` | integer | はい | `0..=10,000` | 計測前のWarmup回数。 |
| `measuredIterations` | integer | はい | `1..=10,000` | 計測する反復回数。 |

対応するOperation / Precisionの組み合わせ:

| Operation | `f32` | `f64` |
| --- | ---: | ---: |
| `add-array` | 対応 | 対応 |
| `sum-f64` | 非対応 | 対応 |

`sum-f64`に`precision: "f32"`を指定した場合、HTTP `400` / `invalid_benchmark_precision`を返します。

### 入力データ

Benchmark用の配列はRequestで送信するのではなく、Server内部で生成します。

配列加算では左辺・右辺の2配列を生成します。`sum-f64`では、`f64`配列加算で左辺に使用するものと同じ形式の入力配列を生成し、その配列を1つの合計値へreduceします。

### Response

時間はnanosecond単位です。

Backend一覧は実行PlatformとBackendの利用可否によって変化します。

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

上記の時間値はResponse形式を示すための例です。実際の結果はBuild構成、CPU、cache、memory bandwidth、OS schedulingなどの影響を受けます。

### Timing Status

`completed`のBackendは、次の`timingStatus`を持ちます。

| timingStatus | 意味 |
| --- | --- |
| `measured` | 全iterationの実行時間を計測できました。 |
| `too-fast-to-measure` | 1回以上のiterationがタイマー分解能未満でした。 |

`too-fast-to-measure`の場合もBackend Statusは`completed`です。
演算結果と参照結果の比較は有効ですが、時間値は`null`となり、
性能比較やSpeedupの対象にはなりません。

### Backend Result Status

`results`の各要素は次のいずれかのStatusになります。

| Status | 意味 |
| --- | --- |
| `completed` | Backendの実行に成功。時間・比較情報が設定されます。 |
| `unavailable` | 現在のPlatformまたはCPUでBackendを利用できません。時間・比較情報は`null`です。 |
| `failed` | Backendは利用対象ですが実行に失敗しました。`error`に失敗内容が入ります。 |

`completed`の場合:

- `matchesReference`: 設定された許容誤差の範囲で参照結果と一致したか。
- `mismatchCount`: Runnerが報告した不一致要素・値の数。
- `maximumAbsoluteError`: 観測された最大絶対誤差。有限値を報告できない場合は`null`になることがあります。

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

## 互換Benchmark Endpoint

### `POST /api/benchmarks/add-array`

`/api/benchmarks/run`と同じBenchmark Request形式を受け取りますが、Serverは常に`add-array`を実行します。

Requestに`operation`を指定した場合も、その値は使用せず`add-array`として実行します。

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

Server側では次の指定として実行されます。

```text
operation = add-array
precision = f32
```

新しくAPIを利用する場合は`/api/benchmarks/run`を推奨します。

## Scalar `f64` Observation API

### `POST /api/observations/add-scalar-f64`

利用可能なBackend間で、Scalar IEEE 754 `f64`加算を観察するためのEndpointです。

Benchmark APIとは異なり、`lhs`と`rhs`はクライアントが10進文字列として指定します。

Request:

```json
{
  "lhs": "0.1",
  "rhs": "0.2"
}
```

Responseには次の情報が含まれます。

| Field | 説明 |
| --- | --- |
| `lhsInput` | 入力した左辺の10進文字列。 |
| `rhsInput` | 入力した右辺の10進文字列。 |
| `lhs` | Parse後の`f64`値、10進表示、bit pattern。 |
| `rhs` | Parse後の`f64`値、10進表示、bit pattern。 |
| `decimalReference` | Runnerが計算した10進参照結果。 |
| `reference` | 参照`f64`値、10進表示、bit pattern。 |
| `results` | Backendごとの結果とbit単位の比較。 |
| `allBackendsMatch` | 報告された全Backendが参照bitと一致したか。 |

`lhs`、`rhs`、`reference`、各Backendの`result`は次の形式です。

```json
{
  "value": 0.3,
  "decimal": "0.29999999999999999",
  "bits": "0x3fd3333333333333"
}
```

各Backendの観察結果:

```json
{
  "backend": "<backend display name>",
  "result": {
    "value": 0.3,
    "decimal": "0.29999999999999999",
    "bits": "0x3fd3333333333333"
  },
  "matchesReferenceBits": true
}
```

PowerShell例:

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

Scalar Observationでは、不正なScalar入力または範囲外の参照値に対して`400` / `invalid_scalar_f64_request`を返します。

実行処理やbackground taskで失敗した場合は、`code`と`message`を含むHTTP `500` Errorを返します。

## Notes

- 現在のServerは`127.0.0.1:1430`でlistenし、全network interfaceには公開しません。
- 現在のServerには認証Layerを実装していません。
- 新しいBenchmark clientでは`/api/benchmarks/run`の利用を推奨します。
- 性能比較を目的とするBenchmarkではRelease buildを推奨します。

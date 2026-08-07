from __future__ import annotations

import subprocess
import sys
from pathlib import Path

EXPECTED_HEAD = "4ef8d75"
ROOT = Path.cwd()


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout


def require_clean_tracked_worktree() -> None:
    for args, label in [
        (("diff", "--quiet"), "unstaged tracked changes"),
        (("diff", "--cached", "--quiet"), "staged changes"),
    ]:
        result = subprocess.run(["git", *args], cwd=ROOT, check=False)
        if result.returncode == 1:
            raise RuntimeError(f"Refusing to run with {label}. Commit or restore them first.")
        if result.returncode != 0:
            raise RuntimeError(f"git {' '.join(args)} failed.")


def head_text(rel: str) -> str:
    result = subprocess.run(
        ["git", "show", f"HEAD:{rel}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"Could not read HEAD:{rel}: {result.stderr.decode(errors='replace').strip()}"
        )
    return result.stdout.decode("utf-8-sig")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def preserve_write(rel: str, text: str) -> None:
    path = ROOT / rel
    current = path.read_bytes()
    bom = current.startswith(b"\xef\xbb\xbf")
    newline = "\r\n" if b"\r\n" in current else ("\r\n" if sys.platform == "win32" else "\n")
    normalized = text.replace("\r\n", "\n").replace("\r", "\n")
    encoded = normalized.replace("\n", newline).encode("utf-8")
    if bom:
        encoded = b"\xef\xbb\xbf" + encoded
    path.write_bytes(encoded)


def main() -> None:
    head = git("rev-parse", "--short=7", "HEAD").strip()
    if head != EXPECTED_HEAD:
        raise RuntimeError(
            f"Expected HEAD {EXPECTED_HEAD}, but found {head}. "
            "This updater is pinned to the pushed Windows GNU NASM commit."
        )

    require_clean_tracked_worktree()
    updated: dict[str, str] = {}

    # ------------------------------------------------------------------
    # Tauri benchmark command / DTO
    # ------------------------------------------------------------------
    rel = "apps/whitebase-app/src-tauri/src/benchmark.rs"
    text = head_text(rel)

    text = replace_once(
        text,
        "    AddF32Report, AddF64Report, BackendRunResult, BackendRunStatus, Runner, RunnerConfig,\n",
        "    AddF32Report, AddF64Report, BackendRunResult, BackendRunStatus, Runner, RunnerConfig,\n"
        "    SumF64Report,\n",
        f"{rel}: import SumF64Report",
    )

    precision_enum = '''/// 配列ベンチマークで使用する浮動小数点精度です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkPrecision {
    F32,
    F64,
}
'''
    operation_and_precision = '''/// ベンチマークで実行する演算です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BenchmarkOperation {
    AddArray,
    SumF64,
}

/// ベンチマークで使用する浮動小数点精度です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkPrecision {
    F32,
    F64,
}
'''
    text = replace_once(
        text,
        precision_enum,
        operation_and_precision,
        f"{rel}: BenchmarkOperation enum",
    )

    text = replace_once(
        text,
        "pub struct BenchmarkRequest {\n"
        "    /// 使用する浮動小数点精度。\n"
        "    pub precision: BenchmarkPrecision,\n",
        "pub struct BenchmarkRequest {\n"
        "    /// 実行する演算。\n"
        "    pub operation: BenchmarkOperation,\n"
        "\n"
        "    /// 使用する浮動小数点精度。\n"
        "    pub precision: BenchmarkPrecision,\n",
        f"{rel}: request operation",
    )

    text = replace_once(
        text,
        "pub struct BenchmarkReportDto {\n"
        "    pub precision: BenchmarkPrecision,\n",
        "pub struct BenchmarkReportDto {\n"
        "    pub operation: BenchmarkOperation,\n"
        "    pub precision: BenchmarkPrecision,\n",
        f"{rel}: report operation",
    )

    command_start = text.find("/// 選択された精度で配列加算をバックグラウンド実行します。\n")
    validate_start = text.find("fn validate_request(request: BenchmarkRequest) -> Result<(), String> {\n", command_start)
    if command_start < 0 or validate_start < 0:
        raise RuntimeError(f"{rel}: benchmark command block boundaries not found")

    command_block = '''/// 選択された演算をバックグラウンドでベンチマークします。
#[tauri::command]
pub async fn run_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, String> {
    tauri::async_runtime::spawn_blocking(move || execute_benchmark(request))
        .await
        .map_err(|error| format!("benchmark task failed: {error}"))?
}

fn execute_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, String> {
    validate_request(request)?;

    let config = RunnerConfig {
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
        ..RunnerConfig::default()
    };

    let runner = Runner::new();

    match request.operation {
        BenchmarkOperation::AddArray => match request.precision {
            BenchmarkPrecision::F32 => {
                let lhs = create_lhs_f32(request.input_length);
                let rhs = create_rhs_f32(request.input_length);

                runner
                    .run_add_f32(&lhs, &rhs, &config)
                    .map(Into::into)
                    .map_err(|error| error.to_string())
            }
            BenchmarkPrecision::F64 => {
                let lhs = create_lhs_f64(request.input_length);
                let rhs = create_rhs_f64(request.input_length);

                runner
                    .run_add_f64(&lhs, &rhs, &config)
                    .map(Into::into)
                    .map_err(|error| error.to_string())
            }
        },
        BenchmarkOperation::SumF64 => {
            if request.precision != BenchmarkPrecision::F64 {
                return Err("sum-f64 benchmark requires f64 precision".to_owned());
            }

            let input = create_lhs_f64(request.input_length);
            runner
                .run_sum_f64(&input, &config)
                .map(Into::into)
                .map_err(|error| error.to_string())
        }
    }
}

'''
    text = text[:command_start] + command_block + text[validate_start:]

    # Report conversions.
    text = replace_once(
        text,
        "        Self {\n"
        "            precision: BenchmarkPrecision::F32,\n",
        "        Self {\n"
        "            operation: BenchmarkOperation::AddArray,\n"
        "            precision: BenchmarkPrecision::F32,\n",
        f"{rel}: AddF32 operation",
    )
    text = replace_once(
        text,
        "        Self {\n"
        "            precision: BenchmarkPrecision::F64,\n"
        "            input_length: report.input_length,\n",
        "        Self {\n"
        "            operation: BenchmarkOperation::AddArray,\n"
        "            precision: BenchmarkPrecision::F64,\n"
        "            input_length: report.input_length,\n",
        f"{rel}: AddF64 operation",
    )

    backend_result_impl = text.find("impl From<BackendRunResult> for BackendResultDto {")
    if backend_result_impl < 0:
        raise RuntimeError(f"{rel}: BackendRunResult conversion not found")
    sum_conversion = '''impl From<SumF64Report> for BenchmarkReportDto {
    fn from(report: SumF64Report) -> Self {
        Self {
            operation: BenchmarkOperation::SumF64,
            precision: BenchmarkPrecision::F64,
            input_length: report.input_length,
            reference_backend: report.reference_backend.display_name().to_owned(),
            warmup_iterations: report.warmup_iterations,
            measured_iterations: report.measured_iterations,
            absolute_tolerance: report.absolute_tolerance,
            results: report
                .results
                .into_iter()
                .map(BackendResultDto::from)
                .collect(),
        }
    }
}

'''
    text = text[:backend_result_impl] + sum_conversion + text[backend_result_impl:]

    # Replace test module with operation-aware tests.
    tests_start = text.find("#[cfg(test)]\nmod tests {\n")
    if tests_start < 0:
        raise RuntimeError(f"{rel}: tests module not found")
    tests = '''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_add_benchmark_reports_selected_operation_and_precision() {
        let report = execute_benchmark(BenchmarkRequest {
            operation: BenchmarkOperation::AddArray,
            precision: BenchmarkPrecision::F64,
            input_length: 17,
            warmup_iterations: 1,
            measured_iterations: 2,
        })
        .expect("f64 add benchmark must succeed");

        assert_eq!(report.operation, BenchmarkOperation::AddArray);
        assert_eq!(report.precision, BenchmarkPrecision::F64);
        assert_eq!(report.input_length, 17);
        assert!(report.results.iter().all(|result| {
            result.status == "unavailable" || result.matches_reference == Some(true)
        }));
    }

    #[test]
    fn sum_f64_benchmark_uses_reduction_runner() {
        let report = execute_benchmark(BenchmarkRequest {
            operation: BenchmarkOperation::SumF64,
            precision: BenchmarkPrecision::F64,
            input_length: 17,
            warmup_iterations: 1,
            measured_iterations: 2,
        })
        .expect("sum f64 benchmark must succeed");

        assert_eq!(report.operation, BenchmarkOperation::SumF64);
        assert_eq!(report.precision, BenchmarkPrecision::F64);
        assert_eq!(report.input_length, 17);
        assert!(report.results.iter().all(|result| {
            result.status == "unavailable" || result.matches_reference == Some(true)
        }));
    }
}
'''
    text = text[:tests_start] + tests
    updated[rel] = text

    # ------------------------------------------------------------------
    # Tauri registration
    # ------------------------------------------------------------------
    rel = "apps/whitebase-app/src-tauri/src/lib.rs"
    text = head_text(rel)
    text = replace_once(
        text,
        "use benchmark::run_add_benchmark;\n",
        "use benchmark::run_benchmark;\n",
        f"{rel}: benchmark import",
    )
    text = replace_once(
        text,
        "            run_add_benchmark,\n",
        "            run_benchmark,\n",
        f"{rel}: benchmark handler",
    )
    updated[rel] = text

    # ------------------------------------------------------------------
    # Frontend
    # ------------------------------------------------------------------
    rel = "apps/whitebase-app/src/main.ts"
    text = head_text(rel)

    text = replace_once(
        text,
        'type BenchmarkPrecision = "f32" | "f64";\n',
        'type BenchmarkOperation = "add-array" | "sum-f64";\n'
        'type BenchmarkPrecision = "f32" | "f64";\n',
        f"{rel}: operation type",
    )
    text = replace_once(
        text,
        "interface BenchmarkRequest {\n"
        "  precision: BenchmarkPrecision;\n",
        "interface BenchmarkRequest {\n"
        "  operation: BenchmarkOperation;\n"
        "  precision: BenchmarkPrecision;\n",
        f"{rel}: request operation",
    )
    text = replace_once(
        text,
        "interface BenchmarkReport {\n"
        "  precision: BenchmarkPrecision;\n",
        "interface BenchmarkReport {\n"
        "  operation: BenchmarkOperation;\n"
        "  precision: BenchmarkPrecision;\n",
        f"{rel}: report operation",
    )

    text = replace_once(
        text,
        '          <p class="eyebrow">F32 / F64 ARRAY</p>\n',
        '          <p id="benchmark-eyebrow" class="eyebrow">F32 / F64 ARRAY</p>\n',
        f"{rel}: benchmark eyebrow id",
    )

    benchmark_form = text.find('      <form id="benchmark-form">\n')
    precision_fieldset = text.find('        <fieldset class="precision-control">\n', benchmark_form)
    if benchmark_form < 0 or precision_fieldset < 0:
        raise RuntimeError(f"{rel}: benchmark form boundaries not found")

    operation_markup = '''        <fieldset class="operation-control">
          <legend>Operation</legend>

          <div class="operation-options">
            <label class="operation-option">
              <input type="radio" name="benchmark-operation" value="add-array" checked />
              <span>Add</span>
            </label>
            <label class="operation-option">
              <input type="radio" name="benchmark-operation" value="sum-f64" />
              <span>Sum f64</span>
            </label>
          </div>
        </fieldset>

'''
    text = text[:precision_fieldset] + operation_markup + text[precision_fieldset:]

    # Add Operation metric before Precision.
    summary_precision = '''      <div class="metric">
        <span>Precision</span>
        <strong id="summary-precision">-</strong>
      </div>
'''
    summary_operation = '''      <div class="metric">
        <span>Operation</span>
        <strong id="summary-operation">-</strong>
      </div>

''' + summary_precision
    text = replace_once(
        text,
        summary_precision,
        summary_operation,
        f"{rel}: operation summary metric",
    )

    # Add operation to request.
    text = replace_once(
        text,
        "  const request: BenchmarkRequest = {\n"
        "    precision: readBenchmarkPrecision(),\n",
        "  const request: BenchmarkRequest = {\n"
        "    operation: readBenchmarkOperation(),\n"
        "    precision: readBenchmarkPrecision(),\n",
        f"{rel}: submit operation",
    )

    # Sync operation/precision behavior after element declarations.
    declaration_anchor = 'const summary = requireElement<HTMLElement>("#summary");\n'
    controls = declaration_anchor + '''const f32PrecisionInput = requireElement<HTMLInputElement>(
  'input[name="benchmark-precision"][value="f32"]',
);
const f64PrecisionInput = requireElement<HTMLInputElement>(
  'input[name="benchmark-precision"][value="f64"]',
);

document
  .querySelectorAll<HTMLInputElement>('input[name="benchmark-operation"]')
  .forEach((input) => input.addEventListener("change", syncBenchmarkControls));

syncBenchmarkControls();
'''
    text = replace_once(
        text,
        declaration_anchor,
        controls,
        f"{rel}: operation controls",
    )

    # Add operation summary rendering before precision.
    text = replace_once(
        text,
        '  setText("summary-precision", report.precision.toUpperCase());\n',
        '  setText(\n'
        '    "summary-operation",\n'
        '    report.operation === "sum-f64" ? "Sum f64" : "Add array",\n'
        '  );\n'
        '  setText("summary-precision", report.precision.toUpperCase());\n',
        f"{rel}: render operation summary",
    )

    # Insert helper functions before readBenchmarkPrecision.
    helper_anchor = "function readBenchmarkPrecision(): BenchmarkPrecision {\n"
    helper_block = '''function syncBenchmarkControls(): void {
  const operation = readBenchmarkOperation();
  const sumSelected = operation === "sum-f64";

  f32PrecisionInput.disabled = sumSelected;
  if (sumSelected) {
    f64PrecisionInput.checked = true;
  }

  setText(
    "benchmark-eyebrow",
    sumSelected ? "F64 REDUCTION" : "F32 / F64 ARRAY",
  );
}

function readBenchmarkOperation(): BenchmarkOperation {
  const input = requireElement<HTMLInputElement>(
    'input[name="benchmark-operation"]:checked',
  );

  if (input.value !== "add-array" && input.value !== "sum-f64") {
    throw new Error(`unsupported benchmark operation: ${input.value}`);
  }

  return input.value;
}

'''
    text = replace_once(
        text,
        helper_anchor,
        helper_block + helper_anchor,
        f"{rel}: operation helpers",
    )

    text = replace_once(
        text,
        '    return invoke<BenchmarkReport>("run_add_benchmark", { request });\n',
        '    return invoke<BenchmarkReport>("run_benchmark", { request });\n',
        f"{rel}: Tauri command",
    )
    text = replace_once(
        text,
        '    response = await fetch(`${API_BASE_URL}/api/benchmarks/add-array`, {\n',
        '    response = await fetch(`${API_BASE_URL}/api/benchmarks/run`, {\n',
        f"{rel}: HTTP benchmark endpoint",
    )
    updated[rel] = text

    # ------------------------------------------------------------------
    # Frontend CSS
    # ------------------------------------------------------------------
    rel = "apps/whitebase-app/src/styles.css"
    text = head_text(rel)

    text = replace_once(
        text,
        "#benchmark-form {\n"
        "  grid-template-columns: minmax(150px, 0.8fr) repeat(3, minmax(150px, 1fr)) auto;\n"
        "}\n",
        "#benchmark-form {\n"
        "  grid-template-columns: repeat(2, minmax(150px, 0.8fr)) repeat(3, minmax(150px, 1fr)) auto;\n"
        "}\n",
        f"{rel}: benchmark grid",
    )

    text = replace_once(
        text,
        ".precision-control {\n"
        "  min-width: 0;\n",
        ".operation-control,\n"
        ".precision-control {\n"
        "  min-width: 0;\n",
        f"{rel}: fieldset selector",
    )
    text = replace_once(
        text,
        ".precision-control legend {\n",
        ".operation-control legend,\n"
        ".precision-control legend {\n",
        f"{rel}: legend selector",
    )
    text = replace_once(
        text,
        ".precision-options {\n",
        ".operation-options,\n"
        ".precision-options {\n",
        f"{rel}: options selector",
    )
    text = replace_once(
        text,
        ".precision-option {\n",
        ".operation-option,\n"
        ".precision-option {\n",
        f"{rel}: option selector",
    )
    text = replace_once(
        text,
        ".precision-option input {\n",
        ".operation-option input,\n"
        ".precision-option input {\n",
        f"{rel}: option input selector",
    )
    text = replace_once(
        text,
        ".precision-option span {\n",
        ".operation-option span,\n"
        ".precision-option span {\n",
        f"{rel}: option span selector",
    )
    text = replace_once(
        text,
        ".precision-option input:checked + span {\n",
        ".operation-option input:checked + span,\n"
        ".precision-option input:checked + span {\n",
        f"{rel}: checked selector",
    )
    text = replace_once(
        text,
        ".precision-option input:focus-visible + span {\n"
        "  box-shadow: 0 0 0 3px #70a7ff35;\n"
        "}\n",
        ".operation-option input:focus-visible + span,\n"
        ".precision-option input:focus-visible + span {\n"
        "  box-shadow: 0 0 0 3px #70a7ff35;\n"
        "}\n"
        "\n"
        ".precision-option input:disabled + span {\n"
        "  cursor: not-allowed;\n"
        "  opacity: 0.35;\n"
        "}\n",
        f"{rel}: focus/disabled selector",
    )
    text = replace_once(
        text,
        ".summary {\n"
        "  display: grid;\n"
        "  grid-template-columns: repeat(5, 1fr);\n",
        ".summary {\n"
        "  display: grid;\n"
        "  grid-template-columns: repeat(6, 1fr);\n",
        f"{rel}: summary grid",
    )
    updated[rel] = text

    # ------------------------------------------------------------------
    # HTTP server parity
    # ------------------------------------------------------------------
    rel = "apps/whitebase-server/src/main.rs"
    text = head_text(rel)

    text = replace_once(
        text,
        "    AddF32Report, AddF64Report, BackendRunResult, BackendRunStatus, F64Value, Runner, RunnerConfig,\n"
        "    RunnerError, ScalarF64BackendObservation, ScalarF64ObservationReport,\n",
        "    AddF32Report, AddF64Report, BackendRunResult, BackendRunStatus, F64Value, Runner, RunnerConfig,\n"
        "    RunnerError, ScalarF64BackendObservation, ScalarF64ObservationReport, SumF64Report,\n",
        f"{rel}: import SumF64Report",
    )

    text = replace_once(
        text,
        '        .route("/api/benchmarks/add-array", post(run_add_benchmark))\n',
        '        .route("/api/benchmarks/run", post(run_benchmark))\n'
        '        .route("/api/benchmarks/add-array", post(run_add_benchmark))\n',
        f"{rel}: run route",
    )

    # Make legacy add-array force AddArray, and add generic route.
    handler_start = text.find("async fn run_add_benchmark(\n")
    legacy_start = text.find("async fn run_legacy_add_f32_benchmark(\n", handler_start)
    if handler_start < 0 or legacy_start < 0:
        raise RuntimeError(f"{rel}: benchmark handler boundaries not found")
    handlers = '''async fn run_benchmark(
    Json(request): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    run_benchmark_task(request).await.map(Json)
}

async fn run_add_benchmark(
    Json(mut request): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    request.operation = BenchmarkOperation::AddArray;
    run_benchmark_task(request).await.map(Json)
}

'''
    text = text[:handler_start] + handlers + text[legacy_start:]

    text = replace_once(
        text,
        "    run_benchmark_task(BenchmarkRequest {\n"
        "        precision: BenchmarkPrecision::F32,\n",
        "    run_benchmark_task(BenchmarkRequest {\n"
        "        operation: BenchmarkOperation::AddArray,\n"
        "        precision: BenchmarkPrecision::F32,\n",
        f"{rel}: legacy operation",
    )

    # Rewrite execution match.
    execute_start = text.find("fn execute_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, ApiError> {\n")
    validate_start = text.find("fn validate_request(request: BenchmarkRequest) -> Result<(), ApiError> {\n", execute_start)
    if execute_start < 0 or validate_start < 0:
        raise RuntimeError(f"{rel}: execute_benchmark boundaries not found")
    execute_block = '''fn execute_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, ApiError> {
    validate_request(request)?;

    let config = RunnerConfig {
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
        ..RunnerConfig::default()
    };

    let runner = Runner::new();

    match request.operation {
        BenchmarkOperation::AddArray => match request.precision {
            BenchmarkPrecision::F32 => {
                let lhs = create_lhs_f32(request.input_length);
                let rhs = create_rhs_f32(request.input_length);

                runner
                    .run_add_f32(&lhs, &rhs, &config)
                    .map(Into::into)
                    .map_err(|error| ApiError::internal("runner_failed", error.to_string()))
            }
            BenchmarkPrecision::F64 => {
                let lhs = create_lhs_f64(request.input_length);
                let rhs = create_rhs_f64(request.input_length);

                runner
                    .run_add_f64(&lhs, &rhs, &config)
                    .map(Into::into)
                    .map_err(|error| ApiError::internal("runner_failed", error.to_string()))
            }
        },
        BenchmarkOperation::SumF64 => {
            if request.precision != BenchmarkPrecision::F64 {
                return Err(ApiError::bad_request(
                    "invalid_benchmark_precision",
                    "sum-f64 benchmark requires f64 precision",
                ));
            }

            let input = create_lhs_f64(request.input_length);
            runner
                .run_sum_f64(&input, &config)
                .map(Into::into)
                .map_err(|error| ApiError::internal("runner_failed", error.to_string()))
        }
    }
}

'''
    text = text[:execute_start] + execute_block + text[validate_start:]

    # Operation enum + request/report.
    precision_enum = '''#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum BenchmarkPrecision {
    F32,
    F64,
}
'''
    operation_precision = '''#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum BenchmarkOperation {
    AddArray,
    SumF64,
}

impl Default for BenchmarkOperation {
    fn default() -> Self {
        Self::AddArray
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum BenchmarkPrecision {
    F32,
    F64,
}
'''
    text = replace_once(
        text,
        precision_enum,
        operation_precision,
        f"{rel}: operation enum",
    )
    text = replace_once(
        text,
        "struct BenchmarkRequest {\n"
        "    precision: BenchmarkPrecision,\n",
        "struct BenchmarkRequest {\n"
        "    #[serde(default)]\n"
        "    operation: BenchmarkOperation,\n"
        "    precision: BenchmarkPrecision,\n",
        f"{rel}: request operation",
    )
    text = replace_once(
        text,
        "struct BenchmarkReportDto {\n"
        "    precision: BenchmarkPrecision,\n",
        "struct BenchmarkReportDto {\n"
        "    operation: BenchmarkOperation,\n"
        "    precision: BenchmarkPrecision,\n",
        f"{rel}: report operation",
    )

    # Add operation to Add conversions.
    text = replace_once(
        text,
        "        Self {\n"
        "            precision: BenchmarkPrecision::F32,\n",
        "        Self {\n"
        "            operation: BenchmarkOperation::AddArray,\n"
        "            precision: BenchmarkPrecision::F32,\n",
        f"{rel}: AddF32 operation",
    )
    text = replace_once(
        text,
        "        Self {\n"
        "            precision: BenchmarkPrecision::F64,\n"
        "            input_length: report.input_length,\n",
        "        Self {\n"
        "            operation: BenchmarkOperation::AddArray,\n"
        "            precision: BenchmarkPrecision::F64,\n"
        "            input_length: report.input_length,\n",
        f"{rel}: AddF64 operation",
    )

    backend_impl = text.find("impl From<BackendRunResult> for BackendResultDto {")
    if backend_impl < 0:
        raise RuntimeError(f"{rel}: BackendRunResult conversion not found")
    sum_conversion = '''impl From<SumF64Report> for BenchmarkReportDto {
    fn from(report: SumF64Report) -> Self {
        Self {
            operation: BenchmarkOperation::SumF64,
            precision: BenchmarkPrecision::F64,
            input_length: report.input_length,
            reference_backend: report.reference_backend.display_name().to_owned(),
            warmup_iterations: report.warmup_iterations,
            measured_iterations: report.measured_iterations,
            absolute_tolerance: report.absolute_tolerance,
            results: report
                .results
                .into_iter()
                .map(BackendResultDto::from)
                .collect(),
        }
    }
}

'''
    text = text[:backend_impl] + sum_conversion + text[backend_impl:]

    # Existing test request gets AddArray + add a Sum test before module close.
    text = replace_once(
        text,
        "        let report = execute_benchmark(BenchmarkRequest {\n"
        "            precision: BenchmarkPrecision::F64,\n",
        "        let report = execute_benchmark(BenchmarkRequest {\n"
        "            operation: BenchmarkOperation::AddArray,\n"
        "            precision: BenchmarkPrecision::F64,\n",
        f"{rel}: existing test operation",
    )

    final_module_close = text.rfind("}\n")
    if final_module_close < 0:
        raise RuntimeError(f"{rel}: test module close not found")
    sum_test = '''
    #[test]
    fn sum_f64_benchmark_uses_all_available_backends() {
        let report = execute_benchmark(BenchmarkRequest {
            operation: BenchmarkOperation::SumF64,
            precision: BenchmarkPrecision::F64,
            input_length: 17,
            warmup_iterations: 1,
            measured_iterations: 2,
        })
        .expect("sum f64 benchmark must succeed");

        assert_eq!(report.operation, BenchmarkOperation::SumF64);
        assert_eq!(report.precision, BenchmarkPrecision::F64);
        assert_eq!(report.input_length, 17);
        assert!(report.results.iter().all(|result| {
            result.status == "unavailable" || result.matches_reference == Some(true)
        }));
    }
'''
    text = text[:final_module_close] + sum_test + text[final_module_close:]
    updated[rel] = text

    # Preflight complete. Write only after all transformations succeeded.
    for rel, new_text in updated.items():
        preserve_write(rel, new_text)

    print("Applied SumF64 multi-operation benchmark UI/API integration.")
    print(f"Changed {len(updated)} tracked files.")
    print(r"Next: cargo fmt --all, npm run build in apps\whitebase-app, then .\scripts\ops.bat check")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print(f"ERROR: {exc}")
        print("No tracked files were intentionally written unless every transformation succeeded.")
        raise SystemExit(1)

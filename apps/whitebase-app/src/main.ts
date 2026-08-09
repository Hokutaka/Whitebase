import { invoke } from "@tauri-apps/api/core";

import {
  getApiBaseUrl,
  initializeComputeClient,
  type ExecutionRoute,
} from "./compute-client";

import {
  observe_add_scalar_f64 as observeAddScalarF64Wasm,
  run_benchmark as runBenchmarkWasm,
} from "./wasm/whitebase_wasm";

import "./styles.css";

interface ScalarF64Request {
  lhs: string;
  rhs: string;
}

interface F64Value {
  value: number;
  decimal: string;
  bits: string;
}

interface ScalarF64BackendResult {
  backend: string;
  result: F64Value;
  matchesReferenceBits: boolean;
}

interface ScalarF64Observation {
  lhsInput: string;
  rhsInput: string;
  lhs: F64Value;
  rhs: F64Value;
  decimalReference: string;
  reference: F64Value;
  results: ScalarF64BackendResult[];
  allBackendsMatch: boolean;
}

type BenchmarkOperation = "add-array" | "sum-f64";
type BenchmarkPrecision = "f32" | "f64";

interface BenchmarkRequest {
  operation: BenchmarkOperation;
  precision: BenchmarkPrecision;
  inputLength: number;
  warmupIterations: number;
  measuredIterations: number;
}

interface BenchmarkReport {
  operation: BenchmarkOperation;
  precision: BenchmarkPrecision;
  inputLength: number;
  referenceBackend: string;
  warmupIterations: number;
  measuredIterations: number;
  absoluteTolerance: number;
  results: BackendResult[];
}

interface BackendResult {
  backend: string;
  status: "completed" | "unavailable" | "failed";

  timingStatus: "measured" | "too-fast-to-measure" | null | undefined;

  iterations: number | null | undefined;
  totalNanoseconds: number | null | undefined;
  minimumNanoseconds: number | null | undefined;
  maximumNanoseconds: number | null | undefined;
  meanNanoseconds: number | null | undefined;

  matchesReference: boolean | null | undefined;
  mismatchCount: number | null | undefined;
  maximumAbsoluteError: number | null | undefined;

  error: string | null | undefined;
}

interface ApiError {
  code: string;
  message: string;
}

function requireElement<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);

  if (!element) {
    throw new Error(`required element was not found: ${selector}`);
  }

  return element;
}

const API_BASE_URL = getApiBaseUrl();

const executionRoute = initializeComputeClient();

void executionRoute.then(
  (route: ExecutionRoute) => {
    console.log(`[Whitebase] Execution route: ${route}`);
  },
  (error: unknown) => {
    console.error(
      `[Whitebase] Execution route initialization failed: ${errorMessage(error)}`,
    );
  },
);

const app = requireElement<HTMLDivElement>("#app");
app.innerHTML = `
  <main class="shell">
    <header class="hero">
      <div>
        <p class="eyebrow">WHITEBASE COMPUTE LAB</p>
        <h1>Backend Observation</h1>
        <p class="subtitle">
          Rust、C++、Assemblyの演算結果を、値とIEEE 754ビット表現まで横断して観測します。
        </p>
      </div>

      <div class="status-panel">
        <span class="status-dot"></span>
        <span id="application-status">Ready</span>
      </div>
    </header>

    <section class="observation-panel">
      <div class="section-heading">
        <div>
          <p class="eyebrow">F64 SCALAR</p>
          <h2>Addition observation</h2>
        </div>

        <p id="observation-error" class="error-message"></p>
      </div>

      <form id="scalar-f64-form" class="observation-form">
        <label>
          <span>Left-hand side</span>
          <input id="scalar-lhs" type="text" inputmode="decimal" value="0.1" required />
        </label>

        <label>
          <span>Right-hand side</span>
          <input id="scalar-rhs" type="text" inputmode="decimal" value="0.2" required />
        </label>

        <button id="observe-button" type="submit">OBSERVE</button>
      </form>

      <div id="observation-summary" class="observation-summary" hidden>
        <div>
          <span>Expression</span>
          <strong id="observation-expression" class="monospace">-</strong>
        </div>

        <div>
          <span>Exact decimal reference</span>
          <strong id="observation-decimal-reference" class="monospace">-</strong>
        </div>

        <div>
          <span>Backend agreement</span>
          <strong id="observation-agreement">-</strong>
        </div>
      </div>

      <div class="table-wrapper observation-table-wrapper">
        <table>
          <thead>
            <tr>
              <th>Backend</th>
              <th>Decimal value</th>
              <th>IEEE 754 bits</th>
              <th>vs decimal reference</th>
            </tr>
          </thead>

          <tbody id="observation-results-body">
            <tr class="empty-row">
              <td colspan="4">Observe 0.1 + 0.2 to display results.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section class="control-panel">
      <div class="section-heading compact-heading">
        <div>
          <p id="benchmark-eyebrow" class="eyebrow">F32 / F64 ARRAY</p>
          <h2>Backend benchmark</h2>
        </div>

        <p id="error-message" class="error-message"></p>
      </div>

      <form id="benchmark-form">
        <fieldset class="operation-control">
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

        <fieldset class="precision-control">
          <legend>Precision</legend>

          <div class="precision-options">
            <label class="precision-option">
              <input type="radio" name="benchmark-precision" value="f32" checked />
              <span>f32</span>
            </label>

            <label class="precision-option">
              <input type="radio" name="benchmark-precision" value="f64" />
              <span>f64</span>
            </label>
          </div>
        </fieldset>

        <label>
          <span>Input length</span>
          <input
            id="input-length"
            type="number"
            min="1"
            max="10000000"
            value="1000000"
            required
          />
        </label>

        <label>
          <span>Warmup</span>
          <input
            id="warmup-iterations"
            type="number"
            min="0"
            max="10000"
            value="3"
            required
          />
        </label>

        <label>
          <span>Iterations</span>
          <input
            id="measured-iterations"
            type="number"
            min="1"
            max="10000"
            value="10"
            required
          />
        </label>

        <button id="run-button" type="submit">RUN BENCHMARK</button>
      </form>
    </section>

    <section class="summary" id="summary" hidden>
      <div class="metric">
        <span>Operation</span>
        <strong id="summary-operation">-</strong>
      </div>

      <div class="metric">
        <span>Precision</span>
        <strong id="summary-precision">-</strong>
      </div>

      <div class="metric">
        <span>Elements</span>
        <strong id="summary-elements">-</strong>
      </div>

      <div class="metric">
        <span>Reference</span>
        <strong id="summary-reference">-</strong>
      </div>

      <div class="metric">
        <span>Iterations</span>
        <strong id="summary-iterations">-</strong>
      </div>

      <div class="metric">
        <span>Fastest</span>
        <strong id="summary-fastest">-</strong>
      </div>
    </section>

    <section class="results-panel">
      <div class="section-heading">
        <div>
          <p class="eyebrow">BENCHMARK RESULTS</p>
          <h2>Backend comparison</h2>
        </div>
      </div>

      <div class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th>Backend</th>
              <th>Status</th>
              <th>Mean</th>
              <th>Minimum</th>
              <th>Maximum</th>
              <th>Speedup</th>
              <th>Result</th>
            </tr>
          </thead>

          <tbody id="results-body">
            <tr class="empty-row">
              <td colspan="7">Run the benchmark to display results.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </main>
`;

const observationForm = requireElement<HTMLFormElement>("#scalar-f64-form");
const observeButton = requireElement<HTMLButtonElement>("#observe-button");
const observationError = requireElement<HTMLParagraphElement>("#observation-error");
const observationSummary = requireElement<HTMLElement>("#observation-summary");
const observationResultsBody = requireElement<HTMLTableSectionElement>(
  "#observation-results-body",
);

const benchmarkForm = requireElement<HTMLFormElement>("#benchmark-form");
const runButton = requireElement<HTMLButtonElement>("#run-button");
const statusElement = requireElement<HTMLSpanElement>("#application-status");
const errorElement = requireElement<HTMLParagraphElement>("#error-message");
const resultsBody = requireElement<HTMLTableSectionElement>("#results-body");
const summary = requireElement<HTMLElement>("#summary");
const f32PrecisionInput = requireElement<HTMLInputElement>(
  'input[name="benchmark-precision"][value="f32"]',
);
const f64PrecisionInput = requireElement<HTMLInputElement>(
  'input[name="benchmark-precision"][value="f64"]',
);

document
  .querySelectorAll<HTMLInputElement>('input[name="benchmark-operation"]')
  .forEach((input) => input.addEventListener("change", syncBenchmarkControls));

syncBenchmarkControls();

observationForm.addEventListener("submit", async (event) => {
  event.preventDefault();

  const request: ScalarF64Request = {
    lhs: readDecimalText("scalar-lhs"),
    rhs: readDecimalText("scalar-rhs"),
  };

  setBusy(true, "observation");
  observationError.textContent = "";

  try {
    const report = await executeScalarF64Observation(request);

    renderScalarF64Observation(report);
    statusElement.textContent = "Completed";
  } catch (error) {
    observationError.textContent = errorMessage(error);
    statusElement.textContent = "Failed";
  } finally {
    setBusy(false, "observation");
  }
});

benchmarkForm.addEventListener("submit", async (event) => {
  event.preventDefault();

  const request: BenchmarkRequest = {
    operation: readBenchmarkOperation(),
    precision: readBenchmarkPrecision(),
    inputLength: readNumber("input-length"),
    warmupIterations: readNumber("warmup-iterations"),
    measuredIterations: readNumber("measured-iterations"),
  };

  setBusy(true, "benchmark");
  errorElement.textContent = "";

  try {
    const report = await executeBenchmark(request);

    console.log(report.results);

    renderBenchmarkReport(report);
    statusElement.textContent = "Completed";
  } catch (error) {
    errorElement.textContent = errorMessage(error);
    statusElement.textContent = "Failed";
  } finally {
    setBusy(false, "benchmark");
  }
});

function renderScalarF64Observation(report: ScalarF64Observation): void {
  const resultRows = report.results
    .map((backendResult) => {
      const matchesReference = backendResult.matchesReferenceBits;

      return `
        <tr>
          <td class="backend-name">${escapeHtml(backendResult.backend)}</td>
          <td class="monospace">${escapeHtml(backendResult.result.decimal)}</td>
          <td class="monospace bits">${escapeHtml(backendResult.result.bits)}</td>
          <td>
            <span class="badge ${matchesReference ? "badge-ok" : "badge-warn"}">
              ${matchesReference ? "BIT MATCH" : "DIFFERENT"}
            </span>
          </td>
        </tr>
      `;
    })
    .join("");

  observationResultsBody.innerHTML = `
    ${resultRows}
    <tr class="expected-row">
      <td class="backend-name">Decimal reference</td>
      <td class="monospace">${escapeHtml(report.reference.decimal)}</td>
      <td class="monospace bits">${escapeHtml(report.reference.bits)}</td>
      <td><span class="badge badge-muted">REFERENCE</span></td>
    </tr>
  `;

  setText(
    "observation-expression",
    `${report.lhsInput} + ${report.rhsInput}`,
  );

  const agreement = report.allBackendsMatch
    ? "All backend bits match"
    : "Backend mismatch detected";

  setText("observation-decimal-reference", report.decimalReference);
  setText("observation-agreement", agreement);
  observationSummary.hidden = false;
}

function renderBenchmarkReport(report: BenchmarkReport): void {
  const completed = report.results.filter(
    (
      result,
    ): result is BackendResult & {
      meanNanoseconds: number;
    } =>
      result.status === "completed" &&
      result.timingStatus === "measured" &&
      typeof result.meanNanoseconds === "number",
  );

  const baseline =
    completed.find((result) => result.backend === "Rust Scalar")
      ?.meanNanoseconds ??
    completed[0]?.meanNanoseconds ??
    null;

  const fastest = completed.reduce<
    (BackendResult & { meanNanoseconds: number }) | null
  >((current, result) => {
    if (!current) {
      return result;
    }

    return result.meanNanoseconds < current.meanNanoseconds ? result : current;
  }, null);

  resultsBody.innerHTML = report.results
    .map((result) => {
      if (result.status === "unavailable") {
        return `
          <tr>
            <td class="backend-name">${escapeHtml(result.backend)}</td>
            <td><span class="badge badge-muted">UNAVAILABLE</span></td>
            <td colspan="5">—</td>
          </tr>
        `;
      }

      if (result.status === "failed") {
        return `
          <tr>
            <td class="backend-name">${escapeHtml(result.backend)}</td>
            <td><span class="badge badge-error">FAILED</span></td>
            <td colspan="5" class="failure">
              ${escapeHtml(result.error ?? "Unknown error")}
            </td>
          </tr>
        `;
      }

      if (result.timingStatus === "too-fast-to-measure") {
        const matches = result.matchesReference === true;

        return `
          <tr>
            <td class="backend-name">${escapeHtml(result.backend)}</td>
            <td><span class="badge badge-ok">COMPLETED</span></td>
            <td colspan="3">
              <span class="badge badge-warn">TOO FAST TO MEASURE</span>
            </td>
            <td>—</td>
            <td>
              <span class="badge ${matches ? "badge-ok" : "badge-error"}">
                ${matches ? "MATCH" : "MISMATCH"}
              </span>
            </td>
          </tr>
        `;
      }

      const speedup =
        baseline !== null && typeof result.meanNanoseconds === "number"
          ? baseline / result.meanNanoseconds
          : null;

      const matches = result.matchesReference === true;

      return `
        <tr>
          <td class="backend-name">${escapeHtml(result.backend)}</td>
          <td><span class="badge badge-ok">COMPLETED</span></td>
          <td>${formatDuration(result.meanNanoseconds)}</td>
          <td>${formatDuration(result.minimumNanoseconds)}</td>
          <td>${formatDuration(result.maximumNanoseconds)}</td>
          <td class="speedup">
            ${speedup === null ? "—" : `${speedup.toFixed(2)}x`}
          </td>
          <td>
            <span class="badge ${matches ? "badge-ok" : "badge-error"}">
              ${matches ? "MATCH" : "MISMATCH"}
            </span>
          </td>
        </tr>
      `;
    })
    .join("");

  setText(
    "summary-operation",
    report.operation === "sum-f64" ? "Sum f64" : "Add array",
  );
  setText("summary-precision", report.precision.toUpperCase());
  setText("summary-elements", report.inputLength.toLocaleString());
  setText("summary-reference", report.referenceBackend);
  setText("summary-iterations", report.measuredIterations.toLocaleString());
  setText(
    "summary-fastest",
    fastest
      ? `${fastest.backend} / ${formatDuration(fastest.meanNanoseconds)}`
      : "—",
  );

  summary.hidden = false;
}

function setBusy(
  running: boolean,
  activeTask: "observation" | "benchmark",
): void {
  observeButton.disabled = running;
  runButton.disabled = running;

  observeButton.textContent =
    running && activeTask === "observation" ? "OBSERVING..." : "OBSERVE";

  runButton.textContent =
    running && activeTask === "benchmark" ? "RUNNING..." : "RUN BENCHMARK";

  if (running) {
    statusElement.textContent = "Running";
  }
}

function syncBenchmarkControls(): void {
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

function readBenchmarkPrecision(): BenchmarkPrecision {
  const input = requireElement<HTMLInputElement>(
    'input[name="benchmark-precision"]:checked',
  );

  if (input.value !== "f32" && input.value !== "f64") {
    throw new Error(`unsupported benchmark precision: ${input.value}`);
  }

  return input.value;
}

function readNumber(id: string): number {
  const input = requireElement<HTMLInputElement>(`#${id}`);
  return Number(input.value);
}

function readDecimalText(id: string): string {
  const input = requireElement<HTMLInputElement>(`#${id}`);
  const value = input.value.trim();

  if (value.length === 0) {
    throw new Error(`${id} must not be empty`);
  }

  return value;
}

function setText(id: string, value: string): void {
  const element = document.getElementById(id);

  if (!element) {
    throw new Error(`element was not found: ${id}`);
  }

  element.textContent = value;
}

function formatDuration(
  nanoseconds: number | null | undefined,
): string {
  if (nanoseconds == null) {
    return "—";
  }

  if (nanoseconds >= 1_000_000) {
    return `${(nanoseconds / 1_000_000).toFixed(3)} ms`;
  }

  if (nanoseconds >= 1_000) {
    return `${(nanoseconds / 1_000).toFixed(3)} μs`;
  }

  return `${nanoseconds.toFixed(1)} ns`;
}

function escapeHtml(value: string): string {
  const element = document.createElement("div");
  element.textContent = value;
  return element.innerHTML;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

async function executeScalarF64Observation(
  request: ScalarF64Request,
): Promise<ScalarF64Observation> {
  const route = await executionRoute;

  if (route === "tauri") {
    return invoke<ScalarF64Observation>("observe_add_scalar_f64", { request });
  }

  if (route === "wasm") {
    return observeAddScalarF64Wasm(
      request.lhs,
      request.rhs,
    ) as ScalarF64Observation;
  }

  let response: Response;

  try {
    response = await fetch(`${API_BASE_URL}/api/observations/add-scalar-f64`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
    });
  } catch {
    throw new Error("Whitebase Serverとの接続が失われました。");
  }

  if (!response.ok) {
    const error = await readApiError(response);

    throw new Error(
      error?.message ??
        `scalar f64 observation server returned HTTP ${response.status}`,
    );
  }

  return (await response.json()) as ScalarF64Observation;
}

async function executeBenchmark(
  request: BenchmarkRequest,
): Promise<BenchmarkReport> {
  const route = await executionRoute;

  if (route === "tauri") {
    return invoke<BenchmarkReport>("run_benchmark", { request });
  }

  if (route === "wasm") {
    return runBenchmarkWasm(
      request.operation,
      request.precision,
      request.inputLength,
      request.warmupIterations,
      request.measuredIterations,
    ) as BenchmarkReport;
  }

  let response: Response;

  try {
    response = await fetch(`${API_BASE_URL}/api/benchmarks/run`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
    });
  } catch {
    throw new Error("Whitebase Serverとの接続が失われました。");
  }

  if (!response.ok) {
    const error = await readApiError(response);

    throw new Error(
      error?.message ?? `benchmark server returned HTTP ${response.status}`,
    );
  }

  return (await response.json()) as BenchmarkReport;
}

async function readApiError(response: Response): Promise<ApiError | null> {
  try {
    return (await response.json()) as ApiError;
  } catch {
    return null;
  }
}

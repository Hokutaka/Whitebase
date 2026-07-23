import {
  invoke,
  isTauri,
} from "@tauri-apps/api/core";

import "./styles.css";

interface BenchmarkRequest {
  inputLength: number;
  warmupIterations: number;
  measuredIterations: number;
}

interface BenchmarkReport {
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

  iterations: number | null;
  totalNanoseconds: number | null;
  minimumNanoseconds: number | null;
  maximumNanoseconds: number | null;
  meanNanoseconds: number | null;

  matchesReference: boolean | null;
  mismatchCount: number | null;
  maximumAbsoluteError: number | null;

  error: string | null;
}

interface ApiError {
  code: string;
  message: string;
}

function requireElement<T extends Element>(
  selector: string,
): T {
  const element = document.querySelector<T>(selector);

  if (!element) {
    throw new Error(
      `required element was not found: ${selector}`,
    );
  }

  return element;
}

const API_BASE_URL = "http://127.0.0.1:1430";
const runningInTauri = isTauri();

const app = requireElement<HTMLDivElement>("#app");
app.innerHTML = `
  <main class="shell">
    <header class="hero">
      <div>
        <p class="eyebrow">WHITEBASE COMPUTE LAB</p>
        <h1>Backend Benchmark</h1>
        <p class="subtitle">
          Rust、C++、AssemblyのScalar / SIMD実装を計測・比較します。
        </p>
      </div>

      <div class="status-panel">
        <span class="status-dot"></span>
        <span id="application-status">Ready</span>
      </div>
    </header>

    <section class="control-panel">
      <form id="benchmark-form">
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

        <button id="run-button" type="submit">
          RUN BENCHMARK
        </button>
      </form>
    </section>

    <section class="summary" id="summary" hidden>
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
          <p class="eyebrow">RESULTS</p>
          <h2>Backend comparison</h2>
        </div>

        <p id="error-message" class="error-message"></p>
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
              <td colspan="7">
                Run the benchmark to display results.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </main>
`;

const form =
  requireElement<HTMLFormElement>("#benchmark-form");

const runButton =
  requireElement<HTMLButtonElement>("#run-button");

const statusElement =
  requireElement<HTMLSpanElement>("#application-status");

const errorElement =
  requireElement<HTMLParagraphElement>("#error-message");

const resultsBody =
  requireElement<HTMLTableSectionElement>("#results-body");

const summary =
  requireElement<HTMLElement>("#summary");

form.addEventListener("submit", async (event) => {
  event.preventDefault();

  const request: BenchmarkRequest = {
    inputLength: readNumber("input-length"),
    warmupIterations: readNumber("warmup-iterations"),
    measuredIterations: readNumber(
      "measured-iterations",
    ),
  };

  setRunning(true);
  errorElement.textContent = "";

  try {
    const report = await executeBenchmark(request);

    renderReport(report);
    statusElement.textContent = "Completed";
  } catch (error) {
    const message =
      error instanceof Error
        ? error.message
        : String(error);

    errorElement.textContent = message;
    statusElement.textContent = "Failed";
  } finally {
    setRunning(false);
  }
});

function renderReport(report: BenchmarkReport): void {
  const completed = report.results.filter(
    (
      result,
    ): result is BackendResult & {
      meanNanoseconds: number;
    } =>
      result.status === "completed" &&
      result.meanNanoseconds !== null,
  );

  const baseline =
    completed.find(
      (result) => result.backend === "Rust Scalar",
    )?.meanNanoseconds ??
    completed[0]?.meanNanoseconds ??
    null;

  const fastest = completed.reduce<
    (BackendResult & { meanNanoseconds: number }) | null
  >((current, result) => {
    if (!current) {
      return result;
    }

    return result.meanNanoseconds <
      current.meanNanoseconds
      ? result
      : current;
  }, null);

  resultsBody.innerHTML = report.results
    .map((result) => {
      if (result.status === "unavailable") {
        return `
          <tr>
            <td class="backend-name">${result.backend}</td>
            <td>
              <span class="badge badge-muted">
                UNAVAILABLE
              </span>
            </td>
            <td colspan="5">—</td>
          </tr>
        `;
      }

      if (result.status === "failed") {
        return `
          <tr>
            <td class="backend-name">${result.backend}</td>
            <td>
              <span class="badge badge-error">
                FAILED
              </span>
            </td>
            <td colspan="5" class="failure">
              ${escapeHtml(result.error ?? "Unknown error")}
            </td>
          </tr>
        `;
      }

      const speedup =
        baseline !== null &&
        result.meanNanoseconds !== null
          ? baseline / result.meanNanoseconds
          : null;

      const matches =
        result.matchesReference === true;

      return `
        <tr>
          <td class="backend-name">${result.backend}</td>
          <td>
            <span class="badge badge-ok">COMPLETED</span>
          </td>
          <td>${formatDuration(result.meanNanoseconds)}</td>
          <td>${formatDuration(result.minimumNanoseconds)}</td>
          <td>${formatDuration(result.maximumNanoseconds)}</td>
          <td class="speedup">
            ${speedup === null ? "—" : `${speedup.toFixed(2)}x`}
          </td>
          <td>
            <span class="badge ${
              matches ? "badge-ok" : "badge-error"
            }">
              ${matches ? "MATCH" : "MISMATCH"}
            </span>
          </td>
        </tr>
      `;
    })
    .join("");

  setText(
    "summary-elements",
    report.inputLength.toLocaleString(),
  );

  setText(
    "summary-reference",
    report.referenceBackend,
  );

  setText(
    "summary-iterations",
    report.measuredIterations.toLocaleString(),
  );

  setText(
    "summary-fastest",
    fastest
      ? `${fastest.backend} / ${formatDuration(
          fastest.meanNanoseconds,
        )}`
      : "—",
  );

  summary.hidden = false;
}

function setRunning(running: boolean): void {
  runButton.disabled = running;
  runButton.textContent = running
    ? "RUNNING..."
    : "RUN BENCHMARK";

  statusElement.textContent = running
    ? "Running"
    : statusElement.textContent;
}

function readNumber(id: string): number {
  const input =
    document.querySelector<HTMLInputElement>(`#${id}`);

  if (!input) {
    throw new Error(`input was not found: ${id}`);
  }

  return Number(input.value);
}

function setText(
  id: string,
  value: string,
): void {
  const element = document.getElementById(id);

  if (!element) {
    throw new Error(`element was not found: ${id}`);
  }

  element.textContent = value;
}

function formatDuration(
  nanoseconds: number | null,
): string {
  if (nanoseconds === null) {
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

async function executeBenchmark(
  request: BenchmarkRequest,
): Promise<BenchmarkReport> {
  if (runningInTauri) {
    return invoke<BenchmarkReport>(
      "run_add_f32_benchmark",
      { request },
    );
  }

  let response: Response;

  try {
    response = await fetch(
      `${API_BASE_URL}/api/benchmarks/add-f32`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(request),
      },
    );
  } catch {
    throw new Error(
      "Whitebase Serverに接続できません。"
      + " cargo run -p whitebase-server"
      + " を起動してください。",
    );
  }

  if (!response.ok) {
    const error = await readApiError(response);

    throw new Error(
      error?.message ??
        `benchmark server returned HTTP ${
          response.status
        }`,
    );
  }

  return await response.json() as BenchmarkReport;
}

async function readApiError(
  response: Response,
): Promise<ApiError | null> {
  try {
    return await response.json() as ApiError;
  } catch {
    return null;
  }
}


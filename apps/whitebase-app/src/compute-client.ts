import { isTauri } from "@tauri-apps/api/core";
import initWasm from "./wasm/whitebase_wasm";

export type ExecutionRoute = "tauri" | "server" | "wasm";

const API_BASE_URL = "http://127.0.0.1:1430";
const SERVER_PROBE_TIMEOUT_MS = 1_000;

let routePromise: Promise<ExecutionRoute> | null = null;

export class ComputeClientInitializationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ComputeClientInitializationError";
  }
}

async function detectExecutionRoute(): Promise<ExecutionRoute> {
  if (isTauri()) {
    return "tauri";
  }

  try {
    const response = await fetch(`${API_BASE_URL}/api/health`, {
      signal: AbortSignal.timeout(SERVER_PROBE_TIMEOUT_MS),
    });

    if (response.ok) {
      return "server";
    }
  } catch {
    // Serverが利用できない場合はWASMへフォールバックします。
  }

  try {
    await initWasm();
  } catch {
    throw new ComputeClientInitializationError(
      "Whitebase WebAssemblyの初期化に失敗しました。",
    );
  }

  return "wasm";
}

export function initializeComputeClient(): Promise<ExecutionRoute> {
  routePromise ??= detectExecutionRoute();

  return routePromise;
}

export function getApiBaseUrl(): string {
  return API_BASE_URL;
}

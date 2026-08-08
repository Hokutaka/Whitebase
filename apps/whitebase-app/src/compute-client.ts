import { isTauri } from "@tauri-apps/api/core";
import initWasm from "./wasm/whitebase_wasm";

export type ExecutionRoute = "tauri" | "server" | "wasm";

const API_BASE_URL = "http://127.0.0.1:1430";

let routePromise: Promise<ExecutionRoute> | null = null;

async function detectExecutionRoute(): Promise<ExecutionRoute> {
  if (isTauri()) {
    return "tauri";
  }

  try {
    const response = await fetch(`${API_BASE_URL}/api/health`);

    if (response.ok) {
      return "server";
    }
  } catch {
    // Serverが起動していない場合はWASMへフォールバックします。
  }

  await initWasm();

  return "wasm";
}

export function initializeComputeClient(): Promise<ExecutionRoute> {
  routePromise ??= detectExecutionRoute();

  return routePromise;
}

export function getApiBaseUrl(): string {
  return API_BASE_URL;
}

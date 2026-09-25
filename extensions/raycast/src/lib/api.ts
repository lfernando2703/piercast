import { getPreferenceValues } from "@raycast/api";
import type { ExposeMode, ExposeResponse, HealthResponse, PiercastApp, StartMode } from "./types";

export interface Preferences {
  apiBaseUrl: string;
  token: string;
}

export class PiercastApiError extends Error {
  readonly status: number;
  readonly body: string;

  constructor(status: number, body: string, path: string) {
    super(`Piercast API ${status} on ${path}: ${body || "empty body"}`);
    this.name = "PiercastApiError";
    this.status = status;
    this.body = body;
  }
}

function preferences(): Preferences {
  const prefs = getPreferenceValues<Preferences>();
  return {
    apiBaseUrl: (prefs.apiBaseUrl || "http://127.0.0.1:47923").replace(/\/$/, ""),
    token: prefs.token,
  };
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const { apiBaseUrl, token } = preferences();
  const headers: Record<string, string> = {
    Accept: "application/json",
    Authorization: `Bearer ${token}`,
  };
  let payload: string | undefined;
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
    payload = JSON.stringify(body);
  }

  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl}${path}`, { method, headers, body: payload });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Cannot reach Piercast daemon at ${apiBaseUrl}: ${message}`);
  }

  const text = await response.text();
  if (!response.ok) {
    throw new PiercastApiError(response.status, text, path);
  }
  if (!text) {
    return undefined as T;
  }
  try {
    return JSON.parse(text) as T;
  } catch {
    return text as unknown as T;
  }
}

function normalizeApps(payload: unknown): PiercastApp[] {
  if (Array.isArray(payload)) {
    return payload as PiercastApp[];
  }
  if (payload && typeof payload === "object") {
    const record = payload as Record<string, unknown>;
    if (Array.isArray(record.apps)) {
      return record.apps as PiercastApp[];
    }
    if (Array.isArray(record.items)) {
      return record.items as PiercastApp[];
    }
  }
  return [];
}

export async function getHealth(): Promise<HealthResponse> {
  return request<HealthResponse>("GET", "/v1/health");
}

export async function listApps(): Promise<PiercastApp[]> {
  const payload = await request<unknown>("GET", "/v1/apps");
  return normalizeApps(payload);
}

export async function startApp(id: string, mode: StartMode = "development"): Promise<void> {
  await request("POST", `/v1/apps/${encodeURIComponent(id)}/start`, { mode });
}

export async function stopApp(id: string): Promise<void> {
  await request("POST", `/v1/apps/${encodeURIComponent(id)}/stop`);
}

export async function restartApp(id: string): Promise<void> {
  await request("POST", `/v1/apps/${encodeURIComponent(id)}/restart`);
}

export async function killApp(id: string): Promise<void> {
  await request("POST", `/v1/apps/${encodeURIComponent(id)}/kill`);
}

/** Open auto-starts the app (deps + last_mode/development) then opens the URL. */
export async function openApp(id: string): Promise<void> {
  await request("POST", `/v1/apps/${encodeURIComponent(id)}/open`);
}

export async function exposeApp(id: string, mode: ExposeMode): Promise<ExposeResponse> {
  return request<ExposeResponse>("POST", `/v1/apps/${encodeURIComponent(id)}/expose`, { mode });
}

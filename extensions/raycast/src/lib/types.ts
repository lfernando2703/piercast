export type StartMode = "development" | "production";
export type ExposeMode = "serve" | "funnel" | "off";

export type AppStatus = "stopped" | "starting" | "running" | "unhealthy" | "stopping" | string;

export interface PiercastApp {
  id: string;
  name: string;
  description?: string;
  root?: string;
  tags?: string[];
  status?: AppStatus;
  last_mode?: StartMode;
  ports?: {
    primary?: number;
    extra?: number[];
  };
  open_url?: string;
  expose?: {
    default?: ExposeMode;
    path?: string;
    url?: string;
    mode?: ExposeMode;
  };
  health?: {
    kind?: string;
    target?: string;
  };
}

export interface HealthResponse {
  ok?: boolean;
  status?: string;
  version?: string;
  [key: string]: unknown;
}

export interface ExposeResponse {
  mode?: ExposeMode;
  url?: string;
  status?: string;
  message?: string;
  [key: string]: unknown;
}

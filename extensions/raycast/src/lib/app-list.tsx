import { Action, ActionPanel, Color, Icon, List, Toast, showToast } from "@raycast/api";
import { useCachedPromise } from "@raycast/utils";
import { exposeApp, killApp, listApps, openApp, restartApp, startApp, stopApp } from "./api";
import type { ExposeMode, PiercastApp } from "./types";

function statusColor(status?: string): Color {
  switch ((status || "").toLowerCase()) {
    case "running":
    case "healthy":
      return Color.Green;
    case "unhealthy":
    case "failed":
      return Color.Red;
    case "starting":
    case "stopping":
      return Color.Orange;
    default:
      return Color.SecondaryText;
  }
}

async function withToast(title: string, action: () => Promise<unknown>, successMessage?: string) {
  const toast = await showToast({ style: Toast.Style.Animated, title });
  try {
    const result = await action();
    toast.style = Toast.Style.Success;
    toast.title = successMessage || title;
    if (result && typeof result === "object" && "url" in result && (result as { url?: string }).url) {
      toast.message = String((result as { url?: string }).url);
    }
  } catch (error) {
    toast.style = Toast.Style.Failure;
    toast.title = "Failed";
    toast.message = error instanceof Error ? error.message : String(error);
  }
}

type Primary = "open" | "start" | "stop" | "restart" | "kill" | "expose";

export function AppActions({ app, onMutate, primary }: { app: PiercastApp; onMutate?: () => void; primary?: Primary }) {
  const refresh = () => onMutate?.();

  const run = (title: string, fn: () => Promise<unknown>, success?: string) => () =>
    withToast(title, fn, success).then(refresh);

  const openAction = (
    <Action
      title="Open App"
      icon={Icon.Globe}
      onAction={run(`Opening ${app.name}`, () => openApp(app.id), `Opened ${app.name}`)}
    />
  );
  const startDevAction = (
    <Action
      title="Start (Development)"
      icon={Icon.Play}
      onAction={run(`Starting ${app.name}`, () => startApp(app.id, "development"), `Started ${app.name}`)}
    />
  );
  const startProdAction = (
    <Action
      title="Start (Production)"
      icon={Icon.Play}
      onAction={run(`Starting ${app.name}`, () => startApp(app.id, "production"), `Started ${app.name}`)}
    />
  );
  const stopAction = (
    <Action
      title="Stop"
      icon={Icon.Stop}
      style={Action.Style.Destructive}
      onAction={run(`Stopping ${app.name}`, () => stopApp(app.id), `Stopped ${app.name}`)}
    />
  );
  const restartAction = (
    <Action
      title="Restart"
      icon={Icon.ArrowClockwise}
      onAction={run(`Restarting ${app.name}`, () => restartApp(app.id), `Restarted ${app.name}`)}
    />
  );
  const killAction = (
    <Action
      title="Kill"
      icon={Icon.XMarkCircle}
      style={Action.Style.Destructive}
      onAction={run(`Killing ${app.name}`, () => killApp(app.id), `Killed ${app.name}`)}
    />
  );
  const exposeAction = (mode: ExposeMode, title: string) => (
    <Action
      title={title}
      icon={Icon.Link}
      onAction={run(`Expose ${app.name} → ${mode}`, () => exposeApp(app.id, mode), `Exposed ${app.name} (${mode})`)}
    />
  );

  let primarySection = (
    <>
      {openAction}
      {startDevAction}
      {startProdAction}
      {stopAction}
      {restartAction}
      {killAction}
      {exposeAction("serve", "Expose: Tailscale Serve")}
      {exposeAction("funnel", "Expose: Tailscale Funnel")}
      {exposeAction("off", "Expose: Off")}
    </>
  );

  if (primary === "open") {
    primarySection = (
      <>
        {openAction}
        {startDevAction}
        {startProdAction}
        {stopAction}
        {restartAction}
        {killAction}
      </>
    );
  } else if (primary === "start") {
    primarySection = (
      <>
        {startDevAction}
        {startProdAction}
        {openAction}
        {stopAction}
        {restartAction}
        {killAction}
      </>
    );
  } else if (primary === "stop") {
    primarySection = (
      <>
        {stopAction}
        {restartAction}
        {killAction}
        {openAction}
        {startDevAction}
      </>
    );
  } else if (primary === "restart") {
    primarySection = (
      <>
        {restartAction}
        {stopAction}
        {killAction}
        {openAction}
        {startDevAction}
      </>
    );
  } else if (primary === "kill") {
    primarySection = (
      <>
        {killAction}
        {stopAction}
        {restartAction}
        {openAction}
      </>
    );
  } else if (primary === "expose") {
    primarySection = (
      <>
        {exposeAction("serve", "Expose: Tailscale Serve")}
        {exposeAction("funnel", "Expose: Tailscale Funnel")}
        {exposeAction("off", "Expose: Off")}
        {openAction}
      </>
    );
  }

  return (
    <ActionPanel>
      <ActionPanel.Section title={app.name}>{primarySection}</ActionPanel.Section>
      <ActionPanel.Section>
        <Action title="Refresh" icon={Icon.ArrowClockwise} onAction={refresh} />
        <Action.CopyToClipboard title="Copy App ID" content={app.id} />
      </ActionPanel.Section>
    </ActionPanel>
  );
}

export function AppList({ primary, emptyTitle }: { primary?: Primary; emptyTitle: string }) {
  const { isLoading, data, error, revalidate } = useCachedPromise(listApps, [], {
    keepPreviousData: true,
  });

  return (
    <List isLoading={isLoading} searchBarPlaceholder="Search Piercast apps…">
      {error ? (
        <List.EmptyView
          icon={Icon.Warning}
          title="Cannot reach Piercast"
          description={error instanceof Error ? error.message : String(error)}
        />
      ) : null}
      {!error && (!data || data.length === 0) ? (
        <List.EmptyView
          icon={Icon.AppWindowGrid2x2}
          title={emptyTitle}
          description="Register an app with piercast upsert or MCP."
        />
      ) : null}
      {(data || []).map((app) => (
        <List.Item
          key={app.id}
          title={app.name}
          subtitle={app.id}
          keywords={[app.id, ...(app.tags || []), app.description || ""]}
          accessories={[
            ...(app.status ? [{ tag: { value: app.status, color: statusColor(app.status) } }] : []),
            ...(app.ports?.primary ? [{ text: `:${app.ports.primary}` }] : []),
          ]}
          actions={<AppActions app={app} primary={primary} onMutate={revalidate} />}
        />
      ))}
    </List>
  );
}

export type { PiercastApp, ExposeMode };

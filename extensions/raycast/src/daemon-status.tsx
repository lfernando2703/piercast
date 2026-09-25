import { Action, ActionPanel, Color, Detail, Icon } from "@raycast/api";
import { useCachedPromise } from "@raycast/utils";
import { getHealth, listApps } from "./lib/api";

export default function DaemonStatusCommand() {
  const health = useCachedPromise(getHealth);
  const apps = useCachedPromise(listApps, [], { execute: !health.isLoading && !health.error });

  const baseOk = !health.error && !health.isLoading;
  const markdown = health.isLoading
    ? "Checking Piercast daemon…"
    : health.error
      ? [
          "# Daemon unreachable",
          "",
          String(health.error instanceof Error ? health.error.message : health.error),
          "",
          "Ensure `piercastd` is running and the API Base URL / token preferences are correct.",
        ].join("\n")
      : [
          "# Daemon online",
          "",
          `- **Health:** \`${JSON.stringify(health.data ?? { ok: true })}\``,
          `- **Registered apps:** ${apps.isLoading ? "…" : apps.error ? "n/a" : String(apps.data?.length ?? 0)}`,
          "",
          "Control plane defaults to `http://127.0.0.1:47923` (override with preference or `PIERCAST_PORT`).",
        ].join("\n");

  return (
    <Detail
      isLoading={health.isLoading || apps.isLoading}
      markdown={markdown}
      metadata={
        <Detail.Metadata>
          <Detail.Metadata.TagList title="Status">
            <Detail.Metadata.TagList.Item
              text={health.isLoading ? "Checking" : baseOk ? "Online" : "Offline"}
              color={health.isLoading ? Color.SecondaryText : baseOk ? Color.Green : Color.Red}
            />
          </Detail.Metadata.TagList>
          <Detail.Metadata.Separator />
          <Detail.Metadata.Label title="Apps" text={apps.data ? String(apps.data.length) : "—"} />
        </Detail.Metadata>
      }
      actions={
        <ActionPanel>
          <Action
            title="Refresh"
            icon={Icon.ArrowClockwise}
            onAction={() => {
              health.revalidate();
              apps.revalidate();
            }}
          />
          <Action.OpenInBrowser title="Piercast Docs" url="https://piercast.io/docs" />
        </ActionPanel>
      }
    />
  );
}

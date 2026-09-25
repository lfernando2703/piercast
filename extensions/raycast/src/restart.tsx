import { AppList } from "./lib/app-list";

export default function RestartCommand() {
  return <AppList primary="restart" emptyTitle="No apps to restart" />;
}

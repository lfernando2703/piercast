import { AppList } from "./lib/app-list";

export default function StartCommand() {
  return <AppList primary="start" emptyTitle="No apps to start" />;
}

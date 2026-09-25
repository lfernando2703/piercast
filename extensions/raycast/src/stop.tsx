import { AppList } from "./lib/app-list";

export default function StopCommand() {
  return <AppList primary="stop" emptyTitle="No apps to stop" />;
}

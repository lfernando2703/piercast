import { AppList } from "./lib/app-list";

export default function ExposeCommand() {
  return <AppList primary="expose" emptyTitle="No apps to expose" />;
}

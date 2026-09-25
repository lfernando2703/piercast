import { AppList } from "./lib/app-list";

export default function OpenAppCommand() {
  return <AppList primary="open" emptyTitle="No apps to open" />;
}

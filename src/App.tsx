import { AppShell } from "./shell/AppShell";
import { features } from "./features/registry";

export default function App() {
  return <AppShell features={features} />;
}

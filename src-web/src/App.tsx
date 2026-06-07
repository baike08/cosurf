import { AppLayout } from "@/components/layout/AppLayout";
import { useTheme } from "@/hooks/useTheme";

export default function App() {
  useTheme();
  return <AppLayout />;
}

import { DesktopHome } from "./pages/DesktopHome";
import { I18nProvider } from "./i18n";

export default function App() {
  return (
    <I18nProvider>
      <DesktopHome />
    </I18nProvider>
  );
}

import { ThemeProvider } from "next-themes";
import MainWindow from "./components/layout/MainWindow";
import CaptureWindow from "./capture/CaptureWindow";
import MenuBar from "./components/MenuBar";

function App() {
  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");

  if (view === "capture") {
    return <CaptureWindow />;
  }
  if (view === "tray") {
    return (
      <ThemeProvider attribute="data-theme" defaultTheme="dark" enableSystem>
        <MenuBar />
      </ThemeProvider>
    );
  }

  return (
    <ThemeProvider attribute="data-theme" defaultTheme="dark" enableSystem>
      <MainWindow />
    </ThemeProvider>
  );
}

export default App;

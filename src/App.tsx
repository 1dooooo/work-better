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
    return <MenuBar />;
  }

  return <MainWindow />;
}

export default App;

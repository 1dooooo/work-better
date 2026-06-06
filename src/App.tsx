import MainWindow from "./components/MainWindow";
import CaptureWindow from "./capture/CaptureWindow";
import MenuBar from "./components/MenuBar";
import "./styles/main-window.css";

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

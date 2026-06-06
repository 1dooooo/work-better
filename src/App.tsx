import MainWindow from "./components/MainWindow";
import CaptureWindow from "./capture/CaptureWindow";
import "./styles/main-window.css";

function App() {
  const params = new URLSearchParams(window.location.search);
  const view = params.get("view");

  if (view === "capture") {
    return <CaptureWindow />;
  }

  return <MainWindow />;
}

export default App;

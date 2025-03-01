import React from "react"
import MainPage from "./pages/MainPage"

const App: React.FC = () => {
  return (
    <div className="container-fluid vh-100" data-bs-theme="dark">
      <MainPage />
    </div>
  );
}

export default App;

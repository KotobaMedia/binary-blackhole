import React from "react"
import MainPage from "./pages/MainPage"

const App: React.FC = () => {
  // useEffect(() => {
  //   fetch(import.meta.env.VITE_API_URL + "/__health")
  //     .then((response) => response.text())
  //     .then((data) => console.log(data))
  // }, []);
  return (
    <div className="container-fluid vh-100">
      <MainPage />
    </div>
  );
}

export default App;

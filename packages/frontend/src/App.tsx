import React from "react";
import { Redirect, Route, Switch } from "wouter";
import ChatMapPage from "./pages/ChatMapPage";

const App: React.FC = () => {
  return (
    <div className="container-fluid vh-100" data-bs-theme="dark">
      <Switch>
        <Route path="/" component={ChatMapPage} />
        <Route path="/chats/:threadId" component={ChatMapPage} />
        <Route>
          <Redirect to="/" />
        </Route>
      </Switch>
    </div>
  );
}

export default App;

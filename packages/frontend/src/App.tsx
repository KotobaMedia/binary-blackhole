import React from "react";
import { Redirect, Route, Switch } from "wouter";
import ChatMapPage from "./pages/ChatMapPage";
import ChatListPage from "./pages/ChatListPage";
import DataNavigatorPage from "./pages/DataNavigatorPage";

const App: React.FC = () => {
  return (
    <Switch>
      <Route path="/" component={ChatMapPage} />
      <Route path="/chats" component={ChatListPage} />
      <Route path="/chats/:threadId" component={ChatMapPage} />
      <Route path="/data" component={DataNavigatorPage} />
      <Route>
        <Redirect to="/" />
      </Route>
    </Switch>
  );
};

export default App;

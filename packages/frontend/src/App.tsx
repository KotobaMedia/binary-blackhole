import React from "react";
import { Redirect, Route, Switch } from "wouter";
import LandingPage from "./pages/LandingPage";
import ChatMapPageWrapper from "./pages/ChatMapPageWrapper";
import ChatListPage from "./pages/ChatListPage";
import DataNavigatorPage from "./pages/DataNavigatorPage";

const App: React.FC = () => {
  return (
    <Switch>
      <Route path="/" component={LandingPage} />
      <Route path="/chats" component={ChatListPage} />
      <Route path="/chats/:threadId" component={ChatMapPageWrapper} />
      <Route path="/data" component={DataNavigatorPage} />
      <Route>
        <Redirect to="/" />
      </Route>
    </Switch>
  );
};

export default App;

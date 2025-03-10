import React from "react";
import Header from "./Header";

const AppLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  return (
    <div className="container">
      <div className="row">
        <div className="col-12 d-flex flex-column min-vh-100">
          <Header />
          <main className="flex-grow-1">
            {children}
          </main>
          <footer className="text-center py-2 bg-body bg-opacity-75">
            &copy; {new Date().getFullYear()} <a href="https://kotobamedia.com" target="_blank" rel="noopener noreferrer" className="text-body-secondary">KotobaMedia</a>. All rights reserved.
          </footer>
        </div>
      </div>
    </div>
  );
};

export default AppLayout;

import React from "react";
import logo from "/logo-small.svg?url";
import { Link } from "wouter";

const Header: React.FC = () => {
  return (
    <nav className="navbar navbar-expand-lg position-sticky top-0 bg-body bg-opacity-75">
      <div className="container-fluid">
        <Link href="/" className="navbar-brand">
          <img
            src={logo}
            alt="logo"
            width="30"
            height="30"
            className="d-inline-block align-middle"
          />
          <span className="ms-1">BinaryBlackhole</span>
        </Link>
      </div>
    </nav>
  );
};

export default Header;

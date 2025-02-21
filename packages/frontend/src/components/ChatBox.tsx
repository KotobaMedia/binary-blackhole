import React, { JSX, useCallback, useState } from "react";
import logo from "/public/logo-small.svg";
import { QuestionCircleFill } from "react-bootstrap-icons";

const AssistantMessage: React.FC<React.PropsWithChildren<{}>> = ({children}) => {
  return (
    <div className="d-flex justify-content-start mb-2">
      <div className="p-2 rounded" style={{"maxWidth": "90%"}}>
        {children}
      </div>
    </div>
  );
}

const UserMessage: React.FC<React.PropsWithChildren<{}>> = ({children}) => {
  return (
    <div className="d-flex justify-content-end mb-2">
      <div className="bg-primary text-white p-2 rounded" style={{"maxWidth": "90%"}}>
        {children}
      </div>
    </div>
  );
}

const SendMessageBox: React.FC = () => {
  const onSubmit = useCallback<React.FormEventHandler<HTMLFormElement>>((e) => {
    e.preventDefault();

  }, []);
  const onInput = useCallback<React.FormEventHandler<HTMLTextAreaElement>>((e) => {
    const el = e.currentTarget;
    const maxHeight = 150; // maximum height in pixels
    el.style.height = "auto";
    if (el.scrollHeight < maxHeight) {
      el.style.height = `${el.scrollHeight}px`;
      el.style.overflowY = "hidden";
    } else {
      el.style.height = `${maxHeight}px`;
      el.style.overflowY = "auto";
    }
  }, []);

  return (
    <form onSubmit={onSubmit}>
      <div className="input-group">
        <textarea
          className="form-control"
          placeholder="何を調べましょうか？"
          rows={1}
          style={{ overflow: "hidden", resize: "none", maxHeight: "150px" }}
          onInput={onInput}
        />
        <button className="btn btn-primary" type="submit">
          <QuestionCircleFill className="align-baseline" title="Submit" />
        </button>
      </div>
    </form>
  );
}

const ChatBox: React.FC = () => {
  const messages: JSX.Element[] = [];
  // for (let i = 0; i < 4; i++) {
  //   if (i % 2 === 0) {
  //     messages.push(<AssistantMessage key={i}>
  //       <strong>Assistant:</strong> Hello, how can I help you today?
  //     </AssistantMessage>);
  //   } else {
  //     messages.push(<UserMessage key={i}>
  //       Hi, I need some assistance with my account.
  //     </UserMessage>);
  //   }
  // }

  return (
    <div className="col-4 d-flex flex-column h-100 overflow-auto">
      <nav className="navbar navbar-expand-lg navbar-dark bg-dark position-sticky top-0">
        <div className="container-fluid">
          <a className="navbar-brand" href="#">
            <img src={logo} alt="logo" width="30" height="30" className="d-inline-block align-middle" />
            <span className="ms-1">BinaryBlackhole</span>
          </a>
        </div>
      </nav>

      <div className="d-flex flex-column flex-grow-1">
        {messages}

        <div className="position-sticky bottom-0 mt-auto py-3">
          <SendMessageBox />
        </div>
      </div>
    </div>
  );
}

export default ChatBox;

'use client';

import React from "react";
import CodeEditor from '@uiw/react-textarea-code-editor';
import JsonView from '@uiw/react-json-view';
import 'react-json-view-lite/dist/index.css';
import { defaultFont } from '../styles/fonts'
import SplitPane, { Pane } from 'split-pane-react';
import init, { parse } from "../pkg/index";

const EditorView = () => {
  const [code, setCode] = React.useState(
    `SELECT * FROM foo;`
  );

  const [ast, setAst] = React.useState({});

  const [sizes, setSizes] = React.useState([100, '30%', 'auto']);

  function updateCode(code: string) {
    setCode(code)
    init().then(() => {
      try {
        const parsed = parse(code).root;
        setAst(parsed);
      }
      catch (e) {
        console.error(e);
      }
    });
  }

  return (
    <SplitPane sizes={sizes} onChange={setSizes} className={defaultFont.className}>
      <Pane minSize={300} className="h-screen">
        <div className="h-full overflow-auto text-md">
          <CodeEditor
            value={code}
            language="js"
            placeholder="Please enter JS code."
            onChange={(evn) => updateCode(evn.target.value)}
            padding={15}
          />
        </div>
      </Pane>
      <Pane minSize={600} className="h-full">
        <div className="overflow-y-auto h-full">
          <JsonView value={ast} />
        </div>
      </Pane>
    </SplitPane>
  );
}

export default EditorView;
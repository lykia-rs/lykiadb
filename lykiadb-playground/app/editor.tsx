'use client';
import Editor from "react-simple-code-editor";
import { JsonView, allExpanded, darkStyles } from 'react-json-view-lite';
import React from "react";
import { highlight, languages } from 'prismjs/components/prism-core';
import 'prismjs/components/prism-clike';
import 'prismjs/components/prism-javascript';
import 'prismjs/themes/prism.css';
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
      <Pane minSize={300}>
        <Editor
          insertSpaces={true}
          value={code}
          onValueChange={(code: string) => updateCode(code)}
          highlight={
            (code: string) => {
                return highlight(code, languages.js)
            }
          }
          padding={10}
          style={{
            fontFamily: '',
            fontSize: 16,
          }}
        />
      </Pane>
      <Pane minSize={600} className="h-full">
        <div className="overflow-y-auto h-full">
          <JsonView data={ast} shouldExpandNode={allExpanded} style={darkStyles} />
        </div>
      </Pane>
    </SplitPane>
  );
}

export default EditorView;
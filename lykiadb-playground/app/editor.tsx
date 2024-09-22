'use client';

import React from "react";
import CodeMirror from '@uiw/react-codemirror';
import JsonView from '@uiw/react-json-view';
import { defaultFont } from '../styles/fonts'
import SplitPane, { Pane } from 'split-pane-react';
import init, { parse, tokenize } from "../pkg/index";
import { lyql } from "./parser";
import "./lyqlSyntax.scss";

await init();

const EditorView = () => {
  const [code, setCode] = React.useState(
`SELECT { 'name': user.name } from users;
`);

  const [ast, setAst] = React.useState({});

  const [sizes, setSizes] = React.useState(['50%', '50%']);

  function updateCode(code: string) {
    setCode(code)
    try {
      const parsed = parse(code);
      if (parsed) setAst(parsed);
    }
    catch (e) {
      console.error(e);
    }
  }

  return (
    <SplitPane sizes={sizes} onChange={setSizes} className="border-y border-l border-zinc-500">
      <Pane minSize={600} className="h-full">
        <div className="p-4 text-md text-white bg-zinc-800 border-y border-l border-zinc-500"></div>
        <div>
          <CodeMirror
            className={defaultFont.className}
            value={code}
            extensions={[lyql(tokenize)]} 
            onChange={(value: string) => updateCode(value)} 
          />
        </div>
        <div className="p-2 text-white bg-zinc-800"></div>
      </Pane>
      <Pane minSize={600} className="h-full border-l border-zinc-500">
        <div className="p-2 text-md text-white bg-zinc-800 border-y border-r border-zinc-500">AST</div>
        <div className="overflow-y-auto">
          <div className="p-3 bg-white"><JsonView value={ast} /></div>
        </div>
        <div className="p-2 text-white bg-zinc-800"></div>
      </Pane>
    </SplitPane>
  );
}

export default EditorView;
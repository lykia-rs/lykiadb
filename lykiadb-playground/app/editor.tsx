'use client';

import React from "react";
import CodeMirror from '@uiw/react-codemirror';
import JsonView from '@uiw/react-json-view';
import { defaultFont } from '../styles/fonts'
import SplitPane, { Pane } from 'split-pane-react';
import init, { parse } from "../pkg/index";
import { lyql } from "./parser";
import "./lyqlSyntax.scss";

await init();

const EditorView = () => {
  const [code, setCode] = React.useState(
`var $calc = {
  add: function ($a, $b) {
    return $a + $b;
  },
  sub: function ($a, $b) {
    return $a - $b;
  },
  mul: function ($a, $b) {
    return $a * $b;
  },
  div: function ($a, $b) {
    return $a / $b;
  },
};
print($calc.add(4, 5));
print($calc.sub(4, 5));
print($calc.mul(4, 5));
print($calc.div(4, 5));
`);

  const [ast, setAst] = React.useState({});

  const [sizes, setSizes] = React.useState([100, '30%', 'auto']);

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
    <SplitPane sizes={sizes} onChange={setSizes} className={defaultFont.className}>
      <Pane minSize={300} className="h-full p-1">
        <div className="p-2 text-white bg-slate-700 rounded-t-md">Script</div>
        <div>
          <CodeMirror
            value={code}
            height="400px"

            extensions={[lyql(parse)]} 
            onChange={(value: string) => updateCode(value)} 
          />
        </div>
        <div className="p-2 text-white bg-slate-700 rounded-b-md"></div>
      </Pane>
      <Pane minSize={600} className="h-full p-1">
        <div className="p-2 text-white bg-slate-700 rounded-t-md">Syntax tree</div>
        <div className="overflow-y-auto h-full">
          <div className="p-3 bg-white"><JsonView value={ast} /></div>
        </div>
      </Pane>
    </SplitPane>
  );
}

export default EditorView;
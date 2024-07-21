'use client';

// import init, { parse } from "@/pkg/lykiadb_playground";
import Editor from "react-simple-code-editor";
import React, { useEffect } from "react";
import { highlight, languages } from 'prismjs/components/prism-core';
import 'prismjs/components/prism-clike';
import 'prismjs/components/prism-javascript';
import 'prismjs/themes/prism.css'; //Example style, you can use another

const EditorView = () => {
  const [code, setCode] = React.useState(
    `SELECT * FROM foo;`
  );
  // await init();
  return (
    <Editor
      value={code}
      onValueChange={code => setCode(code)}
      highlight={
        code => {
            return highlight(code, languages.js)
        }
      }
      padding={10}
      style={{
        fontFamily: '"Fira code", "Fira Mono", monospace',
        fontSize: 12,
      }}
    />
  );
}

export default EditorView;
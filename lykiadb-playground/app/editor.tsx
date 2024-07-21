'use client';
import Editor from "react-simple-code-editor";
import React from "react";
import { highlight, languages } from 'prismjs/components/prism-core';
import 'prismjs/components/prism-clike';
import 'prismjs/components/prism-javascript';
import 'prismjs/themes/prism.css';

const EditorView = () => {
  const [code, setCode] = React.useState(
    `SELECT * FROM foo;`
  );
  return (
    <Editor
      value={code}
      onValueChange={(code: string) => setCode(code)}
      highlight={
        (code: string) => {
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
'use client';

import dynamic from 'next/dynamic';
import 'split-pane-react/esm/themes/default.css';

const EditorView = dynamic(() => import('./editor'), {
  ssr: false,
});


export default function Home() {
  return (
    <main className="min-h-screen">
      <div className="h-screen p-6">
        <EditorView />
      </div>
    </main>
  );
}
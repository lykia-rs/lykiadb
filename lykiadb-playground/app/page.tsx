'use client';

import dynamic from 'next/dynamic';
import GridLayout from "react-grid-layout";
import 'react-grid-layout/css/styles.css';

const EditorView = dynamic(() => import('./editor'), {
  ssr: false,
});


// layout is an array of objects, see the demo for more complete usage
const layout = [
  { i: "a", x: 0, y: 0, w: 6, h: 12, isResizable: true, resizeHandles: ['se'] },
  { i: "b", x: 6, y: 0, w: 3, h: 6, isResizable: true, resizeHandles: ['se'] },
  { i: "c", x: 6, y: 0, w: 3, h: 6, isResizable: true, resizeHandles: ['se'] }
];

export default function Home() {
  return (
    <main className="min-h-screen p-6">

      <GridLayout
        className="layout"
        layout={layout}
        cols={12}
        rowHeight={30}
        width={1200}
      >
        <div className="bg-white p-3 rounded-md" key="a">
          <EditorView />
        </div>
        <div className="bg-white p-3 rounded-md" key="b">b</div>
        <div className="bg-white p-3 rounded-md" key="c">c</div>
      </GridLayout>
    </main>
  );
}
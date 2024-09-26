'use client';

import dynamic from 'next/dynamic';
import 'split-pane-react/esm/themes/default.css';
const MainView = dynamic(() => import('./main'), {
  ssr: false,
});

export default function Home() {
  return (
    <main className="min-h-screen">
      <div className="h-screen">
      <MainView
          defaultLayout={undefined}
          defaultCollapsed={undefined}
          navCollapsedSize={4}
        />
      </div>
    </main>
  );
}
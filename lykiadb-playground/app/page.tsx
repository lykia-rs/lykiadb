import dynamic from 'next/dynamic';

const EditorView = dynamic(() => import('./editor'), {
  ssr: false,
});

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <EditorView />
    </main>
  );
}

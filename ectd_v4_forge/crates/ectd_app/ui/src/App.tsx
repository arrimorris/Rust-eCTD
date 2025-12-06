import { useState } from "react";
import InitForm from "./components/InitForm";
import Dashboard from "./components/Dashboard";
import "./App.css"; // Ensure you have basic Tailwind directives here

function App() {
  // The "Session": A Submission UUID we are currently working on
  const [submissionId, setSubmissionId] = useState<string | null>(null);

  return (
    <div className="min-h-screen bg-slate-50 text-slate-900 font-sans">
      <header className="bg-slate-900 text-white p-4 shadow-md flex justify-between items-center">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 bg-blue-500 rounded-md flex items-center justify-center font-bold">F</div>
          <h1 className="text-xl font-semibold tracking-tight">eCTD Forge</h1>
        </div>
        {submissionId && (
          <div className="text-xs font-mono text-slate-400">
            SESSION: {submissionId.slice(0, 8)}...
          </div>
        )}
      </header>

      <main className="max-w-5xl mx-auto p-6">
        {!submissionId ? (
          <div className="max-w-xl mx-auto mt-10">
            <InitForm onSuccess={(uuid) => setSubmissionId(uuid)} />
          </div>
        ) : (
          <Dashboard
            submissionId={submissionId}
            onExit={() => setSubmissionId(null)}
          />
        )}
      </main>
    </div>
  );
}

export default App;

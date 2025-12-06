import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FilePlus, FileText } from "lucide-react";

interface DashboardProps {
  submissionId: string;
  onExit: () => void;
}

export default function Dashboard({ submissionId, onExit }: DashboardProps) {
  const [filePath, setFilePath] = useState("");
  const [context, setContext] = useState("cover-letter");
  const [title, setTitle] = useState("");
  const [logs, setLogs] = useState<string[]>([]);

  const addLog = (msg: string) => setLogs(prev => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev]);

  const handleAttach = async () => {
    if (!filePath || !title) return;

    try {
      addLog(`⏳ Hashing & Uploading: ${filePath}...`);

      // Call Rust Command: add_document
      const docId = await invoke<string>("add_document", {
        submissionId,
        filePath,
        context,
        title
      });

      addLog(`✅ Success! Document attached. UUID: ${docId}`);
      setFilePath(""); // Clear input
      setTitle("");
    } catch (err) {
      addLog(`❌ Error: ${err}`);
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
      {/* LEFT: Actions */}
      <div className="lg:col-span-2 space-y-6">

        {/* Status Card */}
        <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-start justify-between">
          <div>
            <h2 className="text-lg font-semibold text-slate-800">Active Submission</h2>
            <p className="text-slate-500 text-sm font-mono mt-1">{submissionId}</p>
          </div>
          <button onClick={onExit} className="text-sm text-slate-400 hover:text-slate-600">Close Session</button>
        </div>

        {/* Add Document Form */}
        <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 bg-indigo-100 rounded-lg">
              <FilePlus className="w-5 h-5 text-indigo-600" />
            </div>
            <h3 className="font-semibold text-slate-800">Attach Document</h3>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-slate-500 uppercase mb-1">Local File Path</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  placeholder="/path/to/file.pdf"
                  className="flex-1 rounded-md border-slate-300 border p-2 text-sm font-mono"
                  value={filePath}
                  onChange={e => setFilePath(e.target.value)}
                />
                {/* Future: Add File Picker Dialog Button here */}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-xs font-medium text-slate-500 uppercase mb-1">Document Title</label>
                <input
                  type="text"
                  placeholder="e.g. Cover Letter"
                  className="w-full rounded-md border-slate-300 border p-2 text-sm"
                  value={title}
                  onChange={e => setTitle(e.target.value)}
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-slate-500 uppercase mb-1">Context of Use</label>
                <select
                  className="w-full rounded-md border-slate-300 border p-2 text-sm"
                  value={context}
                  onChange={e => setContext(e.target.value)}
                >
                  <option value="cover-letter">Cover Letter (m1)</option>
                  <option value="product-labeling">Product Labeling (m1)</option>
                  <option value="clinical-dataset">Clinical Dataset (m5)</option>
                  <option value="generic-document">Generic Document</option>
                </select>
              </div>
            </div>

            <button
              onClick={handleAttach}
              disabled={!filePath || !title}
              className="w-full bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white font-medium py-2 rounded-lg transition-colors"
            >
              Attach to Submission
            </button>
          </div>
        </div>
      </div>

      {/* RIGHT: Activity Log */}
      <div className="bg-slate-900 text-slate-300 p-4 rounded-xl shadow-inner h-[500px] overflow-y-auto font-mono text-xs">
        <div className="flex items-center gap-2 mb-4 text-slate-400 border-b border-slate-800 pb-2">
          <FileText className="w-4 h-4" />
          <span>System Log</span>
        </div>
        <div className="space-y-2">
          {logs.length === 0 && <span className="text-slate-600 italic">Ready for operations...</span>}
          {logs.map((log, i) => (
            <div key={i} className={log.includes("Error") ? "text-red-400" : log.includes("Success") ? "text-green-400" : ""}>
              {log}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

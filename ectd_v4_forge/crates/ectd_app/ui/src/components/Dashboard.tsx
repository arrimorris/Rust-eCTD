import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FilePlus, FileText, ShieldCheck, Download, AlertTriangle, CheckCircle } from "lucide-react";

interface DashboardProps {
  submissionId: string;
  onExit: () => void;
}

export default function Dashboard({ submissionId, onExit }: DashboardProps) {
  const [filePath, setFilePath] = useState("");
  const [context, setContext] = useState("cover-letter");
  const [title, setTitle] = useState("");
  const [logs, setLogs] = useState<string[]>([]);

  // NEW State for Validation/Export
  const [validationErrors, setValidationErrors] = useState<string[] | null>(null);

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

  const handleValidate = async () => {
    try {
      addLog("🔍 Running Validation Engine...");
      const errors = await invoke<string[]>("validate_submission", { submissionId });

      if (errors.length === 0) {
        addLog("✅ Validation Passed! No errors found.");
        setValidationErrors([]);
      } else {
        addLog(`⚠️ Found ${errors.length} Validation Errors.`);
        setValidationErrors(errors);
      }
    } catch (err) {
      addLog(`❌ Validation System Error: ${err}`);
    }
  };

  const handleExport = async () => {
    try {
      // In a real app, use the Dialog plugin to pick a folder.
      // For MVP, we'll just ask for a path string or default to Downloads.
      // Assuming user inputs path for now or hardcoded for demo.
      // For this "appliance" model, we'll try a common temp location or relative path
      // Actually, let's just use a hardcoded safe path for now or ask user if possible
      // But we don't have a prompt implementation, so let's default to `/tmp/ectd_export`
      const targetDir = "/tmp/ectd_export_" + submissionId.slice(0,8);

      addLog(`📦 Exporting to: ${targetDir}...`);
      await invoke("export_submission", { submissionId, targetDir });
      addLog("🎉 Export Complete!");
    } catch (err) {
      addLog(`❌ Export Failed: ${err}`);
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

        {/* NEW: Exit Doors Panel (Validate & Export) */}
        <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 bg-emerald-100 rounded-lg">
              <ShieldCheck className="w-5 h-5 text-emerald-600" />
            </div>
            <h3 className="font-semibold text-slate-800">Finalize & Ship</h3>
          </div>

          <div className="flex gap-4">
            <button
              onClick={handleValidate}
              className="flex-1 bg-amber-50 hover:bg-amber-100 text-amber-700 border border-amber-200 font-medium py-3 rounded-lg flex items-center justify-center gap-2 transition-colors"
            >
              <AlertTriangle className="w-4 h-4" /> Validate
            </button>
            <button
              onClick={handleExport}
              className="flex-1 bg-emerald-50 hover:bg-emerald-100 text-emerald-700 border border-emerald-200 font-medium py-3 rounded-lg flex items-center justify-center gap-2 transition-colors"
            >
              <Download className="w-4 h-4" /> Export Package
            </button>
          </div>

          {/* Validation Results Display */}
          {validationErrors !== null && (
            <div className={`mt-4 p-4 rounded-lg text-sm border ${validationErrors.length === 0 ? "bg-green-50 border-green-200 text-green-700" : "bg-red-50 border-red-200 text-red-700"}`}>
              {validationErrors.length === 0 ? (
                <div className="flex items-center gap-2">
                  <CheckCircle className="w-4 h-4" />
                  <strong>Ready to Submit:</strong> eCTD structure is compliant.
                </div>
              ) : (
                <ul className="list-disc pl-4 space-y-1">
                  {validationErrors.map((err, i) => <li key={i}>{err}</li>)}
                </ul>
              )}
            </div>
          )}
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

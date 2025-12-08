import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FilePlus, ShieldCheck, Download, AlertTriangle, CheckCircle, Activity, Server } from "lucide-react";

interface DashboardProps {
  submissionId: string;
  onExit: () => void;
}

interface ExportProgress {
  fileName: string;
  processedFiles: number;
  totalFiles: number;
  bytesProcessed: number;
  status: string;
}

export default function Dashboard({ submissionId, onExit }: DashboardProps) {
  const [filePath, setFilePath] = useState("");
  const [context, setContext] = useState("cover-letter");
  const [title, setTitle] = useState("");
  const [logs, setLogs] = useState<string[]>([]);
  const [validationErrors, setValidationErrors] = useState<string[] | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [exportProgress, setExportProgress] = useState<ExportProgress | null>(null);
  const [systemHealth, setSystemHealth] = useState<"checking" | "ok" | "error">("checking");

  const addLog = (msg: string) => setLogs(prev => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev]);

  useEffect(() => {
    checkSystem();
    const unlisten = listen<ExportProgress>('export-progress', (event) => {
      setExportProgress(event.payload);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  const checkSystem = async () => {
    try {
      await invoke("ensure_infrastructure");
      setSystemHealth("ok");
      addLog("✅ System Health Check Passed (Database Connected)");
    } catch (e) {
      setSystemHealth("error");
      addLog(`❌ SYSTEM FAILURE: ${e}`);
    }
  };

  const handleAttach = async () => {
    if (!filePath || !title) return;
    try {
      addLog(`⏳ Uploading: ${filePath}...`);
      const docId = await invoke<string>("add_document", {
        submissionId,
        filePath,
        context,
        title
      });
      addLog(`✅ Attached. UUID: ${docId}`);
      setFilePath(""); setTitle("");
    } catch (err) {
      addLog(`❌ Error: ${err}`);
    }
  };

  const handleValidate = async () => {
    try {
      addLog("🔍 Validating...");
      const errors = await invoke<string[]>("validate_submission", { submissionId });
      if (errors.length === 0) {
        addLog("✅ Validation Passed!");
        setValidationErrors([]);
      } else {
        addLog(`⚠️ Found ${errors.length} Errors.`);
        setValidationErrors(errors);
      }
    } catch (err) {
      addLog(`❌ Validation Error: ${err}`);
    }
  };

  const handleExport = async () => {
    try {
      setIsExporting(true);
      const targetDir = "/tmp/ectd_export_" + submissionId.slice(0,8);
      addLog(`📦 Starting Stream to: ${targetDir}`);
      await invoke("export_submission", { submissionId, targetDir });
      addLog("🎉 Export Stream Complete!");
    } catch (err) {
      addLog(`❌ Export Failed: ${err}`);
    } finally {
      setIsExporting(false);
      setExportProgress(null);
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <div className="lg:col-span-2 space-y-6">
        <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-start justify-between">
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-lg font-semibold text-slate-800">Active Submission</h2>
              {systemHealth === "ok" ?
                <span className="bg-green-100 text-green-700 text-xs px-2 py-0.5 rounded-full flex items-center gap-1"><Server className="w-3 h-3"/> Online</span> :
                <span className="bg-red-100 text-red-700 text-xs px-2 py-0.5 rounded-full flex items-center gap-1"><AlertTriangle className="w-3 h-3"/> Offline</span>
              }
            </div>
            <p className="text-slate-500 text-sm font-mono mt-1">{submissionId}</p>
          </div>
          <button onClick={onExit} className="text-sm text-slate-400 hover:text-slate-600">Close Session</button>
        </div>

        <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 bg-indigo-100 rounded-lg"><FilePlus className="w-5 h-5 text-indigo-600" /></div>
            <h3 className="font-semibold text-slate-800">Attach Document</h3>
          </div>
          <div className="space-y-4">
            <input type="text" placeholder="/path/to/file.pdf" className="w-full rounded-md border-slate-300 border p-2 text-sm font-mono" value={filePath} onChange={e => setFilePath(e.target.value)} />
            <div className="grid grid-cols-2 gap-4">
              <input type="text" placeholder="Title" className="w-full rounded-md border-slate-300 border p-2 text-sm" value={title} onChange={e => setTitle(e.target.value)} />
              <select className="w-full rounded-md border-slate-300 border p-2 text-sm" value={context} onChange={e => setContext(e.target.value)}>
                <option value="cover-letter">Cover Letter (m1)</option>
                <option value="product-labeling">Labeling (m1)</option>
                <option value="clinical-dataset">Dataset (m5)</option>
              </select>
            </div>
            <button onClick={handleAttach} disabled={!filePath || !title} className="w-full bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white font-medium py-2 rounded-lg transition-colors">Attach to Submission</button>
          </div>
        </div>

        <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 bg-emerald-100 rounded-lg"><ShieldCheck className="w-5 h-5 text-emerald-600" /></div>
            <h3 className="font-semibold text-slate-800">Finalize & Ship</h3>
          </div>

          {isExporting && exportProgress && (
            <div className="mb-4 bg-slate-50 p-3 rounded-lg border border-slate-200">
              <div className="flex justify-between text-xs mb-1 font-medium text-slate-600">
                <span>{exportProgress.status}</span>
                <span>{exportProgress.processedFiles} / {exportProgress.totalFiles}</span>
              </div>
              <div className="w-full bg-slate-200 rounded-full h-2.5">
                <div className="bg-emerald-500 h-2.5 rounded-full transition-all duration-300" style={{ width: `${(exportProgress.processedFiles / exportProgress.totalFiles) * 100}%` }}></div>
              </div>
              <div className="text-xs text-slate-400 mt-1 truncate font-mono">{exportProgress.fileName}</div>
            </div>
          )}

          <div className="flex gap-4">
            <button onClick={handleValidate} disabled={isExporting} className="flex-1 bg-amber-50 hover:bg-amber-100 text-amber-700 border border-amber-200 font-medium py-3 rounded-lg flex items-center justify-center gap-2 transition-colors">
              <AlertTriangle className="w-4 h-4" /> Validate
            </button>
            <button onClick={handleExport} disabled={isExporting} className="flex-1 bg-emerald-50 hover:bg-emerald-100 text-emerald-700 border border-emerald-200 font-medium py-3 rounded-lg flex items-center justify-center gap-2 transition-colors">
              <Download className="w-4 h-4" /> {isExporting ? "Exporting..." : "Export Package"}
            </button>
          </div>

          {validationErrors !== null && (
            <div className={`mt-4 p-4 rounded-lg text-sm border ${validationErrors.length === 0 ? "bg-green-50 border-green-200 text-green-700" : "bg-red-50 border-red-200 text-red-700"}`}>
              {validationErrors.length === 0 ? (
                <div className="flex items-center gap-2"><CheckCircle className="w-4 h-4" /><strong>Ready to Submit</strong></div>
              ) : (
                <ul className="list-disc pl-4 space-y-1">{validationErrors.map((err, i) => <li key={i}>{err}</li>)}</ul>
              )}
            </div>
          )}
        </div>
      </div>

      <div className="bg-slate-900 text-slate-300 p-4 rounded-xl shadow-inner h-[600px] overflow-y-auto font-mono text-xs flex flex-col">
        <div className="flex items-center gap-2 mb-4 text-slate-400 border-b border-slate-800 pb-2">
          <Activity className="w-4 h-4" /><span>System Log</span>
        </div>
        <div className="space-y-2 flex-1">
          {logs.length === 0 && <span className="text-slate-600 italic">Ready for operations...</span>}
          {logs.map((log, i) => (
            <div key={i} className={log.includes("Error") || log.includes("FAILURE") ? "text-red-400" : log.includes("Success") || log.includes("Passed") ? "text-green-400" : ""}>
              {log}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

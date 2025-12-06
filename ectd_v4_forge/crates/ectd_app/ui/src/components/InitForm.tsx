import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Rocket, Loader2 } from "lucide-react";

interface InitFormProps {
  onSuccess: (uuid: string) => void;
}

export default function InitForm({ onSuccess }: InitFormProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form State
  const [formData, setFormData] = useState({
    app_number: "123456",
    app_type: "nda",
    applicant: "Acme Pharmaceuticals",
    sequence: 1,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      // Call Rust Command: init_submission
      // Signature matches crates/ectd_app/src/commands.rs
      const uuid = await invoke<string>("init_submission", {
        args: {
          app_number: formData.app_number,
          app_type: formData.app_type,
          applicant: formData.applicant,
          sequence: Number(formData.sequence),
        }
      });

      onSuccess(uuid);
    } catch (err) {
      console.error(err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-white rounded-xl shadow-lg border border-slate-200 p-8">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-slate-800 flex items-center gap-2">
          <Rocket className="w-6 h-6 text-blue-600" />
          Initialize Submission
        </h2>
        <p className="text-slate-500">Create a new eCTD v4.0 Submission Unit shell.</p>
      </div>

      {error && (
        <div className="bg-red-50 text-red-600 p-3 rounded-md mb-4 text-sm border border-red-200">
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Application Type</label>
            <select
              className="w-full rounded-md border-slate-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 p-2 border"
              value={formData.app_type}
              onChange={e => setFormData({...formData, app_type: e.target.value})}
            >
              <option value="nda">NDA (New Drug Application)</option>
              <option value="bla">BLA (Biologics License)</option>
              <option value="ind">IND (Investigational)</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">App Number</label>
            <input
              type="text"
              className="w-full rounded-md border-slate-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 p-2 border"
              value={formData.app_number}
              onChange={e => setFormData({...formData, app_number: e.target.value})}
            />
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-700 mb-1">Applicant Name</label>
          <input
            type="text"
            className="w-full rounded-md border-slate-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 p-2 border"
            value={formData.applicant}
            onChange={e => setFormData({...formData, applicant: e.target.value})}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-700 mb-1">Sequence Number</label>
          <input
            type="number"
            className="w-full rounded-md border-slate-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 p-2 border"
            value={formData.sequence}
            onChange={e => setFormData({...formData, sequence: Number(e.target.value)})}
          />
        </div>

        <button
          type="submit"
          disabled={loading}
          className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2.5 rounded-lg transition-colors flex justify-center items-center gap-2"
        >
          {loading ? <Loader2 className="animate-spin w-5 h-5" /> : "Create Submission Shell"}
        </button>
      </form>
    </div>
  );
}

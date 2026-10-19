using System.Globalization;
using EmuSen.Crystal.Recording;

namespace EmuSen.WiseMan.Crystal
{
    // The recorder's verbs over the C# FrameRecorder, its folder's local time taken out, as emusen-crystal-debug's recording module runs them - see EmuSen_Backend.md §29.36.
    public sealed partial class CrystalDebugScript
    {
        private FrameRecorder? _recorder;
        private string? _recording;
        private readonly List<string> _recorded = new();

        private string? RecordingAnswer(string verb, string[] words)
        {
            if (!verb.StartsWith("rec.", StringComparison.Ordinal)) return null;
            string W(int i) => i < words.Length ? words[i] : "";
            _recorder ??= new FrameRecorder(_fake, _recorded.Add);
            switch (verb)
            {
                case "rec.start":
                    try
                    {
                        _recording = _recorder.Start(W(0), int.TryParse(W(1), out int s) ? s : 1);
                        return "ok " + Path.GetFileName(_recording).Split('_')[0];
                    }
                    catch (InvalidOperationException e) { return "! " + e.Message; }
                case "rec.recording": return Lower(_recorder.IsRecording);
                case "rec.capture":
                    _fake.Frame = Number(W(0));
                    _recorder.CaptureFrame(path => File.Copy(W(1), path));
                    return "ok";
                case "rec.stop":
                {
                    if (!_recorder.IsRecording) return "not recording";
                    _recorded.Clear();
                    string folder = _recorder.SessionDir!;
                    _recorder.Stop();
                    return string.Join(" | ", _recorded.Where(l => l.StartsWith("[RECORD]", StringComparison.Ordinal)).Select(l => l.Replace(folder, "<folder>", StringComparison.Ordinal)));
                }
                case "rec.files":
                    if (_recording is null) return "none";
                    return string.Join(",", Directory.EnumerateFiles(_recording).Select(f =>
                    {
                        string name = Path.GetFileName(f);
                        long size = new FileInfo(f).Length;
                        return name.EndsWith(".mkv", StringComparison.Ordinal) || name.EndsWith(".zip", StringComparison.Ordinal) ? $"{name}:{Lower(size > 0)}" : $"{name}:{size.ToString(CultureInfo.InvariantCulture)}";
                    }).OrderBy(n => n, StringComparer.Ordinal));
                case "rec.ledger":
                    if (_recording is null) return "none";
                    return string.Join("|", File.ReadAllLines(Path.Combine(_recording, "frames.log")).Select(l =>
                    {
                        string[] f = l.Split('\t');
                        if (f.Length > 1 && f[0] != "Frame") f[1] = "-";
                        return string.Join(" ", f);
                    }));
                default: return $"unknown verb {verb}";
            }
        }
    }
}

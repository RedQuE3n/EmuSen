using System;
using System.IO;

namespace EmuSen.WiseMan.Fixtures
{
    // A run's folder at the same path in every run whatever TMPDIR is, so a window that shows it lays out alike - see EmuSen_InterfaceSpec.md §6.1.
    public sealed class FixedRunFolder : IDisposable
    {
        private readonly FileStream _claim;

        public FixedRunFolder(string name)
        {
            string parent = Path.Combine(OperatingSystem.IsWindows() ? Path.GetTempPath() : "/tmp", name);
            Directory.CreateDirectory(parent);
            for (int slot = 1; ; slot++)
            {
                string root = Path.Combine(parent, slot.ToString("D3"));
                try { _claim = new FileStream(root + ".lock", FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None); }
                catch (IOException) { continue; }
                if (Directory.Exists(root)) Directory.Delete(root, recursive: true);
                Root = root;
                return;
            }
        }

        public string Root { get; }

        // Any run folder's numbered slot, under /tmp or the temporary folder, as recordings must not tell one slot from another.
        public static readonly System.Text.RegularExpressions.Regex Slot = new(
            $"(?:/tmp|{System.Text.RegularExpressions.Regex.Escape(Path.GetTempPath().TrimEnd('/', '\\'))})[/\\\\]EmuSen[A-Za-z]+[/\\\\][0-9]{{3}}(?![0-9])");

        public void Dispose()
        {
            try { if (Directory.Exists(Root)) Directory.Delete(Root, recursive: true); } catch { }
            _claim.Dispose();
        }
    }
}

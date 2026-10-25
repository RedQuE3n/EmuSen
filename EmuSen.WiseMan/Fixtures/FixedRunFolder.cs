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

        public void Dispose()
        {
            try { if (Directory.Exists(Root)) Directory.Delete(Root, recursive: true); } catch { }
            _claim.Dispose();
        }
    }
}

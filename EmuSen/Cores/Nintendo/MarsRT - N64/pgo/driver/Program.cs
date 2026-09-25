using System.Diagnostics;
using System.Security.Cryptography;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;

// marsrt-pgo <rom> <state|-> <frames> [Setting=value]... [rewind=1]: Mistress's frame loop on MarsRT, one line of ms a frame and the state's SHA-256 - see Mars_Native.md §6.17.
if (args.Length < 3 || !int.TryParse(args[2], out int frames))
{
    Console.Error.WriteLine("usage: marsrt-pgo <rom> <state|-> <frames> [ThreadedRdp=false] [RdpWorkers=n] [RenderScale=n] [Gpu=true] [Recompiler=false] [rewind=1]");
    return 2;
}
string rom = args[0], statePath = args[1];
var settings = args.Skip(3).Select(a => a.Split('=', 2)).Where(p => p.Length == 2).ToList();
bool rewind = settings.RemoveAll(p => p[0] == "rewind" && p[1] == "1") > 0;

string scratch = Path.Combine(Path.GetTempPath(), "marsrt-pgo-" + Environment.ProcessId);
EmuSen.Galaxia.Library.DataStore.OverrideDirectory = Path.Combine(scratch, "home");
EmuSen.Galaxia.ConfigStore.OverrideDirectory = Path.Combine(scratch, "config");
CoreOptions.BatteryRamDisabled = true;
try
{
    var session = new EmulatorSession { Engine = CoreCatalog.MarsRtEngine };
    session.LoadRom(rom);
    if (session.Core is not MarsRtCore core)
    {
        Console.Error.WriteLine($"MarsRT is not running: {session.EngineNotice ?? MarsNative.Report}");
        return 3;
    }
    foreach (var pair in settings) core.Set(pair[0], pair[1]);
    if (statePath != "-") core.LoadState(new MemoryStream(File.ReadAllBytes(statePath)));
    var buffer = rewind ? new RewindBuffer { Enabled = true } : null;

    long serial = -1;
    var clock = Stopwatch.StartNew();
    for (int i = 0; i < frames; i++)
    {
        session.RunFrame();
        session.DebugTarget?.RefreshProviders();
        buffer?.OnFrameCompleted(core);
        session.DequeueAudioSamples(int.MaxValue);
        if (session.FrameSerial is not long now || now != serial)
        {
            serial = session.FrameSerial ?? -1;
            core.ReturnFrameBuffer(session.GetFrameBufferRgba());
        }
    }
    double ms = clock.Elapsed.TotalMilliseconds / frames;

    using var state = new MemoryStream();
    core.SaveState(state);
    string hash = Convert.ToHexString(SHA256.HashData(state.ToArray()))[..16];
    string shown = string.Join(' ', settings.Select(p => $"{p[0]}={p[1]}").Append(rewind ? "rewind=1" : "").Where(s => s.Length > 0));
    Console.WriteLine($"{Path.GetFileName(rom)} {(statePath == "-" ? "power-on" : Path.GetFileName(statePath))} {frames} frames: {ms:F3} ms a frame, state {hash}, {core.RdpWorkers} workers{(shown.Length > 0 ? ", " + shown : "")}");
    core.Dispose();
    return 0;
}
finally
{
    try { Directory.Delete(scratch, recursive: true); } catch (IOException) { }
}

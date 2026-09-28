using System.Diagnostics;
using System.Security.Cryptography;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;

// marsrt-pgo <rom> <state|-> <frames> [Setting=value]... [rewind=1] [mistress=1] [warmup=n]: Mistress's frame loop on MarsRT, ms a frame and the state's SHA-256 - see Mars_Native.md §6.17.
if (args.Length < 3 || !int.TryParse(args[2], out int frames))
{
    Console.Error.WriteLine("usage: marsrt-pgo <rom> <state|-> <frames> [ThreadedRdp=false] [RdpWorkers=n] [RenderScale=n] [Gpu=true] [Recompiler=false] [rewind=1] [mistress=1] [warmup=n]");
    return 2;
}
string rom = args[0], statePath = args[1];
var settings = args.Skip(3).Select(a => a.Split('=', 2)).Where(p => p.Length == 2).ToList();
bool rewind = settings.RemoveAll(p => p[0] == "rewind" && p[1] == "1") > 0;
bool mistress = settings.RemoveAll(p => p[0] == "mistress" && p[1] == "1") > 0;
int warmup = settings.Where(p => p[0] == "warmup").Select(p => int.Parse(p[1])).LastOrDefault();
settings.RemoveAll(p => p[0] == "warmup");

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

    // mistress=1 is MainWindow's rewind: a snapshot four times a second with its thumbnail - see EmuSen_Settings_Reference.md §4.49.
    RewindBuffer? buffer = mistress ? new RewindBuffer { Enabled = true, ThumbnailWidth = RewindBuffer.DefaultThumbnailWidth, IntervalFrames = Math.Max(1, (int)Math.Round(session.FrameRateHz / 4)) }
        : rewind ? new RewindBuffer { Enabled = true } : null;
    long? offered = null, thumbnailSerial = null;
    RewindThumbnail? thumbnail = null;

    var clock = new Stopwatch();
    double runMs = 0;
    for (int i = 0; i < warmup + frames; i++)
    {
        if (i == warmup) { clock.Restart(); runMs = 0; }
        long start = Stopwatch.GetTimestamp();
        session.RunFrame();
        runMs += Stopwatch.GetElapsedTime(start).TotalMilliseconds;
        session.DebugTarget?.RefreshProviders();
        bool captured = buffer?.OnFrameCompleted(core) ?? false;
        session.DequeueAudioSamples(int.MaxValue);
        long? serial = session.FrameSerial;
        if (serial is null || serial != offered)
        {
            byte[] frame = session.GetFrameBufferRgba();
            if (captured && mistress && buffer!.AttachThumbnail(frame, session.ScreenWidth, session.ScreenHeight, session.RowRepeat) is { } made)
            {
                thumbnail = made;
                thumbnailSerial = serial;
            }
            core.ReturnFrameBuffer(frame);
            offered = serial;
        }
        else if (captured && mistress)
        {
            if (serial == thumbnailSerial && thumbnail is not null) buffer!.AttachThumbnail(thumbnail);
            else
            {
                byte[] frame = session.GetFrameBufferRgba();
                buffer!.AttachThumbnail(frame, session.ScreenWidth, session.ScreenHeight, session.RowRepeat);
                core.ReturnFrameBuffer(frame);
            }
        }
    }
    double ms = clock.Elapsed.TotalMilliseconds / frames;

    using var state = new MemoryStream();
    core.SaveState(state);
    string hash = Convert.ToHexString(SHA256.HashData(state.ToArray()))[..16];
    string shown = string.Join(' ', settings.Select(p => $"{p[0]}={p[1]}").Append(rewind ? "rewind=1" : "").Append(mistress ? "mistress=1" : "").Where(s => s.Length > 0));
    Console.WriteLine($"{Path.GetFileName(rom)} {(statePath == "-" ? "power-on" : Path.GetFileName(statePath))} {frames} frames: {ms:F3} ms a frame, run {runMs / frames:F3}, state {hash}, {core.RdpWorkers} workers{(shown.Length > 0 ? ", " + shown : "")}");
    core.Dispose();
    return 0;
}
finally
{
    try { Directory.Delete(scratch, recursive: true); } catch (IOException) { }
}

using System;
using System.Linq;
using System.Threading;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // htop for the .NET VM, not for emulated hardware - see `man vstop`.
    public class VstopCommand : IDianaOSCommand
    {
        // No target parameter, unlike coretop's: nothing here depends on a loaded core.
        private readonly Action? _openWindow;

        public VstopCommand(Action? openWindow = null)
        {
            _openWindow = openWindow;
        }

        public string Name => "vstop";

        // Blocks the terminal until Ctrl+C, same call as `coretop`'s.
        public bool IsReadOnly => false;

        public string Usage => "  vstop [-w]                    live htop-style dashboard of the .NET runtime this process runs on (Ctrl+C to exit - interactive terminal only;\n" +
                                "                                -w opens it in a separate window instead, if this frontend supports one)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Any(a => a.Equals("-w", StringComparison.OrdinalIgnoreCase)))
            {
                if (_openWindow is null) return DianaOSResult.Fail("vstop: -w is not supported by this frontend.");
                _openWindow();
                return DianaOSResult.Ok("vstop: opened in a separate window.");
            }

            if (Console.IsInputRedirected || Console.IsOutputRedirected)
            {
                return DianaOSResult.Fail("vstop: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)");
            }

            return new Dashboard().Run();
        }

        private sealed class Dashboard
        {
            private static readonly TimeSpan RefreshInterval = TimeSpan.FromMilliseconds(500);

            private readonly RuntimeSampler _sampler = new();

            public DianaOSResult Run()
            {
                bool previousTreatCtrlCAsInput = SafeGetTreatCtrlCAsInput();
                try
                {
                    Console.TreatControlCAsInput = true;
                    try { Console.CursorVisible = false; } catch { }

                    // Primes the rate counters, so the first frame shows real numbers - see `man vstop`.
                    _sampler.Sample();

                    while (true)
                    {
                        Draw(_sampler.Sample());

                        DateTime deadline = DateTime.UtcNow + RefreshInterval;
                        while (DateTime.UtcNow < deadline)
                        {
                            if (Console.KeyAvailable)
                            {
                                ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                                bool ctrl = (key.Modifiers & ConsoleModifiers.Control) != 0;
                                if (ctrl && key.Key == ConsoleKey.C) return DianaOSResult.Ok("vstop: exited");
                                if (key.Key == ConsoleKey.Q) return DianaOSResult.Ok("vstop: exited");
                                if (key.Key == ConsoleKey.G) GC.Collect();
                            }
                            Thread.Sleep(20);
                        }
                    }
                }
                finally
                {
                    try { Console.CursorVisible = true; } catch { }
                    Console.TreatControlCAsInput = previousTreatCtrlCAsInput;
                    Console.Clear();
                    Console.SetCursorPosition(0, 0);
                }
            }

            private static bool SafeGetTreatCtrlCAsInput()
            {
                try { return Console.TreatControlCAsInput; } catch { return false; }
            }

            private static void Draw(RuntimeSnapshot s)
            {
                int width = SafeWindowWidth();
                int height = SafeWindowHeight();
                Console.Clear();
                int row = 0;

                void WriteLine(string text = "")
                {
                    if (row < height)
                    {
                        Console.SetCursorPosition(0, row);
                        Console.Write(text);
                    }
                    row++;
                }

                WriteLine($" DianaOS vstop  -  pid {s.ProcessId}  -  up {RuntimeSampler.FormatUptime(s.Uptime)}  -  Ctrl+C or Q to exit, G to collect ");
                WriteLine($" {s.Framework}  on  {s.Architecture}  -  {s.ProcessorCount} cores ");
                WriteLine(new string('-', Math.Min(width, 78)));

                WriteLine($"  CPU        {ColoredBar(s.CpuPercent, 30)} {s.CpuPercent,5:0.0}%   of {s.ProcessorCount} cores");
                if (s.HasGcData)
                {
                    WriteLine($"  Machine    {ColoredBar(s.MemoryLoadPercent, 30)} {s.MemoryLoadPercent,5:0.0}%   " +
                              $"{RuntimeSampler.FormatBytes(s.MemoryLoadBytes)} / {RuntimeSampler.FormatBytes(s.TotalAvailableMemoryBytes)}");
                    WriteLine($"  Heap frag  {ColoredBar(s.FragmentationPercent, 30)} {s.FragmentationPercent,5:0.0}%   " +
                              $"{RuntimeSampler.FormatBytes(s.HeapFragmentedBytes)} dead");
                }
                else
                {
                    WriteLine($"  Machine    {EmptyBar(30)}     -   of {RuntimeSampler.FormatBytes(s.TotalAvailableMemoryBytes)} {NoGcYet}");
                    WriteLine($"  Heap frag  {EmptyBar(30)}     -   {NoGcYet}");
                }
                WriteLine($"  Pool load  {ColoredBar(s.ThreadPoolSaturationPercent, 30)} {s.ThreadPoolSaturationPercent,5:0.0}%   " +
                          $"{s.ThreadPoolThreads} of max {s.MaxWorkerThreads}");
                WriteLine();

                WriteLine("Memory:");
                WriteLine($"  Working set    {RuntimeSampler.FormatBytes(s.WorkingSetBytes),12}      Private        {RuntimeSampler.FormatBytes(s.PrivateMemoryBytes),12}");
                WriteLine($"  Managed heap   {RuntimeSampler.FormatBytes(s.ManagedHeapBytes),12}      Committed      {(s.HasGcData ? RuntimeSampler.FormatBytes(s.HeapCommittedBytes) : "-"),12}");
                WriteLine($"  Allocated      {RuntimeSampler.FormatBytes(s.TotalAllocatedBytes),12}      Alloc rate     {RuntimeSampler.FormatBytes(s.AllocatedBytesPerSecond) + "/s",12}");
                WriteLine(s.HasGcData && s.GenerationSizes.Length > 0
                    ? "  " + string.Join("   ", s.GenerationSizes.Select((size, i) => $"{GenerationName(i)} {RuntimeSampler.FormatBytes(size)}"))
                    : $"  per-generation sizes {NoGcYet}");
                WriteLine();

                WriteLine("Garbage collector:");
                WriteLine($"  Mode           {(s.IsServerGc ? "server" : "workstation"),12}      Concurrent     {(s.IsConcurrentGc ? "yes" : "no"),12}");
                WriteLine($"  Latency        {s.LatencyMode,12}      Pause time     {(s.HasGcData ? s.GcPauseTimePercent.ToString("0.00") + "%" : "-"),12}");
                WriteLine($"  Collections    gen0 {s.Gen0Collections}  gen1 {s.Gen1Collections}  gen2 {s.Gen2Collections}   " +
                          $"({s.CollectionsPerMinute:0.0}/min, {s.TotalPauseDuration.TotalMilliseconds:0} ms paused total)");
                WriteLine();

                WriteLine("Threads:");
                WriteLine($"  OS threads     {s.OsThreads,12}      Handles        {s.HandleCount,12}");
                WriteLine($"  Pool threads   {s.ThreadPoolThreads,12}      Pool min/max   {s.MinWorkerThreads + "/" + s.MaxWorkerThreads,12}");
                WriteLine($"  Queued work    {s.PendingWorkItems,12}      Completed      {s.CompletedWorkItems + " (" + s.WorkItemsPerSecond.ToString("0.0") + "/s)",12}");
                WriteLine();

                WriteLine($"Assemblies loaded: {s.AssemblyCount}");
            }

            // A runtime reporting fewer generations just drops the tail names.
            private static string GenerationName(int index) => index switch
            {
                0 or 1 or 2 => $"gen{index}",
                3 => "LOH",
                4 => "POH",
                _ => $"g{index}",
            };

            private const string NoGcYet = "(no GC yet - press G)";

            private static string EmptyBar(int width) => "[" + new string('-', width) + "]";

            // Same green/yellow/red thresholds coretop's bars use.
            private static string ColoredBar(double percent, int width)
            {
                percent = Math.Clamp(percent, 0, 100);
                int filled = (int)Math.Round(percent / 100.0 * width);
                string color = percent >= 85 ? "\x1b[31m" : percent >= 60 ? "\x1b[33m" : "\x1b[32m";
                return "[" + color + new string('#', filled) + "\x1b[0m" + new string('-', width - filled) + "]";
            }

            private static int SafeWindowWidth()
            {
                try { return Math.Max(20, Console.WindowWidth); } catch { return 80; }
            }

            private static int SafeWindowHeight()
            {
                try { return Math.Max(5, Console.WindowHeight); } catch { return 24; }
            }
        }
    }
}

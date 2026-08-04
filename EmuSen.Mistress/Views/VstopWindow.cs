using System;
using System.Collections.Generic;
using Avalonia.Controls;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // The GUI counterpart to DianaOS's `vstop` - see `man vstop`. Needs no
    // IDebugTarget at all, so unlike CoretopWindow it has no no-ROM state.
    public class VstopWindow : PollingWindow
    {
        private readonly RuntimeSampler _sampler = new();

        private readonly MonoText _header = new() { Name = "HeaderText", FontWeight = Avalonia.Media.FontWeight.Bold, FontSize = LunaPalette.HeaderFontSize };
        private readonly MonoText _host = new() { Name = "HostText", Foreground = LunaPalette.Muted, FontSize = LunaPalette.HintFontSize };
        private readonly MeterList _meters = new() { Name = "MetersPanel" };
        private readonly MonoText _memory = new() { Name = "MemoryText" };
        private readonly MonoText _gc = new() { Name = "GcText" };
        private readonly MonoText _threads = new() { Name = "ThreadsText" };
        private readonly HintText _hint = new() { Name = "HintText" };

        public VstopWindow()
        {
            Title = "DianaOS vstop";
            Width = 480;
            Height = 620;
            this.MinSize(360, 360);

            Content = Ui.Dock(
                Ui.Cols("*,Auto",
                    _hint.Center(),
                    Ui.Button("Collect now", CollectNow).Name("CollectButton")).Dock(Dock.Bottom).Margin(12, 8),
                Ui.Scroll(Ui.Stack(8,
                    _header,
                    _host,
                    _meters,
                    Ui.Section("Memory", _memory),
                    Ui.Section("Garbage collector", _gc),
                    Ui.Section("Threads", _threads)).Margin(12)));

            _sampler.Sample(); // primes the rate counters - see `man vstop`
            StartPolling();
        }

        // Same 500ms cadence the console dashboard refreshes at.
        protected override TimeSpan RefreshInterval => TimeSpan.FromMilliseconds(500);

        private void CollectNow()
        {
            GC.Collect();
            RefreshNow();
        }

        protected override void Refresh()
        {
            RuntimeSnapshot s = _sampler.Sample();

            _header.Text = $"pid {s.ProcessId}   up {RuntimeSampler.FormatUptime(s.Uptime)}";
            _host.Text = $"{s.Framework} on {s.Architecture} - {s.ProcessorCount} cores\n{s.OperatingSystem}";

            _hint.Text = s.HasGcData
                ? "Refreshing twice a second."
                : "Heap figures stay blank until the first collection - see `man vstop`.";

            var meters = new List<MeterEntry>
            {
                new("CPU", s.CpuPercent, $"{s.CpuPercent:0.0}%"),
            };

            // GCMemoryInfo describes the LAST collection, so these read zero
            // in a process that has not had one - see `man vstop`.
            if (s.HasGcData)
            {
                meters.Add(new("Machine memory", s.MemoryLoadPercent, $"{s.MemoryLoadPercent:0.0}%"));
                meters.Add(new("Heap fragmentation", s.FragmentationPercent, $"{s.FragmentationPercent:0.0}%"));
            }
            else
            {
                meters.Add(new("Machine memory", 0, "-"));
                meters.Add(new("Heap fragmentation", 0, "-"));
            }

            meters.Add(new("Thread pool", s.ThreadPoolSaturationPercent, $"{s.ThreadPoolSaturationPercent:0.0}%"));
            _meters.Meters = meters;

            string committed = s.HasGcData ? RuntimeSampler.FormatBytes(s.HeapCommittedBytes) : "-";
            string generations = s.HasGcData && s.GenerationSizes.Length > 0
                ? string.Join("   ", Enumerate(s.GenerationSizes))
                : "(no collection yet)";

            _memory.Text =
                $"Working set   {RuntimeSampler.FormatBytes(s.WorkingSetBytes),12}\n" +
                $"Private       {RuntimeSampler.FormatBytes(s.PrivateMemoryBytes),12}\n" +
                $"Managed heap  {RuntimeSampler.FormatBytes(s.ManagedHeapBytes),12}\n" +
                $"Committed     {committed,12}\n" +
                $"Allocated     {RuntimeSampler.FormatBytes(s.TotalAllocatedBytes),12}\n" +
                $"Alloc rate    {RuntimeSampler.FormatBytes(s.AllocatedBytesPerSecond) + "/s",12}\n" +
                generations;

            _gc.Text =
                $"Mode          {(s.IsServerGc ? "server" : "workstation"),12}\n" +
                $"Concurrent    {(s.IsConcurrentGc ? "yes" : "no"),12}\n" +
                $"Latency       {s.LatencyMode,12}\n" +
                $"Pause time    {(s.HasGcData ? s.GcPauseTimePercent.ToString("0.00") + "%" : "-"),12}\n" +
                $"Collections   gen0 {s.Gen0Collections}  gen1 {s.Gen1Collections}  gen2 {s.Gen2Collections}\n" +
                $"              {s.CollectionsPerMinute:0.0}/min, {s.TotalPauseDuration.TotalMilliseconds:0} ms paused total";

            _threads.Text =
                $"OS threads    {s.OsThreads,12}\n" +
                $"Handles       {s.HandleCount,12}\n" +
                $"Pool threads  {s.ThreadPoolThreads,12}\n" +
                $"Pool min/max  {s.MinWorkerThreads + "/" + s.MaxWorkerThreads,12}\n" +
                $"Queued work   {s.PendingWorkItems,12}\n" +
                $"Completed     {s.CompletedWorkItems,12}  ({s.WorkItemsPerSecond:0.0}/s)\n" +
                $"Assemblies    {s.AssemblyCount,12}";
        }

        // gen0/1/2 then LOH then POH, matching the console dashboard's own naming.
        private static IEnumerable<string> Enumerate(long[] sizes)
        {
            for (int i = 0; i < sizes.Length; i++)
            {
                string name = i switch { 0 or 1 or 2 => $"gen{i}", 3 => "LOH", 4 => "POH", _ => $"g{i}" };
                yield return $"{name} {RuntimeSampler.FormatBytes(sizes[i])}";
            }
        }
    }
}

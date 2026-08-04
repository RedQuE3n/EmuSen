using System;
using System.Collections.Generic;
using EmuSen.LunaP.Theme;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Mistress.Views
{
    // The GUI counterpart to DianaOS's `vstop` - see `man vstop`. Needs no
    // IDebugTarget at all, so unlike CoretopWindow it has no no-ROM state.
    public partial class VstopWindow : Window
    {
        private readonly RuntimeSampler _sampler = new();
        private readonly DispatcherTimer _timer;

        public VstopWindow()
        {
            InitializeComponent();

            // Same 500ms cadence the console dashboard refreshes at.
            _sampler.Sample(); // primes the rate counters - see `man vstop`
            _timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(500) };
            _timer.Tick += (_, _) => Refresh();
            _timer.Start();
            Closed += (_, _) => _timer.Stop();

            Refresh();
        }

        private void OnCollectClick(object? sender, RoutedEventArgs e)
        {
            GC.Collect();
            Refresh();
        }

        private void Refresh()
        {
            RuntimeSnapshot s = _sampler.Sample();

            HeaderText.Text = $"pid {s.ProcessId}   up {RuntimeSampler.FormatUptime(s.Uptime)}";
            HostText.Text = $"{s.Framework} on {s.Architecture} - {s.ProcessorCount} cores\n{s.OperatingSystem}";

            HintText.Text = s.HasGcData
                ? "Refreshing twice a second."
                : "Heap figures stay blank until the first collection - see `man vstop`.";

            var meters = new List<Control>
            {
                BuildMeterRow("CPU", s.CpuPercent, $"{s.CpuPercent:0.0}%"),
            };

            // GCMemoryInfo describes the LAST collection, so these read zero
            // in a process that has not had one - see `man vstop`.
            if (s.HasGcData)
            {
                meters.Add(BuildMeterRow("Machine memory", s.MemoryLoadPercent,
                    $"{s.MemoryLoadPercent:0.0}%"));
                meters.Add(BuildMeterRow("Heap fragmentation", s.FragmentationPercent,
                    $"{s.FragmentationPercent:0.0}%"));
            }
            else
            {
                meters.Add(BuildMeterRow("Machine memory", 0, "-"));
                meters.Add(BuildMeterRow("Heap fragmentation", 0, "-"));
            }

            meters.Add(BuildMeterRow("Thread pool", s.ThreadPoolSaturationPercent,
                $"{s.ThreadPoolSaturationPercent:0.0}%"));

            MetersPanel.Children.Clear();
            foreach (Control meter in meters) MetersPanel.Children.Add(meter);

            string committed = s.HasGcData ? RuntimeSampler.FormatBytes(s.HeapCommittedBytes) : "-";
            string generations = s.HasGcData && s.GenerationSizes.Length > 0
                ? string.Join("   ", Enumerate(s.GenerationSizes))
                : "(no collection yet)";

            MemoryText.Text =
                $"Working set   {RuntimeSampler.FormatBytes(s.WorkingSetBytes),12}\n" +
                $"Private       {RuntimeSampler.FormatBytes(s.PrivateMemoryBytes),12}\n" +
                $"Managed heap  {RuntimeSampler.FormatBytes(s.ManagedHeapBytes),12}\n" +
                $"Committed     {committed,12}\n" +
                $"Allocated     {RuntimeSampler.FormatBytes(s.TotalAllocatedBytes),12}\n" +
                $"Alloc rate    {RuntimeSampler.FormatBytes(s.AllocatedBytesPerSecond) + "/s",12}\n" +
                generations;

            GcText.Text =
                $"Mode          {(s.IsServerGc ? "server" : "workstation"),12}\n" +
                $"Concurrent    {(s.IsConcurrentGc ? "yes" : "no"),12}\n" +
                $"Latency       {s.LatencyMode,12}\n" +
                $"Pause time    {(s.HasGcData ? s.GcPauseTimePercent.ToString("0.00") + "%" : "-"),12}\n" +
                $"Collections   gen0 {s.Gen0Collections}  gen1 {s.Gen1Collections}  gen2 {s.Gen2Collections}\n" +
                $"              {s.CollectionsPerMinute:0.0}/min, {s.TotalPauseDuration.TotalMilliseconds:0} ms paused total";

            ThreadsText.Text =
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

        // Same shape and thresholds as CoretopWindow's own meter rows.
        private static Control BuildMeterRow(string label, double percent, string valueText)
        {
            var grid = new Grid { ColumnDefinitions = new ColumnDefinitions("140,*,55") };

            var labelText = new TextBlock { Text = label, Foreground = LunaPalette.MeterText, VerticalAlignment = VerticalAlignment.Center, TextTrimming = TextTrimming.CharacterEllipsis };
            var bar = new ProgressBar { Minimum = 0, Maximum = 100, Value = Math.Clamp(percent, 0, 100), Height = 14, Foreground = LunaPalette.ForLoad(percent) };
            var valueTextBlock = new TextBlock { Text = valueText, Foreground = LunaPalette.MeterText, Margin = new Thickness(8, 0, 0, 0), VerticalAlignment = VerticalAlignment.Center };

            Grid.SetColumn(labelText, 0);
            Grid.SetColumn(bar, 1);
            Grid.SetColumn(valueTextBlock, 2);
            grid.Children.Add(labelText);
            grid.Children.Add(bar);
            grid.Children.Add(valueTextBlock);
            return grid;
        }
    }
}

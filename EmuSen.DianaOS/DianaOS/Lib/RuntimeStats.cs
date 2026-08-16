using System;
using System.Diagnostics;
using System.Runtime;
using System.Runtime.InteropServices;
using System.Threading;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // One reading of the .NET VM this process runs on - see `man vstop`.
    public readonly struct RuntimeSnapshot
    {
        // Host identity, fixed for the process.
        public string Framework { get; init; }
        public string OperatingSystem { get; init; }
        public string Architecture { get; init; }
        public int ProcessorCount { get; init; }
        public int ProcessId { get; init; }
        public TimeSpan Uptime { get; init; }

        // Rates, zero on the first sample - see RuntimeSampler.
        public double CpuPercent { get; init; }
        public double AllocatedBytesPerSecond { get; init; }
        public double WorkItemsPerSecond { get; init; }
        public double CollectionsPerMinute { get; init; }

        // Process memory, as the OS accounts for it.
        public long WorkingSetBytes { get; init; }
        public long PrivateMemoryBytes { get; init; }

        // Managed heap, as the GC accounts for it.
        public long ManagedHeapBytes { get; init; }
        public long HeapCommittedBytes { get; init; }
        public long HeapFragmentedBytes { get; init; }
        public long TotalAllocatedBytes { get; init; }

        // Machine-wide pressure the GC is reacting to.
        public long MemoryLoadBytes { get; init; }
        public long TotalAvailableMemoryBytes { get; init; }
        public long HighMemoryLoadThresholdBytes { get; init; }

        public int Gen0Collections { get; init; }
        public int Gen1Collections { get; init; }
        public int Gen2Collections { get; init; }
        public double GcPauseTimePercent { get; init; }
        public TimeSpan TotalPauseDuration { get; init; }
        public bool IsServerGc { get; init; }
        public bool IsConcurrentGc { get; init; }
        public string LatencyMode { get; init; }

        // Per-generation sizes, index 0-2 then LOH then POH where reported.
        public long[] GenerationSizes { get; init; }

        public int ThreadPoolThreads { get; init; }
        public long PendingWorkItems { get; init; }
        public long CompletedWorkItems { get; init; }
        public int MinWorkerThreads { get; init; }
        public int MaxWorkerThreads { get; init; }
        public int OsThreads { get; init; }
        public int HandleCount { get; init; }
        public int AssemblyCount { get; init; }

        // GCMemoryInfo describes the last collection, so it is zero until one happens - see `man vstop`.
        public bool HasGcData => Gen0Collections + Gen1Collections + Gen2Collections > 0;

        // Percent of the managed heap that is dead space the GC has not given back.
        public double FragmentationPercent =>
            HeapCommittedBytes > 0 ? HeapFragmentedBytes * 100.0 / HeapCommittedBytes : 0;

        // Machine memory in use, the figure the GC's own load threshold is compared against.
        public double MemoryLoadPercent =>
            TotalAvailableMemoryBytes > 0 ? MemoryLoadBytes * 100.0 / TotalAvailableMemoryBytes : 0;

        public double ThreadPoolSaturationPercent =>
            MaxWorkerThreads > 0 ? Math.Min(100.0, ThreadPoolThreads * 100.0 / MaxWorkerThreads) : 0;
    }

    // A rate needs two readings, so the previous one is held here - see `man vstop`.
    public sealed class RuntimeSampler
    {
        private readonly Process _process = Process.GetCurrentProcess();
        private readonly Stopwatch _clock = Stopwatch.StartNew();

        private bool _primed;
        private TimeSpan _lastElapsed;
        private TimeSpan _lastCpuTime;
        private long _lastAllocated;
        private long _lastCompletedWorkItems;
        private int _lastTotalCollections;

        // Nothing here may throw: a dashboard that kills the shell is worse than a zero.
        public RuntimeSnapshot Sample()
        {
            TimeSpan elapsed = _clock.Elapsed;
            double seconds = (elapsed - _lastElapsed).TotalSeconds;

            Try(() => _process.Refresh());

            TimeSpan cpuTime = Get(() => _process.TotalProcessorTime, TimeSpan.Zero);
            long allocated = Get(() => GC.GetTotalAllocatedBytes(), 0L);
            long completedWorkItems = Get(() => ThreadPool.CompletedWorkItemCount, 0L);

            int gen0 = Get(() => GC.CollectionCount(0), 0);
            int gen1 = Get(() => GC.CollectionCount(1), 0);
            int gen2 = Get(() => GC.CollectionCount(2), 0);
            int totalCollections = gen0 + gen1 + gen2;

            GCMemoryInfo memoryInfo = Get(() => GC.GetGCMemoryInfo(), default);

            int minWorkers = 0, maxWorkers = 0;
            Try(() => { ThreadPool.GetMinThreads(out minWorkers, out _); });
            Try(() => { ThreadPool.GetMaxThreads(out maxWorkers, out _); });

            var snapshot = new RuntimeSnapshot
            {
                Framework = Get(() => RuntimeInformation.FrameworkDescription, ".NET"),
                OperatingSystem = Get(() => RuntimeInformation.OSDescription, "unknown"),
                Architecture = Get(() => RuntimeInformation.ProcessArchitecture.ToString(), "unknown"),
                ProcessorCount = Get(() => Environment.ProcessorCount, 1),
                ProcessId = Get(() => Environment.ProcessId, 0),
                Uptime = Get(() => DateTime.Now - _process.StartTime, TimeSpan.Zero),

                // Divided by core count, so 100% means every core busy - htop's aggregate scale.
                CpuPercent = _primed && seconds > 0
                    ? Math.Clamp((cpuTime - _lastCpuTime).TotalSeconds / seconds * 100.0 / Math.Max(1, Environment.ProcessorCount), 0, 100)
                    : 0,
                AllocatedBytesPerSecond = _primed && seconds > 0 ? Math.Max(0, allocated - _lastAllocated) / seconds : 0,
                WorkItemsPerSecond = _primed && seconds > 0 ? Math.Max(0, completedWorkItems - _lastCompletedWorkItems) / seconds : 0,
                CollectionsPerMinute = _primed && seconds > 0 ? Math.Max(0, totalCollections - _lastTotalCollections) / seconds * 60.0 : 0,

                WorkingSetBytes = Get(() => _process.WorkingSet64, 0L),
                PrivateMemoryBytes = Get(() => _process.PrivateMemorySize64, 0L),

                ManagedHeapBytes = Get(() => GC.GetTotalMemory(forceFullCollection: false), 0L),
                HeapCommittedBytes = memoryInfo.TotalCommittedBytes,
                HeapFragmentedBytes = memoryInfo.FragmentedBytes,
                TotalAllocatedBytes = allocated,

                MemoryLoadBytes = memoryInfo.MemoryLoadBytes,
                TotalAvailableMemoryBytes = memoryInfo.TotalAvailableMemoryBytes,
                HighMemoryLoadThresholdBytes = memoryInfo.HighMemoryLoadThresholdBytes,

                Gen0Collections = gen0,
                Gen1Collections = gen1,
                Gen2Collections = gen2,
                GcPauseTimePercent = memoryInfo.PauseTimePercentage,
                TotalPauseDuration = Get(GC.GetTotalPauseDuration, TimeSpan.Zero),
                IsServerGc = Get(() => GCSettings.IsServerGC, false),
                IsConcurrentGc = memoryInfo.Concurrent,
                LatencyMode = Get(() => GCSettings.LatencyMode.ToString(), "Unknown"),
                GenerationSizes = ReadGenerationSizes(memoryInfo),

                ThreadPoolThreads = Get(() => ThreadPool.ThreadCount, 0),
                PendingWorkItems = Get(() => ThreadPool.PendingWorkItemCount, 0L),
                CompletedWorkItems = completedWorkItems,
                MinWorkerThreads = minWorkers,
                MaxWorkerThreads = maxWorkers,
                OsThreads = Get(() => _process.Threads.Count, 0),
                HandleCount = Get(() => _process.HandleCount, 0),
                AssemblyCount = Get(() => AppDomain.CurrentDomain.GetAssemblies().Length, 0),
            };

            _lastElapsed = elapsed;
            _lastCpuTime = cpuTime;
            _lastAllocated = allocated;
            _lastCompletedWorkItems = completedWorkItems;
            _lastTotalCollections = totalCollections;
            _primed = true;

            return snapshot;
        }

        // GenerationInfo is a span, so it cannot be stored - copied out here.
        private static long[] ReadGenerationSizes(GCMemoryInfo info)
        {
            try
            {
                var sizes = new long[info.GenerationInfo.Length];
                for (int i = 0; i < sizes.Length; i++) sizes[i] = info.GenerationInfo[i].SizeAfterBytes;
                return sizes;
            }
            catch
            {
                return Array.Empty<long>();
            }
        }

        private static T Get<T>(Func<T> read, T fallback)
        {
            try { return read(); } catch { return fallback; }
        }

        private static void Try(Action run)
        {
            try { run(); } catch { }
        }

        // 1.2 GB / 340 MB / 12 KB - the width htop gives a memory column.
        public static string FormatBytes(double bytes)
        {
            string[] units = { "B", "KB", "MB", "GB", "TB" };
            int unit = 0;
            double value = Math.Abs(bytes);
            while (value >= 1024 && unit < units.Length - 1)
            {
                value /= 1024;
                unit++;
            }
            if (bytes < 0) value = -value;
            return unit == 0 ? $"{value:0} {units[unit]}" : $"{value:0.0} {units[unit]}";
        }

        public static string FormatUptime(TimeSpan uptime) =>
            uptime.TotalDays >= 1
                ? $"{(int)uptime.TotalDays}d {uptime.Hours:00}:{uptime.Minutes:00}:{uptime.Seconds:00}"
                : $"{(int)uptime.TotalHours:00}:{uptime.Minutes:00}:{uptime.Seconds:00}";
    }
}

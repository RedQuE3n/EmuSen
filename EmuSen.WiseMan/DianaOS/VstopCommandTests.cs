using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.DianaOS.DianaOS.Bin.Commands.Unix;

namespace EmuSen.WiseMan.DianaOS
{
    // `vstop` - the .NET VM dashboard. The drawing itself needs a real
    // terminal and isn't covered here, same as `coretop`'s isn't; what is
    // covered is every number it draws - see `man vstop`.
    public class VstopCommandTests
    {
        // --- The sampler ---

        [Fact]
        public void A_sample_reports_the_runtime_it_is_actually_running_on()
        {
            RuntimeSnapshot s = new RuntimeSampler().Sample();

            Assert.Contains(".NET", s.Framework);
            Assert.True(s.ProcessorCount >= 1);
            Assert.Equal(Environment.ProcessId, s.ProcessId);
            Assert.True(s.Uptime > TimeSpan.Zero);
            Assert.False(string.IsNullOrWhiteSpace(s.Architecture));
        }

        [Fact]
        public void A_sample_reports_a_live_managed_heap_and_process_memory()
        {
            RuntimeSnapshot s = new RuntimeSampler().Sample();

            Assert.True(s.ManagedHeapBytes > 0, "the managed heap cannot be empty inside a running test");
            Assert.True(s.WorkingSetBytes > 0);
            Assert.True(s.TotalAllocatedBytes > 0);
            Assert.True(s.AssemblyCount > 0);
            Assert.True(s.OsThreads > 0);
        }

        // Rates need two readings, so the first can only ever be zero.
        [Fact]
        public void Rates_are_zero_on_the_first_sample_and_real_on_the_second()
        {
            var sampler = new RuntimeSampler();

            RuntimeSnapshot first = sampler.Sample();
            Assert.Equal(0, first.CpuPercent);
            Assert.Equal(0, first.AllocatedBytesPerSecond);

            // Allocate enough that the rate cannot round to nothing.
            var ballast = new List<byte[]>();
            for (int i = 0; i < 400; i++) ballast.Add(new byte[64 * 1024]);
            Thread.Sleep(60);

            RuntimeSnapshot second = sampler.Sample();

            Assert.True(second.AllocatedBytesPerSecond > 0,
                $"allocated {second.TotalAllocatedBytes - first.TotalAllocatedBytes} bytes but the rate read 0");
            Assert.True(second.TotalAllocatedBytes > first.TotalAllocatedBytes);
            GC.KeepAlive(ballast);
        }

        [Fact]
        public void Collection_counts_only_ever_climb()
        {
            var sampler = new RuntimeSampler();
            RuntimeSnapshot before = sampler.Sample();

            GC.Collect();
            RuntimeSnapshot after = sampler.Sample();

            Assert.True(after.Gen0Collections > before.Gen0Collections);
            Assert.True(after.Gen2Collections >= before.Gen2Collections);
            Assert.True(after.CollectionsPerMinute > 0);
        }

        // CPU is divided by core count - htop's aggregate scale, not the
        // per-core one an 8-core box would read 800% on.
        [Fact]
        public void Cpu_percent_stays_inside_zero_to_one_hundred_under_load()
        {
            var sampler = new RuntimeSampler();
            sampler.Sample();

            var spin = new System.Diagnostics.Stopwatch();
            spin.Start();
            while (spin.ElapsedMilliseconds < 80) { }

            RuntimeSnapshot s = sampler.Sample();

            Assert.InRange(s.CpuPercent, 0, 100);
            Assert.InRange(s.MemoryLoadPercent, 0, 100);
            Assert.InRange(s.FragmentationPercent, 0, 100);
            Assert.InRange(s.ThreadPoolSaturationPercent, 0, 100);
        }

        // GCMemoryInfo describes the last collection, so it reads all zeroes
        // in a process that has not had one - the dashboard must say so
        // rather than draw a truthful-looking 0%.
        [Fact]
        public void A_snapshot_knows_whether_any_gc_has_happened_yet()
        {
            Assert.False(new RuntimeSnapshot().HasGcData);
            Assert.False(new RuntimeSnapshot { Gen0Collections = 0 }.HasGcData);
            Assert.True(new RuntimeSnapshot { Gen0Collections = 1 }.HasGcData);
            Assert.True(new RuntimeSnapshot { Gen2Collections = 3 }.HasGcData);

            GC.Collect();
            Assert.True(new RuntimeSampler().Sample().HasGcData);
        }

        [Fact]
        public void Gc_derived_figures_are_populated_once_a_collection_has_run()
        {
            GC.Collect();
            RuntimeSnapshot s = new RuntimeSampler().Sample();

            Assert.True(s.HasGcData);
            Assert.True(s.HeapCommittedBytes > 0, "committed bytes should be real after a collection");
            Assert.True(s.TotalAvailableMemoryBytes > 0);
            Assert.NotEmpty(s.GenerationSizes);
        }

        [Fact]
        public void Derived_percentages_do_not_divide_by_zero_on_an_empty_snapshot()
        {
            var empty = new RuntimeSnapshot();

            Assert.Equal(0, empty.FragmentationPercent);
            Assert.Equal(0, empty.MemoryLoadPercent);
            Assert.Equal(0, empty.ThreadPoolSaturationPercent);
        }

        [Theory]
        [InlineData(0, "0 B")]
        [InlineData(512, "512 B")]
        [InlineData(1024, "1.0 KB")]
        [InlineData(1536, "1.5 KB")]
        [InlineData(1048576, "1.0 MB")]
        [InlineData(1073741824, "1.0 GB")]
        public void Bytes_format_the_way_a_memory_column_should(double bytes, string expected) =>
            Assert.Equal(expected, RuntimeSampler.FormatBytes(bytes));

        [Fact]
        public void Uptime_grows_a_day_field_only_once_there_is_a_day_to_show()
        {
            Assert.Equal("02:03:04", RuntimeSampler.FormatUptime(new TimeSpan(0, 2, 3, 4)));
            Assert.Equal("3d 02:03:04", RuntimeSampler.FormatUptime(new TimeSpan(3, 2, 3, 4)));
        }

        // --- The command ---

        [Fact]
        public void Vstop_is_registered_and_needs_no_rom_loaded()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Contains("vstop", shell.CommandNames);

            // Everything else core-facing answers "No ROM loaded" here.
            var result = shell.Submit("vstop -w");
            Assert.DoesNotContain("No ROM loaded", result.Output);
        }

        [Fact]
        public void Minus_w_without_a_window_opener_reports_not_supported()
        {
            var shell = DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new VstopCommand() });

            var result = shell.Submit("vstop -w");

            Assert.Contains("not supported by this frontend", result.Output);
        }

        [Fact]
        public void Minus_w_with_a_window_opener_calls_it_instead_of_the_console_dashboard()
        {
            int opened = 0;
            var shell = DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new VstopCommand(() => opened++) });

            var result = shell.Submit("vstop -W");

            Assert.Equal(1, opened);
            Assert.Contains("opened in a separate window", result.Output);
        }

        // The test runner redirects stdout, which is exactly the condition
        // the guard exists for - so this asserts the refusal, not a drawn frame.
        [Fact]
        public void Without_a_real_terminal_it_refuses_rather_than_drawing()
        {
            var shell = DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new VstopCommand() });

            var result = shell.Submit("vstop");

            Assert.Contains("needs a real interactive terminal", result.Output);
        }

        [Fact]
        public void Vstop_blocks_the_terminal_so_it_is_not_fast_path_eligible() =>
            Assert.False(new VstopCommand().IsReadOnly);
    }
}

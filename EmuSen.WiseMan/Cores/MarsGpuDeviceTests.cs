using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars.Rdp.Gpu;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The compute device under the multiple's shading, and the shaders it is given - see Mars_Gpu.md §1 to §3.
    public class MarsGpuDeviceTests
    {
        private readonly ITestOutputHelper _output;

        public MarsGpuDeviceTests(ITestOutputHelper output) => _output = output;

        private struct AddPush { public uint Count; public int Bias; }

        // Every device the machine offers, so the software one is exercised beside the real one; none at all is a machine the CPU path serves.
        public static TheoryData<string> Devices()
        {
            var data = new TheoryData<string>();
            foreach (string name in GpuDevice.DeviceNames().Distinct()) data.Add(name);
            if (data.Count == 0) data.Add("");
            return data;
        }

        [Theory]
        [MemberData(nameof(Devices))]
        public void A_shader_adds_two_buffers_in_integers_that_wrap(string deviceName)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: nothing to run, and that is the CPU path's machine"); return; }

            var clock = Stopwatch.StartNew();
            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            double createMs = clock.Elapsed.TotalMilliseconds;

            const int count = 100_003;
            using GpuBuffer a = device.CreateBuffer(count * 4);
            using GpuBuffer b = device.CreateBuffer(count * 4);
            using GpuBuffer onDevice = device.CreateBuffer(count * 4, GpuMemory.Device);
            using GpuBuffer c = device.CreateBuffer(count * 4);

            var random = new Random(64);
            Span<int> left = a.Span<int>(), right = b.Span<int>();
            for (int i = 0; i < count; i++) { left[i] = random.Next(int.MinValue, int.MaxValue); right[i] = random.Next(int.MinValue, int.MaxValue); }
            c.Span<int>().Fill(-1);

            clock.Restart();
            using GpuProgram add = device.CreateProgram(GpuShaders.Load("add"), buffers: 3, pushBytes: 8);
            double programMs = clock.Elapsed.TotalMilliseconds;

            // Through device memory and back, which is the path a frame image will take.
            clock.Restart();
            device.Submit(commands =>
            {
                commands.Dispatch(add, new[] { a, b, onDevice }, new AddPush { Count = count, Bias = 7 }, (count + 63) / 64);
                commands.Copy(onDevice, c);
            });
            double runMs = clock.Elapsed.TotalMilliseconds;

            Span<int> sum = c.Span<int>();
            for (int i = 0; i < count; i++)
                if (sum[i] != unchecked(left[i] + right[i] + 7)) Assert.Fail($"element {i} on {device.Name}: {sum[i]}, not {unchecked(left[i] + right[i] + 7)}");

            _output.WriteLine($"{report}: device {createMs:F1} ms, pipeline {programMs:F1} ms, dispatch and readback of {count} words {runMs:F2} ms");
        }

        [Fact]
        public void A_name_no_device_has_is_refused_with_a_reason_and_nothing_thrown()
        {
            Assert.Null(GpuDevice.TryCreate("no device is called this", out string report));
            Assert.False(string.IsNullOrWhiteSpace(report));
        }

        [Theory]
        [MemberData(nameof(Devices))]
        public void A_dispatch_given_the_wrong_buffers_or_constants_is_refused_before_the_device_sees_it(string deviceName)
        {
            if (deviceName.Length == 0) return;

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out _)!;
            using GpuBuffer only = device.CreateBuffer(64);
            using GpuProgram add = device.CreateProgram(GpuShaders.Load("add"), buffers: 3, pushBytes: 8);

            Assert.Throws<ArgumentException>(() => device.Submit(c => c.Dispatch(add, new[] { only }, new AddPush(), 1)));
            Assert.Throws<ArgumentException>(() => device.Submit(c => c.Dispatch(add, new[] { only, only, only }, (byte)0, 1)));

            // A refusal leaves the device usable: the next submission is recorded from a reset buffer.
            only.Span<int>().Fill(3);
            device.Submit(c => c.Dispatch(add, new[] { only, only, only }, new AddPush { Count = 16, Bias = 1 }, 1));
            Assert.All(only.Span<int>().ToArray(), word => Assert.Equal(7, word));
        }

        // The product embeds the .spv; this is what keeps it the compilation of the .comp beside it - see Mars_Gpu.md §2.
        [Fact]
        public void Every_compiled_shader_is_the_compilation_of_its_source()
        {
            string[] sources = Directory.GetFiles(ShaderCompiler.ShaderDirectory, "*.comp");
            Assert.NotEmpty(sources);

            foreach (string source in sources)
            {
                string name = Path.GetFileNameWithoutExtension(source);
                byte[] compiled = ShaderCompiler.Compile(File.ReadAllText(source), name);
                string binary = Path.ChangeExtension(source, ".spv");

                if (Environment.GetEnvironmentVariable(ShaderCompiler.RecordVariable) == "1") File.WriteAllBytes(binary, compiled);

                Assert.True(File.Exists(binary), $"{name}.spv is missing: run once with {ShaderCompiler.RecordVariable}=1");
                Assert.True(compiled.AsSpan().SequenceEqual(File.ReadAllBytes(binary)), $"{name}.spv is stale: run once with {ShaderCompiler.RecordVariable}=1, then rebuild");
                Assert.True(compiled.AsSpan().SequenceEqual(GpuShaders.Load(name)), $"the embedded {name} is stale: rebuild EmuSen");
            }
        }
    }
}

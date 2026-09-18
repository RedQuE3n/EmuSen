using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Every field a Mars state walks, filled with noise, compared after a fresh core loads it - see Mars_SaveStates.md §4.
    public class MarsSaveStateTests : IDisposable
    {
        private static readonly byte[] SpinForever = { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 };

        private readonly string _rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, SpinForever)));

        public void Dispose()
        {
            try { File.Delete(_rom); } catch (IOException) { }
        }

        [Fact]
        public void Every_field_the_state_walks_comes_back_in_a_fresh_core()
        {
            var saver = new MarsCore(batteryRamDisabled: true);
            saver.LoadRom(_rom);
            var noise = new Random(1996);
            Fill(saver.Cpu!, noise, new HashSet<object>(ReferenceEqualityComparer.Instance));
            Fill(saver.Bus!, noise, new HashSet<object>(ReferenceEqualityComparer.Instance));

            using var state = new MemoryStream();
            saver.SaveState(state);
            state.Position = 0;

            var loader = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            loader.LoadRom(_rom);
            loader.LoadState(state);

            var differences = new List<string>();
            Compare("Cpu", saver.Cpu!, loader.Cpu!, differences);
            Compare("Bus", saver.Bus!, loader.Bus!, differences);

            Assert.Empty(differences);
        }

        // The TLB is the one table no game here uses, so nothing but this test would miss it - see Mars_SaveStates.md §4.
        [Fact]
        public void The_tlb_comes_back()
        {
            var saver = new MarsCore(batteryRamDisabled: true);
            saver.LoadRom(_rom);
            Fill(saver.Cpu!.Tlb, new Random(64), new HashSet<object>(ReferenceEqualityComparer.Instance));

            using var state = new MemoryStream();
            saver.SaveState(state);
            state.Position = 0;

            var loader = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            loader.LoadRom(_rom);
            loader.LoadState(state);

            Assert.Equal(saver.Cpu.Tlb.Entries, loader.Cpu!.Tlb.Entries);
        }

        private static IEnumerable<FieldInfo> StateFields(Type type) => type
            .GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)
            .Where(f => f.GetCustomAttribute<SkipInStateAttribute>() == null)
            .OrderBy(f => f.Name, StringComparer.Ordinal);

        private static void Fill(object target, Random noise, HashSet<object> seen)
        {
            if (!target.GetType().IsValueType && !seen.Add(target)) return;

            foreach (FieldInfo field in StateFields(target.GetType()))
            {
                Type t = field.FieldType;
                object? value = field.GetValue(target);

                if (value is byte[] bytes)
                {
                    noise.NextBytes(bytes);
                }
                else if (t.IsArray && value is Array array)
                {
                    Type element = t.GetElementType()!;
                    for (int i = 0; i < array.Length; i++)
                    {
                        if (Random(element, noise) is { } scalar) array.SetValue(scalar, i);
                        else if (array.GetValue(i) is { } item)
                        {
                            Fill(item, noise, seen);
                            if (element.IsValueType) array.SetValue(item, i);
                        }
                    }
                }
                else if (Random(t, noise) is { } scalar) field.SetValue(target, scalar);
                else if (value != null)
                {
                    Fill(value, noise, seen);
                    if (t.IsValueType) field.SetValue(target, value);
                }
            }
        }

        private static object? Random(Type t, Random noise)
        {
            if (t == typeof(bool)) return noise.Next(2) == 1;
            if (t == typeof(byte)) return (byte)noise.Next(256);
            if (t == typeof(sbyte)) return (sbyte)noise.Next(256);
            if (t == typeof(short)) return (short)noise.Next();
            if (t == typeof(ushort)) return (ushort)noise.Next();
            if (t == typeof(int)) return noise.Next();
            if (t == typeof(uint)) return (uint)noise.Next();
            if (t == typeof(long)) return noise.NextInt64();
            if (t == typeof(ulong)) return (ulong)noise.NextInt64();
            if (t == typeof(float)) return (float)noise.NextDouble();
            if (t == typeof(char)) return (char)noise.Next(0x20, 0x7F);
            if (t.IsEnum) return Enum.ToObject(t, noise.Next(4));
            return null;
        }

        private static void Compare(string path, object expected, object actual, List<string> differences)
        {
            foreach (FieldInfo field in StateFields(expected.GetType()))
            {
                object? a = field.GetValue(expected), b = field.GetValue(actual);
                string at = $"{path}.{field.Name}";

                if (a is byte[] leftBytes && b is byte[] rightBytes)
                {
                    if (!leftBytes.AsSpan().SequenceEqual(rightBytes)) differences.Add(at);
                }
                else if (a is Array left && b is Array right)
                {
                    for (int i = 0; i < left.Length; i++)
                    {
                        object? x = left.GetValue(i), y = right.GetValue(i);
                        if (x is null || y is null || Random(x.GetType(), new Random(0)) != null) { if (!Equals(x, y)) differences.Add($"{at}[{i}]"); }
                        else Compare($"{at}[{i}]", x, y, differences);
                        if (differences.Count > 20) return;
                    }
                }
                else if (a is null || b is null || Random(field.FieldType, new Random(0)) != null || field.FieldType == typeof(string))
                {
                    if (!Equals(a, b)) differences.Add(at);
                }
                else Compare(at, a, b, differences);
            }
        }
    }
}

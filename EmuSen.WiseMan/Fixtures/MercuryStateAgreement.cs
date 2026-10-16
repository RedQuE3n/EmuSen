using System;
using System.Collections.Concurrent;
using System.Reflection;
using System.Threading;
using EmuSen.Cores.Nintendo.Mercury;

namespace EmuSen.WiseMan.Fixtures
{
    // The layout listing's reading of a state held to C# Mercury's, field by field, while both are on main - see Mercury_Native.md §14.5.
    public static class MercuryStateAgreement
    {
        private static long _states;

        // States both readers have read alike since the process started.
        public static long States => Interlocked.Read(ref _states);

        private static readonly ConcurrentDictionary<(Type, string), MemberInfo?> Members = new();

        public static void Check(MercuryCore view, ReadOnlySpan<byte> state, MercuryStateLayout layout)
        {
            Assert.True(state.Length == layout.Length, $"{layout.Name} lists {layout.Length} bytes, the state has {state.Length}");
            foreach (MercuryStateLayout.Field f in layout.Fields)
            {
                object? csharp = Resolve(view, f.Path);
                if (f.Type.StartsWith("u8[", StringComparison.Ordinal))
                {
                    var bytes = (byte[])csharp!;
                    int same = bytes.AsSpan().CommonPrefixLength(state.Slice(f.Offset, f.Length));
                    Assert.True(same == f.Length && bytes.Length == f.Length, $"{layout.Name}, {f.Path}: C# reads {bytes.Length} bytes, the listing {f.Length}; first difference at {same}");
                    continue;
                }
                long listed = MercuryStateLayout.Get(state, f);
                long held = csharp switch
                {
                    null => 0,
                    bool b => b ? 1 : 0,
                    byte b => b,
                    ushort u => u,
                    int i => i,
                    uint u => u,
                    long l => l,
                    double d => BitConverter.DoubleToInt64Bits(d),
                    Enum e => Convert.ToInt64(e),
                    _ when f.Type == "class" => 1,
                    _ => throw new InvalidOperationException($"{f.Path}: a {csharp.GetType().Name} the agreement does not compare"),
                };
                if (f.Type == "bool") listed = listed != 0 ? 1 : 0;
                Assert.True(listed == held, $"{layout.Name}, {f.Path}: the listing reads {listed}, C# Mercury holds {held}");
            }
            Interlocked.Increment(ref _states);
        }

        // A listing path walked from the core: the header's fields, then member by member through Cart, Mapper, Cpu and Bus.
        private static object? Resolve(MercuryCore core, string path)
        {
            switch (path)
            {
                case "Magic": return MercuryCore.StateMagic;
                case "Version": return MercuryCore.StateVersion;
                case "Console": return core.Bus!.CgbHardware;
                case "TotalFrames": return core.TotalFrames;
            }
            string[] parts = path.Split('.');
            object? at = parts[0] switch
            {
                "Cart" => core.Cart,
                "Mapper" => core.Cart!.Mapper,
                "Cpu" => core.Cpu,
                "Bus" => core.Bus,
                _ => Member(core, parts[0]),
            };
            for (int i = 1; i < parts.Length; i++) at = Member(at!, parts[i]);
            return at;
        }

        private static object? Member(object target, string name)
        {
            MemberInfo? member = Members.GetOrAdd((target.GetType(), name), key =>
            {
                const BindingFlags all = BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic;
                for (Type? t = key.Item1; t is not null; t = t.BaseType)
                {
                    if (t.GetField(key.Item2, all) is { } field) return field;
                    if (t.GetProperty(key.Item2, all) is { } property) return property;
                }
                return null;
            });
            return member switch
            {
                FieldInfo field => field.GetValue(target),
                PropertyInfo property => property.GetValue(target),
                _ => throw new InvalidOperationException($"{target.GetType().Name} has no member {name}"),
            };
        }
    }
}

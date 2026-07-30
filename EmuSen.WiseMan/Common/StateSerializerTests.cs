using System;
using System.IO;
using EmuSen.Common;

namespace EmuSen.WiseMan.Common
{
    // Alias skipping and the pre-v1 read path - see EmuSen_Save_States.md §2.
    public class StateSerializerTests
    {
        // Mirrors Spc700/SDsp: one real array plus an alias of it.
        private sealed class Node
        {
            public byte[] Ram = new byte[16];
            [AliasOfSerializedField] public byte[] RamAlias = null!;
            public int Value;

            public static Node Make(byte fill, int value)
            {
                var n = new Node { Value = value };
                Array.Fill(n.Ram, fill);
                n.RamAlias = n.Ram;
                return n;
            }
        }

        private sealed class Outer
        {
            public Node Child = Node.Make(0, 0);
            public int Tag;
        }

        private static byte[] Write(object o)
        {
            var ms = new MemoryStream();
            using (var w = new BinaryWriter(ms, System.Text.Encoding.UTF8, leaveOpen: true)) StateSerializer.Write(w, o);
            return ms.ToArray();
        }

        private static void Read(byte[] bytes, object o, bool includeAliases)
        {
            using var r = new BinaryReader(new MemoryStream(bytes));
            StateSerializer.Read(r, o, includeAliases);
        }

        [Fact]
        public void Writing_skips_the_alias_entirely()
        {
            Assert.Equal(16 + sizeof(int), Write(Node.Make(0xAB, 7)).Length);
        }

        [Fact]
        public void A_v1_payload_round_trips()
        {
            byte[] bytes = Write(Node.Make(0xAB, 7));
            var target = Node.Make(0x00, 0);
            Read(bytes, target, includeAliases: false);

            Assert.Equal(7, target.Value);
            Assert.All(target.Ram, b => Assert.Equal(0xAB, b));
        }

        // A pre-v1 file carries the alias inline, in field-name order - §2.
        [Fact]
        public void A_pre_v1_payload_reads_when_aliases_are_included()
        {
            var ms = new MemoryStream();
            using (var w = new BinaryWriter(ms, System.Text.Encoding.UTF8, leaveOpen: true))
            {
                w.Write(Enumerable_Repeat(0xCD, 16)); // Ram
                w.Write(Enumerable_Repeat(0xCD, 16)); // RamAlias, same bytes on real hardware
                w.Write(99);                          // Value
            }

            var target = Node.Make(0x00, 0);
            Read(ms.ToArray(), target, includeAliases: true);

            Assert.Equal(99, target.Value);
            Assert.All(target.Ram, b => Assert.Equal(0xCD, b));
        }

        [Fact]
        public void A_pre_v1_payload_misaligns_if_aliases_are_not_included()
        {
            var ms = new MemoryStream();
            using (var w = new BinaryWriter(ms, System.Text.Encoding.UTF8, leaveOpen: true))
            {
                w.Write(Enumerable_Repeat(0xCD, 16));
                w.Write(Enumerable_Repeat(0xCD, 16));
                w.Write(99);
            }

            var target = Node.Make(0x00, 0);
            Read(ms.ToArray(), target, includeAliases: false);
            Assert.NotEqual(99, target.Value); // reads into the alias bytes instead
        }

        // The flag has to reach nested objects, since the real aliases live
        // two levels down (Spc700 -> SDsp -> DspVoice) - §2.
        [Fact]
        public void The_alias_flag_propagates_into_nested_objects()
        {
            var source = new Outer { Child = Node.Make(0x5A, 3), Tag = 11 };
            byte[] bytes = Write(source);
            // +1 for the presence flag WriteValue emits ahead of a class field.
            Assert.Equal(1 + 16 + sizeof(int) + sizeof(int), bytes.Length);

            var target = new Outer();
            Read(bytes, target, includeAliases: false);
            Assert.Equal(11, target.Tag);
            Assert.Equal(3, target.Child.Value);
            Assert.All(target.Child.Ram, b => Assert.Equal(0x5A, b));
        }

        private static byte[] Enumerable_Repeat(byte value, int count)
        {
            byte[] b = new byte[count];
            Array.Fill(b, value);
            return b;
        }
    }
}

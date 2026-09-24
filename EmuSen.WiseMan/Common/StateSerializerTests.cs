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

        private struct Pair
        {
            public ushort Low;
            public byte High;
        }

        private sealed class Wide
        {
            public uint[] Words = new uint[3];
            public ulong[] Doubles = new ulong[2];
            public long[] Signed = new long[1];
            public Pair One;
            public Pair[] Many = new Pair[2];
            public char Letter;
        }

        // The palette Venus decodes from CGRAM, which a load had read into boxed copies and dropped - see EmuSen_Save_States.md §5.
        [Fact]
        public void A_uint_array_is_restored_by_a_read()
        {
            var ppu = new EmuSen.Cores.Nintendo.Venus.Video.Ppu();
            ppu.Palette[5] = 0x1122_3344;
            byte[] bytes = Write(ppu);

            ppu.Palette[5] = 0;
            Read(bytes, ppu, includeAliases: false);

            Assert.Equal(0x1122_3344u, ppu.Palette[5]);
        }

        private sealed class Arrays
        {
            public ulong[] Doubles = { 0x0807_0605_0403_0201UL };
            public long[] Signed = { -2 };
            public uint[] Words = { 0x0403_0201 };
        }

        // Passes against the serializer before these types had cases too, which is the whole compatibility claim - see EmuSen_Save_States.md §5.
        [Fact]
        public void Wide_arrays_write_the_bytes_the_element_walk_always_wrote()
        {
            Assert.Equal(new byte[]
            {
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0x01, 0x02, 0x03, 0x04,
            }, Write(new Arrays()));
        }

        [Fact]
        public void Wide_arrays_structs_struct_arrays_and_chars_round_trip()
        {
            var source = new Wide { Letter = 'Z' };
            source.Words[2] = 0xDEAD_BEEF;
            source.Doubles[1] = 0x0123_4567_89AB_CDEFUL;
            source.Signed[0] = -5;
            source.One = new Pair { Low = 0x1234, High = 0x56 };
            source.Many[1] = new Pair { Low = 0xABCD, High = 0xEF };

            var target = new Wide();
            Read(Write(source), target, includeAliases: false);

            Assert.Equal(0xDEAD_BEEFu, target.Words[2]);
            Assert.Equal(0x0123_4567_89AB_CDEFUL, target.Doubles[1]);
            Assert.Equal(-5L, target.Signed[0]);
            Assert.Equal((ushort)0x1234, target.One.Low);
            Assert.Equal((byte)0x56, target.One.High);
            Assert.Equal((ushort)0xABCD, target.Many[1].Low);
            Assert.Equal((byte)0xEF, target.Many[1].High);
            Assert.Equal('Z', target.Letter);
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

        // Mirrors Mercury's version 6: a host string the state dropped, and a second reference to an object it already carries - see EmuSen_Save_States.md §7.
        private sealed class Board
        {
            public byte[] Ram = new byte[4];
            [RetiredFromState] public string? Path = "host";
            [RetiredFromState] public int Old = 5;
            public int Value;
        }

        private sealed class Holder
        {
            public Board Cart = new();
            [RetiredFromState] public Board Copy = null!;
            public int Tag;

            public static Holder Make()
            {
                var h = new Holder();
                h.Copy = h.Cart;
                return h;
            }
        }

        [Fact]
        public void A_retired_field_is_not_written_and_its_older_bytes_are_read_with_the_values_dropped()
        {
            var source = Holder.Make();
            Array.Fill(source.Cart.Ram, (byte)0x11);
            source.Cart.Value = 3;
            source.Tag = 9;
            Assert.Equal(1 + 4 + sizeof(int) + sizeof(int), Write(source).Length);

            var older = new MemoryStream();
            using (var w = new BinaryWriter(older, System.Text.Encoding.UTF8, leaveOpen: true)) StateSerializer.Write(w, source, includeRetired: true);
            byte[] bytes = older.ToArray();
            Assert.Equal(2 * (1 + sizeof(int) + 4 + 1 + "host".Length + sizeof(int)) + sizeof(int), bytes.Length);

            // Each board is its flag, then Old, Path, Ram and Value by name; the copy comes second, so filling its RAM differently shows which one stands.
            int board = 1 + sizeof(int) + 1 + "host".Length + 4 + sizeof(int);
            int copyRam = board + 1 + sizeof(int) + 1 + "host".Length;
            Array.Fill(bytes, (byte)0x22, copyRam, 4);

            var target = Holder.Make();
            target.Cart.Path = "mine";
            target.Cart.Old = 1;
            using var r = new BinaryReader(new MemoryStream(bytes));
            StateSerializer.Read(r, target, includeRetired: true);

            Assert.Equal(bytes.Length, r.BaseStream.Position);
            Assert.Equal("mine", target.Cart.Path);
            Assert.Equal(1, target.Cart.Old);
            Assert.Equal(3, target.Cart.Value);
            Assert.Equal(9, target.Tag);
            Assert.All(target.Cart.Ram, b => Assert.Equal(0x22, b));
        }
    }
}

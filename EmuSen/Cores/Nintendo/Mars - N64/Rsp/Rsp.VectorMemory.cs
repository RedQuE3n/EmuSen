using System;
using System.Runtime.Intrinsics;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // The twelve vector loads and twelve vector stores, each with its own idea of size and alignment - see Mars_RspVector.md §4 and §5.
    public sealed partial class Rsp
    {
        // How far each format scales its seven-bit offset, in the order the format field numbers them.
        private static readonly int[] TransferScale = { 0, 1, 2, 3, 4, 4, 3, 3, 4, 4, 4, 4 };

        private static readonly System.Runtime.Intrinsics.Vector128<byte> SwapPairs = System.Runtime.Intrinsics.Vector128.Create((byte)1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14);

        private const int FormatQuad = 4;
        private const int FormatRest = 5;
        private const int FormatPacked = 6;
        private const int FormatUnsigned = 7;
        private const int FormatHalf = 8;
        private const int FormatFraction = 9;
        private const int FormatWhole = 10;
        private const int FormatTransposed = 11;

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void ExecuteVectorLoad(uint instruction)
        {
            if (!TransferOperands(instruction, out int format, out int vt, out int element, out uint address)) return;

            switch (format)
            {
                case < FormatQuad: LoadBytes(vt, element, address, 1 << format); return;
                case FormatQuad: LoadBytes(vt, element, address, 16 - (int)(address & 0xF)); return;
                case FormatRest: LoadRest(vt, element, address); return;
                case FormatPacked: LoadUnpacked(vt, element, address, shift: 8, stride: 1); return;
                case FormatUnsigned: LoadUnpacked(vt, element, address, shift: 7, stride: 1); return;
                case FormatHalf: LoadUnpacked(vt, element, address, shift: 7, stride: 2); return;
                case FormatFraction: LoadFraction(vt, element, address); return;
                case FormatTransposed: LoadTransposed(vt, element, address); return;

                // The whole-register load does not exist and leaves its register as it was - see §4.
                default: return;
            }
        }

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void ExecuteVectorStore(uint instruction)
        {
            if (!TransferOperands(instruction, out int format, out int vt, out int element, out uint address)) return;

            switch (format)
            {
                case < FormatQuad: StoreBytes(vt, element, address, 1 << format); return;
                case FormatQuad: StoreBytes(vt, element, address, 16 - (int)(address & 0xF)); return;
                case FormatRest: StoreRest(vt, element, address); return;
                case FormatPacked: StorePacked(vt, element, address, lowerShift: 8, upperShift: 7); return;
                case FormatUnsigned: StorePacked(vt, element, address, lowerShift: 7, upperShift: 8); return;
                case FormatHalf: StoreHalves(vt, element, address); return;
                case FormatFraction: StoreFraction(vt, element, address); return;
                case FormatWhole: StoreWhole(vt, element, address); return;
                case FormatTransposed: StoreTransposed(vt, element, address); return;
            }
        }

        private bool TransferOperands(uint instruction, out int format, out int vt, out int element, out uint address)
        {
            format = (int)((instruction >> 11) & 0x1F);
            vt = Rt(instruction);
            element = (int)((instruction >> 7) & 0xF);
            address = 0;

            if (format >= TransferScale.Length) return false;

            int offset = (int)(instruction << 25) >> 25;
            address = (Read(Rs(instruction)) + (uint)(offset << TransferScale[format])) & DataMask;
            return true;
        }

        // A load that runs out of register stops there rather than wrapping - see §4.
        private void LoadBytes(int vt, int element, uint address, int count)
        {
            // A whole register from sixteen bytes that do not wrap is one load with each pair of bytes exchanged - see Mars_RspVector.md §15.
            if (element == 0 && count == 16 && address <= DataMask - 15 && BitConverter.IsLittleEndian)
            {
                var bytes = System.Runtime.Intrinsics.Vector128.LoadUnsafe(ref _bus.SpDmem[address]);
                System.Runtime.Intrinsics.Vector128.StoreUnsafe(System.Runtime.Intrinsics.Vector128.ShuffleNative(bytes, SwapPairs).AsUInt16(), ref Vector[vt * Elements]);
                return;
            }

            for (int i = 0; i < Math.Min(16 - element, count); i++) SetVectorByte(vt, element + i, DataByte(address + (uint)i));
        }

        // The bytes before the address, from the start of its region, landing at the register's end - see §4.
        private void LoadRest(int vt, int element, uint address)
        {
            for (int i = 16 - (int)(address & 0xF); i < 16 && element + i <= 15; i++)
            {
                SetVectorByte(vt, element + i, DataByte(address + (uint)i - 16));
            }
        }

        // Eight bytes widened to eight elements, read in a rotation the selector and misalignment set - see §4.
        private void LoadUnpacked(int vt, int element, uint address, int shift, int stride)
        {
            uint aligned = address & ~7u;
            int misalignment = (int)(address & 7);

            for (int i = 0; i < Elements; i++)
            {
                int at = (misalignment - element + i * stride) & 0xF;
                Vector[vt * Elements + i] = (ushort)(DataByte(aligned + (uint)at) << shift);
            }
        }

        // Bytes at a fixed pattern of offsets widened to eight elements, then copied in from the selector's byte - see §4.
        private void LoadFraction(int vt, int element, uint address)
        {
            uint aligned = address & ~7u;
            int misalignment = (int)(address & 7);

            Span<int> offsets = stackalloc int[] { element, 4 - element, 8 - element, 12 - element, 8 - element, 12 - element, -element, 4 - element };
            Span<ushort> unpacked = stackalloc ushort[Elements];

            for (int i = 0; i < Elements; i++)
            {
                int at = (misalignment + offsets[i]) & 0xF;
                unpacked[i] = (ushort)(DataByte(aligned + (uint)at) << 7);
            }

            for (int b = element; b < element + Math.Min(8, 16 - element); b++)
            {
                SetVectorByte(vt, b, (byte)((b & 1) == 0 ? unpacked[b >> 1] >> 8 : unpacked[b >> 1]));
            }
        }

        // One element into each register of vt's group of eight, starting where the selector says - see §4.
        private void LoadTransposed(int vt, int element, uint address)
        {
            int group = vt & ~7;
            uint aligned = address & ~7u;
            int rotation = (int)(address & 8);

            for (int i = 0; i < Elements; i++)
            {
                int register = group + (((element >> 1) + i) & 7);
                int at = rotation + element + i * 2;

                Vector[register * Elements + i] = (ushort)((DataByte(aligned + (uint)(at & 0xF)) << 8) | DataByte(aligned + (uint)((at + 1) & 0xF)));
            }
        }

        // A store that runs out of register wraps to its start, unlike a load - see §5.
        private void StoreBytes(int vt, int element, uint address, int count)
        {
            if (element == 0 && count == 16 && address <= DataMask - 15 && BitConverter.IsLittleEndian)
            {
                var lanes = System.Runtime.Intrinsics.Vector128.LoadUnsafe(ref Vector[vt * Elements]).AsByte();
                System.Runtime.Intrinsics.Vector128.StoreUnsafe(System.Runtime.Intrinsics.Vector128.ShuffleNative(lanes, SwapPairs), ref _bus.SpDmem[address]);
                return;
            }

            for (int i = 0; i < count; i++) SetDataByte(address + (uint)i, VectorByte(vt, (element + i) & 0xF));
        }

        private void StoreRest(int vt, int element, uint address)
        {
            for (int i = 16 - (int)(address & 0xF); i < 16; i++)
            {
                SetDataByte(address + (uint)i - 16, VectorByte(vt, (element + i) & 0xF));
            }
        }

        // Each element narrowed to a byte, by a shift that changes at the register's midpoint - see §5.
        private void StorePacked(int vt, int element, uint address, int lowerShift, int upperShift)
        {
            for (int i = 0; i < Elements; i++)
            {
                int index = element + i;
                int shift = (index & 8) == 0 ? lowerShift : upperShift;

                SetDataByte(address + (uint)i, (byte)(Vector[vt * Elements + (index & 7)] >> shift));
            }
        }

        // Byte pairs taken from the selector's byte on, which cross element boundaries when it is odd - see §5.
        private void StoreHalves(int vt, int element, uint address)
        {
            uint aligned = address & ~7u;
            int misalignment = (int)(address & 7);

            for (int i = 0; i < Elements; i++)
            {
                int index = element + i * 2;
                int value = (VectorByte(vt, index & 0xF) << 8) | VectorByte(vt, (index + 1) & 0xF);

                SetDataByte(aligned + (uint)((misalignment + i * 2) & 0xF), (byte)(value >> 7));
            }
        }

        // Only eight selectors store anything, each starting at its own element; the rest store zeroes - see §5.
        private void StoreFraction(int vt, int element, uint address)
        {
            uint aligned = address & ~7u;
            int misalignment = (int)(address & 7);

            int first = element switch { 0 => 0, 1 => 6, 4 => 1, 5 => 7, 8 => 4, 11 => 3, 12 => 5, 15 => 0, _ => -1 };

            for (int i = 0; i < 4; i++)
            {
                byte value = first < 0 ? (byte)0 : (byte)(Vector[vt * Elements + ((first & 4) | ((first + i) & 3))] >> 7);
                SetDataByte(aligned + (uint)((misalignment + i * 4) & 0xF), value);
            }
        }

        // The whole register, rotated so that its start lands on the address - see §5.
        private void StoreWhole(int vt, int element, uint address)
        {
            uint aligned = address & ~7u;
            int misalignment = (int)(address & 7);

            for (int i = 0; i < 16; i++) SetDataByte(aligned + (uint)((misalignment + i) & 0xF), VectorByte(vt, (element + i) & 0xF));
        }

        // Two bytes from each register of vt's group, the group's rotation set by the address and selector together - see §5.
        private void StoreTransposed(int vt, int element, uint address)
        {
            int group = vt & ~7;
            uint aligned = address & ~7u;

            for (int i = 0; i < 16; i++)
            {
                int register = group + (((i >> 1) - (int)(aligned >> 1) + (element >> 1)) & 7);
                byte value = VectorByte(register, (int)((i + aligned) & 0xF));

                SetDataByte(aligned + ((address + (uint)i) & 0xF), value);
            }
        }

        private byte DataByte(uint address) => _bus.SpDmem[address & DataMask];

        private void SetDataByte(uint address, byte value) => _bus.SpDmem[address & DataMask] = value;
    }
}

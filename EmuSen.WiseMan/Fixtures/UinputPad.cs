using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // A kernel input device made through /dev/uinput with an Xbox 360 pad's buttons and axes, which SDL finds as it finds a pad plugged in - see EmuSen_Input.md §8.12.
    public sealed unsafe class UinputPad : IDisposable
    {
        public const ushort XboxVendor = 0x045e, Xbox360Product = 0x028e, SteamVendor = 0x28de, SteamVirtualProduct = 0x11ff;

        // BTN_SOUTH, BTN_EAST, BTN_NORTH, BTN_WEST, BTN_TL, BTN_TR, BTN_SELECT, BTN_START, BTN_MODE, BTN_THUMBL, BTN_THUMBR.
        public const ushort South = 0x130, East = 0x131, Start = 0x13b;
        private static readonly ushort[] Keys = [0x130, 0x131, 0x133, 0x134, 0x136, 0x137, 0x13a, 0x13b, 0x13c, 0x13d, 0x13e];
        private static readonly (ushort Code, int Min, int Max, int Fuzz, int Flat)[] Axes =
            [(0, -32768, 32767, 16, 128), (1, -32768, 32767, 16, 128), (3, -32768, 32767, 16, 128), (4, -32768, 32767, 16, 128), (2, 0, 255, 0, 0), (5, 0, 255, 0, 0), (0x10, -1, 1, 0, 0), (0x11, -1, 1, 0, 0)];

        private const int EvSyn = 0, EvKey = 1, EvAbs = 3;
        private const ulong DevCreate = 0x5501, DevDestroy = 0x5502;
        private static ulong Write(uint nr, uint size) => (1UL << 30) | ((ulong)size << 16) | (0x55UL << 8) | nr;
        private static readonly ulong DevSetup = Write(3, 92), AbsSetup = Write(4, 28), SetEvBit = Write(100, 4), SetKeyBit = Write(101, 4), SetAbsBit = Write(103, 4), SetPhys = Write(108, 8);
        private static readonly ulong GetSysName = (2UL << 30) | (64UL << 16) | (0x55UL << 8) | 44;

        [DllImport("libc", SetLastError = true)] private static extern int open(string path, int flags);
        [DllImport("libc", SetLastError = true)] private static extern int ioctl(int fd, ulong request, nint arg);
        [DllImport("libc", SetLastError = true)] private static extern nint write(int fd, void* buffer, nint count);
        [DllImport("libc")] private static extern int close(int fd);

        private int _fd;

        // Null where this account cannot make devices; else the reason the tests that need one are skipped is this text.
        public static string? Unavailable
        {
            get
            {
                if (!OperatingSystem.IsLinux()) return "uinput is Linux's";
                int fd = open("/dev/uinput", 0x801);
                if (fd < 0) return $"/dev/uinput is not writable here (errno {Marshal.GetLastPInvokeError()})";
                close(fd);
                return null;
            }
        }

        public string Name { get; }

        // The kernel's name for it, inputN, and its event node, eventN, once made.
        public string SysName { get; private set; } = "";

        public UinputPad(string name, ushort vendor = XboxVendor, ushort product = Xbox360Product, ushort version = 0x0114, string? phys = null)
        {
            Name = name;
            _fd = open("/dev/uinput", 0x801);
            if (_fd < 0) throw new IOException($"/dev/uinput: errno {Marshal.GetLastPInvokeError()}");
            Check(ioctl(_fd, SetEvBit, EvKey));
            Check(ioctl(_fd, SetEvBit, EvAbs));
            foreach (ushort key in Keys) Check(ioctl(_fd, SetKeyBit, key));
            int* abs = stackalloc int[7];
            foreach (var axis in Axes)
            {
                Check(ioctl(_fd, SetAbsBit, axis.Code));
                abs[0] = axis.Code;
                abs[1] = 0;
                abs[2] = axis.Min;
                abs[3] = axis.Max;
                abs[4] = axis.Fuzz;
                abs[5] = axis.Flat;
                abs[6] = 0;
                Check(ioctl(_fd, AbsSetup, (nint)abs));
            }
            if (phys is not null)
            {
                byte[] text = Encoding.ASCII.GetBytes(phys + "\0");
                fixed (byte* p = text) Check(ioctl(_fd, SetPhys, (nint)p));
            }
            byte* setup = stackalloc byte[92];
            new Span<byte>(setup, 92).Clear();
            *(ushort*)setup = 0x03;
            *(ushort*)(setup + 2) = vendor;
            *(ushort*)(setup + 4) = product;
            *(ushort*)(setup + 6) = version;
            Encoding.ASCII.GetBytes(name.AsSpan(0, Math.Min(name.Length, 79)), new Span<byte>(setup + 8, 79));
            Check(ioctl(_fd, DevSetup, (nint)setup));
            Check(ioctl(_fd, DevCreate, 0));
            byte* sys = stackalloc byte[64];
            new Span<byte>(sys, 64).Clear();
            if (ioctl(_fd, GetSysName, (nint)sys) >= 0) SysName = Marshal.PtrToStringAnsi((nint)sys) ?? "";
        }

        // The event node the kernel gave it, as SDL reports a pad's path.
        public string? EventPath =>
            Directory.Exists($"/sys/devices/virtual/input/{SysName}")
                ? Directory.GetDirectories($"/sys/devices/virtual/input/{SysName}", "event*") is [var node, ..] ? "/dev/input/" + Path.GetFileName(node) : null
                : null;

        public void Press(ushort key, bool down)
        {
            Emit(EvKey, key, down ? 1 : 0);
            Emit(EvSyn, 0, 0);
        }

        private void Emit(ushort type, ushort code, int value)
        {
            byte* e = stackalloc byte[24];
            new Span<byte>(e, 24).Clear();
            *(ushort*)(e + 16) = type;
            *(ushort*)(e + 18) = code;
            *(int*)(e + 20) = value;
            if (write(_fd, e, 24) != 24) throw new IOException($"uinput write: errno {Marshal.GetLastPInvokeError()}");
        }

        private static void Check(int result)
        {
            if (result < 0) throw new IOException($"uinput ioctl: errno {Marshal.GetLastPInvokeError()}");
        }

        // Unplugs it.
        public void Dispose()
        {
            if (_fd < 0) return;
            ioctl(_fd, DevDestroy, 0);
            close(_fd);
            _fd = -1;
        }
    }
}

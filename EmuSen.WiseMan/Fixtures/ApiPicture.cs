using System;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;

namespace EmuSen.WiseMan.Fixtures
{
    // A game's picture after some frames, taken through the frontend API as a window takes it.
    public static class ApiPicture
    {
        public static (byte[] Rgba, int Width, int Height) After(string path, int frames)
        {
            using var runtime = new CrystalRuntime();
            using CrystalSession session = runtime.Open(new CrystalOpen { Path = path, NoBattery = true });
            var info = new CrystalFrameInfo();
            for (int f = 0; f < frames; f++) session.Frame(ref info);
            byte[] rgba = new byte[session.PictureBytes];
            session.CopyPicture(rgba);
            return (rgba, info.width, info.height);
        }
    }
}

using System.IO;
using SkiaSharp;

namespace EmuSen.Hotaru.Imaging
{
    // Encodes a raw RGBA8888 frame buffer straight to a real PNG file, for
    // the F3 screenshot hotkey and FrameRecorder's own per-frame capture
    // (both in Views/GameWindow.axaml.cs) - replacing Raylib.TakeScreenshot,
    // which captured the composited window surface directly and no longer
    // exists once this frontend drops Raylib entirely.
    //
    // Deliberately uses SkiaSharp directly (SKImage.Encode) rather than
    // Avalonia's WriteableBitmap.Save, even though the latter was this
    // migration's original plan text: WriteableBitmap needs Avalonia's own
    // platform render interface resolved via AvaloniaLocator, which only
    // happens once an AppBuilder has actually been set up - fine from
    // GameWindow's real emulation thread (the app is always running by
    // then), but it would make this code untestable from a plain xUnit
    // test with no display and no App.Run anywhere in sight. SkiaSharp's
    // own SKImage/SKData types need no such bootstrap at all - they're
    // already a real, explicit dependency here via EmuSen.Serenity's own
    // PackageReference (see GameFrameControl's identical SKImage usage) -
    // so this produces byte-identical real PNG output while staying
    // directly unit-testable (see
    // EmuSen.WiseMan/Imaging/FrameImageWriterTests.cs).
    public static class FrameImageWriter
    {
        public static void SavePng(byte[] rgba, int width, int height, string path)
            => EmuSen.Common.Imaging.PngFile.Write(path, rgba, width, height);
    }
}

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Serenity.Native;
using Silk.NET.Core.Native;
using Silk.NET.Vulkan;
using SkiaSharp;

namespace EmuSen.Serenity.Slang
{
    // A preset built by the platform library for one of its devices: a pipeline per pass, their outputs, and each frame's run from the game's picture to the screen's - see EmuSen_Serenity.md §7.4.
    public sealed unsafe class SlangChain : IDisposable
    {
        // The library's chain, the device it was built on, and the tokens its lent pictures give their readbacks back by - see EmuSen_RustPlatform.md §17.2.
        private readonly SerenityHandle _native;
        private readonly SlangVulkan _gpu;
        private readonly List<LentPicture> _pictures = new();
        private bool _disposed;

        // Readbacks an image may be lent; a lent one is written again only once its image lets go - see EmuSen_Serenity.md §9.1.
        internal const int MaxLent = 3;

        // Images made by copying because every lendable readback was still held, for a test.
        internal int CopiedImages => (int)Get(SerenityNative.ChainCopiedImages);

        internal int ReadbackCount => (int)Get(SerenityNative.ChainReadbacks);

        public SlangPreset Preset { get; }

        // Every parameter the passes declare, with the preset's value as its default where it gives one - see EmuSen_Serenity.md §7.6.
        public IReadOnlyList<SlangParameter> Parameters { get; }

        // Passes compiled on every processor but one, leaving one for the emulation thread - see EmuSen_Serenity.md §9.5.
        public static int DefaultBuilders => Math.Max(1, Environment.ProcessorCount - 1);

        public SlangChain(SlangVulkan gpu, SlangPreset preset, SpirvCache? cache = null) : this(gpu, preset, cache, DefaultBuilders) { }

        internal SlangChain(SlangVulkan gpu, SlangPreset preset, SpirvCache? cache, int builders)
        {
            Preset = preset;
            _gpu = gpu;
            SerenityHandle? kept = cache?.NativeValue;
            byte[] document = PresetDocument(preset);
            long now = kept is null ? 0 : cache!.Clock();
            nint made;
            fixed (byte* d = document) made = SerenityNative.ChainNew(gpu.NativeValue, d, (nuint)document.Length, kept?.Value ?? 0, now, builders);
            string? words = made == 0 ? SerenityNative.Words() : null;
            Exception? failure = made == 0 ? Waiting() : null;
            GC.KeepAlive(kept);
            GC.KeepAlive(gpu);
            gpu.Heard();
            if (made == 0) throw failure ?? new InvalidOperationException(words);

            _native = new SerenityHandle(made, SerenityNative.ChainFree);
            try
            {
                using (JsonDocument parameters = SerenityNative.Document((buffer, capacity) => SerenityNative.ChainParameters(made, buffer, capacity)))
                    Parameters = SlangParameters.FromJson(parameters.RootElement);
                for (int i = 0; i < preset.Textures.Count; i++) LoadLut(i, preset.Textures[i]);
            }
            catch
            {
                _native.Dispose();
                throw;
            }
        }

        // A preset as the document the library's reader answers with, a float as its 32 bits.
        private static byte[] PresetDocument(SlangPreset preset) => JsonSerializer.SerializeToUtf8Bytes(new Dictionary<string, object?>
        {
            ["path"] = preset.Path,
            ["passes"] = preset.Passes.Select(pass => new Dictionary<string, object?>
            {
                ["shader"] = pass.ShaderPath, ["alias"] = pass.Alias, ["linear"] = pass.FilterLinear, ["wrap"] = (int)pass.Wrap,
                ["type_x"] = (int)pass.ScaleTypeX, ["type_y"] = (int)pass.ScaleTypeY, ["scale_x"] = SerenityNative.Bits(pass.ScaleX), ["scale_y"] = SerenityNative.Bits(pass.ScaleY),
                ["float"] = pass.FloatFramebuffer, ["srgb"] = pass.SrgbFramebuffer, ["mipmap"] = pass.MipmapInput, ["mod"] = pass.FrameCountMod,
            }),
            ["textures"] = preset.Textures.Select(texture => new Dictionary<string, object?>
            {
                ["name"] = texture.Name, ["path"] = texture.Path, ["linear"] = texture.Linear, ["wrap"] = (int)texture.Wrap, ["mipmap"] = texture.Mipmap,
            }),
            ["parameters"] = preset.Parameters.Select(pair => new object[] { pair.Key, SerenityNative.Bits(pair.Value) }),
        });

        // Why the library built no chain, as the exception it stands for; null when the failure is the call's own.
        private static Exception? Waiting()
        {
            byte[]? document = SerenityNative.Bytes((buffer, capacity) => SerenityNative.Take(buffer, capacity), out _);
            if (document is not { Length: > 0 }) return null;
            using JsonDocument why = JsonDocument.Parse(document);
            return SerenityNative.Failure(why.RootElement);
        }

        // The image is decoded here, by Skia's decoder, the one the recorded pictures were drawn with, since a picture is the same only if the decoder is - see EmuSen_RustPlatform.md §17.2.
        private void LoadLut(int index, SlangTextureSpec lut)
        {
            using SKBitmap? decoded = SKBitmap.Decode(lut.Path);
            if (decoded is null) throw new InvalidDataException($"The lookup image {lut.Path} does not decode.");
            using SKBitmap rgba = new(new SKImageInfo(decoded.Width, decoded.Height, SKColorType.Rgba8888, SKAlphaType.Unpremul));
            using (var canvas = new SKCanvas(rgba)) canvas.DrawBitmap(decoded, 0, 0);
            ReadOnlySpan<byte> pixels = rgba.GetPixelSpan();
            int status;
            fixed (byte* p = pixels) status = SerenityNative.ChainLut(_native.Value, (uint)index, rgba.Width, rgba.Height, p, (nuint)pixels.Length);
            After(status);
        }

        // What follows every call into the library's chain: the device's findings and marks handed on, and a failure thrown as its exception.
        private void After(long status)
        {
            string? words = status < 0 ? SerenityNative.Words() : null;
            GC.KeepAlive(_native);
            _gpu!.Heard();
            if (status >= 0) return;
            throw status == SerenityNative.BadArgument ? new ArgumentOutOfRangeException() : new InvalidOperationException(words);
        }

        private long Get(uint which)
        {
            long value = SerenityNative.ChainGet(_native.Value, which);
            GC.KeepAlive(_native);
            return value;
        }

        // Values by id over the defaults, read by the next Render; an id no pass declares is ignored, one left out is its default - see EmuSen_Serenity.md §7.6.
        public void SetParameters(IReadOnlyDictionary<string, float>? values)
        {
            int count = Parameters.Count;
            Span<float> chosen = count <= 1024 ? stackalloc float[count] : new float[count];
            for (int i = 0; i < count; i++) chosen[i] = values is not null && values.TryGetValue(Parameters[i].Id, out float value) ? value : Parameters[i].Initial;
            float none = 0;
            int status;
            fixed (float* c = chosen) status = SerenityNative.ChainSetParameters(_native.Value, c == null ? &none : c, (nuint)count);
            After(status);
        }

        public float ValueOf(string id)
        {
            ArgumentNullException.ThrowIfNull(id);
            byte[] name = SerenityNative.Utf8(id)!;
            float value = 0;
            int found;
            fixed (byte* n = name) found = SerenityNative.ChainValue(_native.Value, SerenityNative.Pin(n), (nuint)name.Length, &value);
            GC.KeepAlive(_native);
            return found == 1 ? value : float.NaN;
        }

        // The format a pass's pragma names, by the library's reading; none named is the fallback.
        public static Format ParseFormat(string? name, Format fallback) =>
            string.IsNullOrEmpty(name) ? fallback : ParseFormatNative(name, fallback);

        internal static Format ParseFormatNative(string name, Format fallback)
        {
            byte[] word = SerenityNative.Utf8(name)!;
            int format;
            fixed (byte* w = word)
                if (SerenityNative.ParseFormat(SerenityNative.Pin(w), (nuint)word.Length, (int)fallback, &format) != 0) throw new InvalidOperationException(SerenityNative.Words());
            return (Format)format;
        }

        // Every pass run for a view no larger than the target and its picture copied into the target's corner, with no readback and no wait - see EmuSen_DevicePicture.md §4.2.
        public void Present(SlangShown target, int width, int height, bool waitFree)
        {
            int status = SerenityNative.ChainPresent(_native.Value, target.Value, width, height, waitFree ? 1 : 0);
            GC.KeepAlive(target);
            After(status);
        }

        // Whether this chain can present to a target, which every chain of the library's can.
        public bool CanPresent => true;

        // A lookup image of the preset given new pixels, written in place when its size is unchanged: a built-in filter's raw frame - see EmuSen_DevicePicture.md §3.4.
        public void ReplaceLut(string name, ReadOnlySpan<byte> rgba, int width, int height)
        {
            int index = -1;
            for (int i = 0; i < Preset.Textures.Count; i++) if (Preset.Textures[i].Name == name) { index = i; break; }
            if (index < 0) throw new ArgumentException($"The preset has no lookup image called '{name}'.", nameof(name));
            int status;
            fixed (byte* p = rgba) status = SerenityNative.ChainLut(_native.Value, (uint)index, width, height, p, (nuint)rgba.Length);
            After(status);
        }

        // A new frame joins the history with its repeated rows, since a slang pass sees the picture as the screen would; the rows are repeated on the device - see EmuSen_Serenity.md §9.2.
        public void Advance(ReadOnlySpan<byte> rgba, int width, int height, int rowRepeat)
        {
            int status;
            fixed (byte* p = rgba) status = SerenityNative.ChainAdvance(_native.Value, p, (nuint)rgba.Length, width, height, rowRepeat);
            After(status);
        }

        // The newest picture a core handed through lend as the new frame, read on the device by the next render or present; false when there is none newer - see EmuSen_DevicePicture.md §17.
        public bool AdvanceLent(SlangLend lend)
        {
            int status = SerenityNative.ChainAdvanceLent(_native.Value, lend.Value);
            GC.KeepAlive(lend);
            After(status);
            return status == 1;
        }

        // The picture a core's frame_ring answered, value in slot, as the new frame: the next render or present waits for the ring's ready, copies the slot and signals its done - see EmuSen_LibretroHost.md §10f.
        public void AdvanceRing(SlangRing ring, int slot, ulong value, int width, int height)
        {
            int status = SerenityNative.ChainAdvanceRing(_native.Value, ring.Value, (uint)slot, value, width, height);
            GC.KeepAlive(ring);
            After(status);
        }

        // Runs every pass for the viewport's size and returns a copy of the last one's pixels, RGBA8, a row after row.
        public byte[] Render(int viewWidth, int viewHeight)
        {
            var pixels = new byte[(ulong)(uint)Math.Max(1, viewWidth) * (uint)Math.Max(1, viewHeight) * 4];
            long status;
            fixed (byte* p = pixels) status = SerenityNative.ChainRender(_native.Value, viewWidth, viewHeight, p, (nuint)pixels.Length);
            After(status);
            return pixels;
        }

        // Runs every pass and returns an image over the readback itself, no copy; the readback is not written again while Skia holds the image - see EmuSen_Serenity.md §9.1.
        public SKImage RenderImage(int viewWidth, int viewHeight)
        {
            var lent = new SerenityNative.Lent { Size = (uint)sizeof(SerenityNative.Lent) };
            After(SerenityNative.ChainRenderLent(_native.Value, viewWidth, viewHeight, &lent));
            var info = new SKImageInfo(lent.Width, lent.Height, SKColorType.Rgba8888, SKAlphaType.Opaque);
            if (lent.Slot == 0) return SKImage.FromPixelCopy(info, (nint)lent.Pixels, lent.RowBytes) ?? throw new InvalidOperationException("Skia refused the chain's picture");
            using var pixmap = new SKPixmap(info, (nint)lent.Pixels, lent.RowBytes);
            SKImage? image = SKImage.FromPixels(pixmap, LentPicture.Released, Picture(lent.Slot));
            if (image is null)
            {
                SerenityNative.LentRelease(lent.Slot);
                throw new InvalidOperationException("Skia refused the chain's picture");
            }
            return image;
        }

        // The token a lent readback is given back by, kept from one frame to the next so that a frame makes none.
        private LentPicture Picture(ulong slot)
        {
            foreach (LentPicture kept in _pictures) if (kept.Slot == slot) return kept;
            if (_pictures.Count >= 2 * MaxLent) _pictures.RemoveAt(0);
            var picture = new LentPicture(slot);
            _pictures.Add(picture);
            return picture;
        }

        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;
            _native.Dispose();
            _gpu.Heard();
        }
    }

    // A readback the library lent a picture, by the number that gives it back; Skia's release, on whatever thread, gives it - see EmuSen_RustPlatform.md §3.3.
    internal sealed unsafe class LentPicture(ulong slot)
    {
        public ulong Slot { get; } = slot;

        public static readonly SKImageRasterReleaseDelegate Released = (_, context) => SerenityNative.LentRelease(((LentPicture)context).Slot);
    }
}

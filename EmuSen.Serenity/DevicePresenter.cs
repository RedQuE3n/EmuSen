using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Platform;
using Avalonia.Rendering.Composition;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity
{
    // A chain's picture handed to Avalonia's compositor on the device, synchronised by semaphores, with no readback - see EmuSen_DevicePicture.md §4.
    public sealed class DevicePresenter : IDisposable
    {
        // Three images, each imported once with its two semaphores, and the update that last used it.
        private sealed class Slot(SlangShown shown, ICompositionImportedGpuImage image, ICompositionImportedGpuSemaphore ready, ICompositionImportedGpuSemaphore free) : IDisposable
        {
            public readonly SlangShown Shown = shown;
            public readonly ICompositionImportedGpuImage Image = image;
            public readonly ICompositionImportedGpuSemaphore Ready = ready, Free = free;
            public Task? Update;
            public bool Used;

            public void Dispose()
            {
                _ = Image.DisposeAsync();
                _ = Ready.DisposeAsync();
                _ = Free.DisposeAsync();
                Shown.Dispose();
            }
        }

        public const int Slots = 3;

        private readonly Compositor _compositor;
        private readonly ICompositionGpuInterop _interop;
        private readonly CompositionDrawingSurface _surface;
        private readonly CompositionSurfaceVisual _visual;
        private readonly CompositionContainerVisual _clip;
        private readonly SlangVulkan _device;
        private readonly Slot?[] _slots = new Slot?[Slots];
        private int _width, _height, _imageWidth, _next;

        // Rows padded to a multiple of 64 pixels, so a compositor's own pitch reads them where they are - see EmuSen_DevicePicture.md §13.
        public const int RowPixels = 64;
        private bool _disposed;

        // The size in pixels of the pictures handed over now.
        public (int Width, int Height) Size => (_width, _height);

        // Whether the images are laid out row after row, the layout the compositor was found to read as written.
        public bool Linear { get; private set; }

        // Frames not handed over because every image was still the compositor's, for a test and a bench.
        public long Skipped { get; private set; }

        // Updates the compositor has applied, and how many presents waited on the device for one.
        public long Presented { get; private set; }

        public CompositionVisual Visual => _clip;

        // The device the pictures are drawn on.
        public SlangVulkan Device => _device;

        private DevicePresenter(Compositor compositor, ICompositionGpuInterop interop, SlangVulkan device)
        {
            _compositor = compositor;
            _interop = interop;
            _device = device;
            _surface = compositor.CreateDrawingSurface();
            _visual = compositor.CreateSurfaceVisual();
            _visual.Surface = _surface;
            _clip = compositor.CreateContainerVisual();
            _clip.ClipToBounds = true;
            _clip.Children.Add(_visual);
        }

        // Why a compositor cannot take the device's picture, or null when it can - see EmuSen_DevicePicture.md §4.4.
        public static string? Refusal(ICompositionGpuInterop? interop, SlangVulkan device)
        {
            const string fd = KnownPlatformGraphicsExternalImageHandleTypes.VulkanOpaquePosixFileDescriptor;
            if (interop is null) return Refusal(null, null, default, null, device.Identity());
            bool images = interop.SupportedImageHandleTypes.Contains(fd);
            return Refusal(interop.SupportedImageHandleTypes, interop.SupportedSemaphoreTypes, images ? interop.GetSynchronizationCapabilities(fd) : default, interop.DeviceUuid, device.Identity());
        }

        // The same over what the compositor and the device say, for a test that has no compositor.
        internal static string? Refusal(IReadOnlyList<string>? images, IReadOnlyList<string>? semaphores, CompositionGpuImportedImageSynchronizationCapabilities synchronisation, byte[]? compositorUuid, (byte[] Uuid, bool Exports) device)
        {
            if (images is null || semaphores is null) return "the compositor offers no GPU interop";
            if (!images.Contains(KnownPlatformGraphicsExternalImageHandleTypes.VulkanOpaquePosixFileDescriptor)) return "the compositor imports no Vulkan image by file descriptor";
            if (!semaphores.Contains(KnownPlatformGraphicsExternalSemaphoreHandleTypes.VulkanOpaquePosixFileDescriptor)) return "the compositor imports no Vulkan semaphore by file descriptor";
            if (!synchronisation.HasFlag(CompositionGpuImportedImageSynchronizationCapabilities.Semaphores)) return "the compositor cannot synchronise an import by semaphores";
            if (!device.Exports) return "the Vulkan device cannot export its images";
            if (compositorUuid is null || !compositorUuid.SequenceEqual(device.Uuid)) return "the compositor draws on another device";
            return null;
        }

        // The presenter for a visual's compositor, or null with the reason; called on the UI thread.
        public static async Task<(DevicePresenter? Presenter, string? Problem)> TryCreateAsync(Visual host, SlangVulkan device)
        {
            if (ElementComposition.GetElementVisual(host)?.Compositor is not { } compositor) return (null, "the control has no compositor yet");
            return await TryCreateAsync(host, device, compositor, await compositor.TryGetCompositionGpuInterop());
        }

        // The same with the compositor's interop already asked for, so that a caller makes no device for a compositor that cannot take its images.
        public static async Task<(DevicePresenter? Presenter, string? Problem)> TryCreateAsync(Visual host, SlangVulkan device, Compositor compositor, ICompositionGpuInterop? interop)
        {
            if (Refusal(interop, device) is { } refused) return (null, refused);
            var presenter = new DevicePresenter(compositor, interop!, device);
            ElementComposition.SetElementChildVisual(host, presenter._clip);
            foreach (bool linear in new[] { true, false })
            {
                presenter.Linear = linear;
                if (await presenter.ReadsAsWrittenAsync()) return (presenter, null);
            }
            presenter.Dispose();
            ElementComposition.SetElementChildVisual(host, null);
            return (null, "the compositor does not show the device's image as it was written");
        }

        // A known picture presented and read back from the compositor: whether the layout tried is the one it reads - see EmuSen_DevicePicture.md §13.
        private async Task<bool> ReadsAsWrittenAsync()
        {
            const int Size = 13, Rows = 7;
            var picture = new byte[Size * Rows * 4];
            for (int i = 0; i < Size * Rows; i++) (picture[i * 4], picture[i * 4 + 1], picture[i * 4 + 2], picture[i * 4 + 3]) = ((byte)(i * 7), (byte)(i * 13 + 5), (byte)(255 - i), 255);
            try
            {
                using var chain = new SlangChain(_device, SlangPreset.Load(Shaders.BuiltInPreset.Write(Shaders.BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
                chain.Advance(picture, Size, Rows, 1);
                if (!Present(chain, Size, Rows) || _slots.FirstOrDefault(s => s?.Update is not null)?.Update is not { } update) return false;
                Place(new Rect(0, 0, Size, Rows));
                await update;
                using Avalonia.Media.Imaging.Bitmap shot = await SnapshotAsync();
                using var stream = new System.IO.MemoryStream();
                shot.Save(stream);
                stream.Position = 0;
                using SkiaSharp.SKBitmap? decoded = SkiaSharp.SKBitmap.Decode(stream);
                if (decoded is null || decoded.Width < Size || decoded.Height < Rows) return false;
                for (int y = 0; y < Rows; y++)
                    for (int x = 0; x < Size; x++)
                    {
                        SkiaSharp.SKColor c = decoded.GetPixel(x, y);
                        int i = (y * Size + x) * 4;
                        if (c.Red != picture[i] || c.Green != picture[i + 1] || c.Blue != picture[i + 2]) return false;
                    }
                return true;
            }
            catch (Exception e) when (e is InvalidOperationException or NotSupportedException or System.IO.IOException or ArgumentException)
            {
                return false;
            }
            finally
            {
                Resize(0, 0);
            }
        }

        // Images made for a new size, the old ones let go once the compositor is done with them.
        private void Resize(int width, int height)
        {
            for (int i = 0; i < Slots; i++)
            {
                if (_slots[i] is { } old)
                {
                    // The compositor's imports are let go of on the UI thread, which owns them.
                    if (old.Update is { IsCompleted: false } pending) pending.ContinueWith(_ => Avalonia.Threading.Dispatcher.UIThread.Post(old.Dispose), TaskScheduler.Default);
                    else old.Dispose();
                    _slots[i] = null;
                }
            }
            (_width, _height, _imageWidth) = (width, height, (width + RowPixels - 1) / RowPixels * RowPixels);
        }

        private Slot Make()
        {
            SlangShown shown = SlangShown.TryCreate(_device, _imageWidth, _height, Linear, out string? problem) ?? throw new InvalidOperationException(problem);
            try
            {
                var properties = new PlatformGraphicsExternalImageProperties
                {
                    Width = _imageWidth, Height = _height, Format = PlatformGraphicsExternalImageFormat.R8G8B8A8UNorm,
                    MemorySize = shown.MemoryBytes, MemoryOffset = 0, TopLeftOrigin = true,
                };
                ICompositionImportedGpuImage image = _interop.ImportImage(new PlatformHandle(shown.TakeMemoryFd(), KnownPlatformGraphicsExternalImageHandleTypes.VulkanOpaquePosixFileDescriptor), properties);
                ICompositionImportedGpuSemaphore ready = _interop.ImportSemaphore(new PlatformHandle(shown.TakeReadyFd(), KnownPlatformGraphicsExternalSemaphoreHandleTypes.VulkanOpaquePosixFileDescriptor));
                ICompositionImportedGpuSemaphore free = _interop.ImportSemaphore(new PlatformHandle(shown.TakeFreeFd(), KnownPlatformGraphicsExternalSemaphoreHandleTypes.VulkanOpaquePosixFileDescriptor));
                return new Slot(shown, image, ready, free);
            }
            catch
            {
                shown.Dispose();
                throw;
            }
        }

        // Runs the chain into an image the compositor is not using and hands it over; false when every image is still its, and the frame is skipped - see EmuSen_DevicePicture.md §4.2.
        public bool Present(SlangChain chain, int width, int height)
        {
            if (_disposed) return false;
            if (width != _width || height != _height) Resize(width, height);
            for (int tried = 0; tried < Slots; tried++)
            {
                int i = (_next + tried) % Slots;
                Slot slot = _slots[i] ??= Make();
                if (slot.Update is { IsCompleted: false }) continue;
                if (slot.Update is { IsFaulted: true } failed) throw failed.Exception!.GetBaseException();
                chain.Present(slot.Shown, _width, _height, waitFree: slot.Used);
                slot.Used = true;
                slot.Update = _surface.UpdateWithSemaphoresAsync(slot.Image, slot.Ready, slot.Free);
                slot.Update.ContinueWith(t => { if (t.IsCompletedSuccessfully) Presented++; }, TaskScheduler.Default);
                _next = (i + 1) % Slots;
                return true;
            }
            Skipped++;
            return false;
        }

        // Where the picture is drawn in its control, in the control's units, and the part of it shown, which a crop makes smaller - see EmuSen_Serenity.md §2.10.
        public void Place(Rect whole, Rect? shown = null)
        {
            Rect visible = (shown ?? whole).Intersect(whole);
            _clip.Offset = new Avalonia.Vector3D(visible.X, visible.Y, 0);
            _clip.Size = new Avalonia.Vector(visible.Width, visible.Height);
            // The image's padded columns lie past the picture's right edge, outside the clip.
            double padded = _width > 0 ? whole.Width * _imageWidth / _width : whole.Width;
            _visual.Offset = new Avalonia.Vector3D(whole.X - visible.X, whole.Y - visible.Y, 0);
            _visual.Size = new Avalonia.Vector(padded, whole.Height);
        }

        // Hides the picture without letting go of its images, while it is drawn some other way.
        public bool Visible
        {
            get => _clip.Visible;
            set => _clip.Visible = value;
        }

        // The compositor's own picture of the visual, read back on its thread: what a player would see, for a test.
        public Task<Avalonia.Media.Imaging.Bitmap> SnapshotAsync() => _compositor.CreateCompositionVisualSnapshot(_visual, 1.0);

        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;
            Resize(0, 0);
            _surface.Dispose();
        }
    }
}

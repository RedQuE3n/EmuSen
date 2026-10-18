using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Rendering.SceneGraph;
using Avalonia.Skia;
using SkiaSharp;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Serenity
{
    // Shows a raw RGBA8888 frame: handed to the compositor on the device, or drawn by Avalonia's Skia with any filter read back from its chain - see EmuSen_Serenity.md §2 and EmuSen_DevicePicture.md §21.
    public sealed class GameFrameControl : Control
    {
        // One offered picture, its rows' repeat (§2.7) and who can still read its array - see EmuSen_Serenity.md §2.8.
        internal sealed class Offer(byte[] rgba, int width, int height, int rowRepeat, long version, Action<byte[]>? release, long sequence = 0, double aspect = 0, int lines = 0, RawPicture? raw = null, Slang.SlangLend? lend = null)
        {
            public readonly byte[] Rgba = rgba;
            // A core's picture on the device, taken through this lend, in place of the pixels - see EmuSen_DevicePicture.md §17.
            public readonly Slang.SlangLend? Lend = lend;
            public readonly double Aspect = aspect;
            public readonly int Lines = lines;
            public readonly RawPicture? Raw = raw;
            public readonly long Sequence = sequence;
            public readonly int Width = width, Height = height, RowRepeat = rowRepeat;
            public readonly long Version = version;
            public readonly Action<byte[]>? Release = release;
            public int Readers;
            public bool Superseded, Released;
        }

        // Each offer is a new version even of the same array, whose contents a core may have rewritten - see EmuSen_Serenity.md §2.6.
        private long _version;

        // The bookkeeping of §2.8, taken briefly by every thread; never held across a copy or a draw.
        private readonly object _offerLock = new();
        private Offer? _current;

        // The offer the cached image was made from, kept readable, and the newest version ever copied, which only moves forward - see EmuSen_Serenity.md §2.8.
        private Offer? _cached;
        private long _copiedVersion = -1;

        // Superseded offers a draw operation may still copy; bounded, and one pushed out is left to the collector unreleased.
        private readonly List<Offer> _held = new(HeldLimit);
        private const int HeldLimit = 8;

        // The image made from the last version drawn, reused when the control is only redrawn; the render thread's, under the lock.
        private readonly object _cacheLock = new();
        private SKImage? _cachedImage;

        // The raw frame of the offer last converted, made only while the running filter reads one; the render thread's and the device path's, under the cache lock.
        private byte[] _rawImage = Array.Empty<byte>();
        private long _rawImageVersion = -1;

        // The offer's picture before the palette as the image the filter's passes read, made once per offer; null where the filter reads none, or not this encoding.
        private byte[]? RawImage(Offer source, ScreenFilter? filter)
        {
            if (source.Raw is not { } raw || filter?.RawEncoding is not { } encoding || raw.Encoding != encoding || raw.Width != source.Width || raw.Height != source.Height) return null;
            if (_rawImageVersion == source.Version) return _rawImage;
            if (_rawImage.Length != raw.Width * raw.Height * 4) _rawImage = new byte[raw.Width * raw.Height * 4];
            if (!Shaders.RawEncodings.ToImage(raw, _rawImage)) return null;
            _rawImageVersion = source.Version;
            return _rawImage;
        }

        private string? _rawEncoding;

        // The encoding of the picture before the palette the chosen filter reads on this console, or null: what a frontend asks its core to keep - see EmuSen_CoreAPI.md §6.30.
        public string? RawEncoding => System.Threading.Volatile.Read(ref _rawEncoding);

        // Under the cache lock, whenever the filter, its console or its values change.
        private void NoteRawEncoding() =>
            System.Threading.Volatile.Write(ref _rawEncoding, _activeFilter is { RawEncoding: { } e } f && f.ReadsRaw(_filterConsole, _shaderParameters) ? e : null);

        // Not "Effect" - Avalonia's Visual already has an unrelated one - see EmuSen_Serenity.md §2.4.
        public ShaderEffect ActiveEffect { get; set; } = ShaderEffect.None;

        private volatile bool _bilinearFiltering = true, _syncToDisplay = true;

        // graphics.json's filtering, which the frontend sets; read on the render thread - see EmuSen_Settings_Reference.md §3.
        public bool BilinearFiltering
        {
            get => _bilinearFiltering;
            set => _bilinearFiltering = value;
        }

        // graphics.json's display sync, which the frontend sets and its emulation thread's pacer reads each frame - see EmuSen_Settings_Reference.md §4.87.
        public bool SyncToDisplay
        {
            get => _syncToDisplay;
            set => _syncToDisplay = value;
        }

        // A multi-pass filter, drawn instead of ActiveEffect when set, by its chain on the device or read back - see EmuSen_Serenity.md §3.2 and EmuSen_DevicePicture.md §21.
        private ScreenFilter? _activeFilter;

        // Makes the next draw wait for the chosen filter's chain to be built, for a test or a bench that must not see the plain picture first.
        internal void WaitForFilter()
        {
            lock (_cacheLock) _waitForBuild = true;
        }

        public ScreenFilter? ActiveFilter
        {
            get => _activeFilter;
            set
            {
                lock (_cacheLock)
                {
                    if (ReferenceEquals(_activeFilter, value)) return;
                    _activeFilter = value;
                    NoteRawEncoding();
                }
                InvalidateVisual();
            }
        }

        private string? _filterConsole;

        // The console whose defaults a filter's parameters start from; null is the parameters' own - see EmuSen_Serenity.md §3.9.
        public string? FilterConsole
        {
            get { lock (_cacheLock) return _filterConsole; }
            set
            {
                lock (_cacheLock)
                {
                    if (string.Equals(_filterConsole, value, StringComparison.Ordinal)) return;
                    _filterConsole = value;
                    NoteRawEncoding();
                }
                InvalidateVisual();
            }
        }

        // A RetroArch preset, drawn instead of both when it is built; until then, or if it cannot be, the picture is drawn plain - see EmuSen_Serenity.md §7.5.
        private Slang.SlangRunner? _slang;

        // Raised off the UI thread, once per preset that cannot be drawn, with the reason.
        public event Action<string>? SlangFailed;

        public string? ActiveSlangPreset
        {
            get { lock (_cacheLock) return _slang?.PresetPath; }
            set
            {
                lock (_cacheLock)
                {
                    if (string.Equals(_slang?.PresetPath, value, StringComparison.Ordinal)) return;
                    _slang?.Dispose();
                    _slang = value is null ? null : new Slang.SlangRunner(value, problem => SlangFailed?.Invoke(problem),
                        () => Avalonia.Threading.Dispatcher.UIThread.Post(InvalidateVisual));
                    _slang?.SetParameters(_shaderParameters);
                }
                InvalidateVisual();
            }
        }

        private IReadOnlyDictionary<string, float>? _shaderParameters;

        // The player's values for the filter or preset drawn, by parameter id; both take them at their next draw - see EmuSen_Serenity.md §7.6.
        public IReadOnlyDictionary<string, float>? ShaderParameters
        {
            get { lock (_cacheLock) return _shaderParameters; }
            set
            {
                lock (_cacheLock)
                {
                    _shaderParameters = value is null ? null : new Dictionary<string, float>(value, StringComparer.Ordinal);
                    NoteRawEncoding();
                    _slang?.SetParameters(_shaderParameters);
                }
                InvalidateVisual();
            }
        }

        // What the running filter's chain holds for a parameter, NaN when there is no chain yet, for a test.
        internal float FilterParameter(string id) { lock (_cacheLock) return _builtIn?.Chain?.ValueOf(id) ?? float.NaN; }

        // Whether the preset has finished building, for a test that waits on it.
        internal bool SlangBuilt { get { lock (_cacheLock) return _slang?.Built ?? true; } }

        // What the render thread spent showing frames, summed until a frontend takes them - see EmuSen_Serenity.md §2.5.
        private long _presented, _copies, _copyTicks, _drawTicks, _shape;
        private int _gpu;

        public readonly record struct PresentationStatistics(long Frames, long Copies, double CopyMilliseconds, double DrawMilliseconds, bool Gpu, int Width, int Height);

        // Safe from any thread: the sums are exchanged for zero, so each frame is counted by exactly one take.
        public PresentationStatistics TakeStatistics()
        {
            double ms = 1000.0 / System.Diagnostics.Stopwatch.Frequency;
            long shape = System.Threading.Interlocked.Read(ref _shape);
            return new PresentationStatistics(
                System.Threading.Interlocked.Exchange(ref _presented, 0),
                System.Threading.Interlocked.Exchange(ref _copies, 0),
                System.Threading.Interlocked.Exchange(ref _copyTicks, 0) * ms,
                System.Threading.Interlocked.Exchange(ref _drawTicks, 0) * ms,
                System.Threading.Volatile.Read(ref _gpu) != 0,
                (int)(shape >> 32), (int)shape);
        }

        // Every draw is a presenter tick the display clock learns from, and a trace line when one is being written - see EmuSen_Settings_Reference.md §4.87.2.
        private void SampleDisplay(long started, long sequence, bool fresh)
        {
            bool sampled = VblankCounter.TrySample(out long ust, out long msc, out long sbc);
            if (sampled) Display.ObserveVblank(ust, msc, started);
            else Display.ObserveTick(started);
            PresentationTrace.Draw(started, sequence, fresh, sampled, ust, msc, sbc);
        }

        // The render timer's rate while a game's pictures arrive, so one is drawn as it is handed over; 0 leaves the platform's - see EmuSen_Settings_Reference.md §4.87.10.
        public static int RenderFps { get; set; } = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_RENDER_FPS"), out int fps) ? fps : DefaultRenderFps;
        public const int DefaultRenderFps = 1000;

        // A quarter of a second with no picture gives the platform its rate back, so menus over a paused game do not tick at the game's.
        private static readonly TimeSpan RenderHoldIdle = TimeSpan.FromMilliseconds(250);
        private static Avalonia.Threading.DispatcherTimer? _renderHoldTimer;
        private static long _lastHeldPicture;
        private static int _platformFps;

        private void HoldRenderRate()
        {
            _lastHeldPicture = System.Diagnostics.Stopwatch.GetTimestamp();
            if (_renderHoldTimer is null)
            {
                int before = RenderTimerRate.Current;
                if (!RenderTimerRate.Hold(RenderFps)) return;
                _platformFps = before;
                _renderHoldTimer = new Avalonia.Threading.DispatcherTimer { Interval = RenderHoldIdle };
                _renderHoldTimer.Tick += (_, _) =>
                {
                    if (System.Diagnostics.Stopwatch.GetElapsedTime(_lastHeldPicture) < RenderHoldIdle && SyncToDisplay) return;
                    _renderHoldTimer!.Stop();
                    _renderHoldTimer = null;
                    RenderTimerRate.Hold(_platformFps);
                };
                _renderHoldTimer.Start();
            }
            else RenderTimerRate.Hold(RenderFps);
        }

        // The refresh this control's draws have seen, for a frontend that paces to it.
        public DisplayClock Display { get; } = new();

        private void Presented(bool copied, long copyTicks, long drawTicks, bool gpu, int width, int height)
        {
            if (copied) System.Threading.Interlocked.Increment(ref _copies);
            System.Threading.Interlocked.Add(ref _copyTicks, copyTicks);
            System.Threading.Interlocked.Add(ref _drawTicks, drawTicks);
            System.Threading.Volatile.Write(ref _gpu, gpu ? 1 : 0);
            System.Threading.Interlocked.Exchange(ref _shape, (long)width << 32 | (uint)height);
            System.Threading.Interlocked.Increment(ref _presented);
            System.Threading.Volatile.Write(ref _lastShown, System.Diagnostics.Stopwatch.GetTimestamp());
            System.Threading.Interlocked.Exchange(ref _offersUnshown, 0);
        }

        // The device path watched: offers it never shows for StallSeconds give it up for the readback - see EmuSen_DevicePicture.md §18.
        private long _lastShown;
        private int _offersUnshown, _deviceGivenUp;
        internal const double StallSeconds = 1.0;
        internal const int StallOffers = 30;

        private void WatchDevicePath()
        {
            if (System.Threading.Volatile.Read(ref _presenter) is null || System.Threading.Volatile.Read(ref _deviceGivenUp) != 0) return;
            if (System.Threading.Interlocked.Increment(ref _offersUnshown) < StallOffers) return;
            if (System.Diagnostics.Stopwatch.GetElapsedTime(System.Threading.Volatile.Read(ref _lastShown)).TotalSeconds < StallSeconds) return;
            if (System.Threading.Interlocked.Exchange(ref _deviceGivenUp, 1) != 0) return;
            Avalonia.Threading.Dispatcher.UIThread.Post(() => GiveUpDevicePath($"nothing was shown for {StallSeconds:0.#} s while {StallOffers} or more frames were offered"));
        }

        // On the UI thread: the presenter let go of and the picture drawn by Skia from now on, said once.
        internal void GiveUpDevicePath(string why)
        {
            DevicePresenter? presenter;
            lock (_cacheLock) { presenter = _presenter; _presenter = null; }
            if (presenter is null) return;
            System.Threading.Volatile.Write(ref _deviceGivenUp, 1);
            Avalonia.Rendering.Composition.ElementComposition.SetElementChildVisual(this, null);
            presenter.Dispose();
            DevicePictureReport?.Invoke($"The picture is read back from now on: {why}");
            InvalidateVisual();
        }

        // Stores the frame and asks for a repaint; release, if given, is called once with the array when nothing here can read it again - see EmuSen_Serenity.md §2.8.
        // aspect is the screen's width over height as the core reports it, 0 for none - see EmuSen_Serenity.md §2.9.
        // lines is the console's own count in a frame drawn at a multiple of its picture, 0 for the frame's rows - see EmuSen_CRT.md §16.
        // raw is the frame before the console's palette, where its core keeps one, for a filter that models the console's own signal - see EmuSen_CRT.md §22.
        // lend is a core's picture on the device in place of rgba, which the device path alone draws - see EmuSen_DevicePicture.md §17.
        public void UpdateFrame(byte[] rgba, int width, int height, int rowRepeat = 1, Action<byte[]>? release = null, long sequence = 0, double aspect = 0, int lines = 0, RawPicture? raw = null, Slang.SlangLend? lend = null)
        {
            WatchDevicePath();
            if (RenderFps > 0 && SyncToDisplay) HoldRenderRate();
            lock (_offerLock)
            {
                Offer? previous = _current;
                _current = new Offer(rgba, width, height, Math.Max(1, rowRepeat), ++_version, release, sequence, aspect, lines, raw, lend);
                if (previous is not null)
                {
                    previous.Superseded = true;
                    if (!TryRelease(previous)) Hold(previous);
                }
            }
            InvalidateVisual();
        }

        // Under _offerLock: an offer is dead once superseded, not the cache's, and read by nothing that could still copy it - see EmuSen_Serenity.md §2.8.
        private bool TryRelease(Offer offer)
        {
            if (offer.Released || !offer.Superseded || ReferenceEquals(offer, _cached)) return false;
            if (offer.Readers > 0 && offer.Version > _copiedVersion) return false;
            offer.Released = true;
            if (offer.Release is { } release && !StillOffered(offer)) release(offer.Rgba);
            if (offer.Release is { } giveBack && offer.Raw is { } raw) giveBack(raw.Values);
            return true;
        }

        // The same array offered again, as a core that rewrites its own buffer does, is not given back while a later offer holds it.
        private bool StillOffered(Offer offer)
        {
            if (ReferenceEquals(_current?.Rgba, offer.Rgba) || ReferenceEquals(_cached?.Rgba, offer.Rgba)) return true;
            foreach (Offer held in _held)
            {
                if (held != offer && !held.Released && ReferenceEquals(held.Rgba, offer.Rgba)) return true;
            }
            return false;
        }

        private void Hold(Offer offer)
        {
            if (_held.Count == HeldLimit) _held.RemoveAt(0);
            _held.Add(offer);
        }

        // Under _offerLock: whatever the newer copy has made dead is given back.
        private void ReleaseHeld()
        {
            for (int i = _held.Count - 1; i >= 0; i--)
            {
                if (_held[i].Released || TryRelease(_held[i])) _held.RemoveAt(i);
            }
        }

        // A draw operation's reference, taken when it is made and put down when it is disposed or has finished reading.
        private Offer? TakeReader(Offer? offer)
        {
            lock (_offerLock)
            {
                offer ??= _current;
                if (offer is not null) offer.Readers++;
                return offer;
            }
        }

        private void PutReader(Offer offer)
        {
            lock (_offerLock)
            {
                offer.Readers--;
                if (offer.Superseded) ReleaseHeld();
            }
        }

        private bool _squarePixels;

        // The frame drawn at its own pixel count's shape, not the screen's the core reports; a filter that states a shape keeps it - see EmuSen_Serenity.md §2.9.
        public bool SquarePixels
        {
            get => _squarePixels;
            set { _squarePixels = value; InvalidateVisual(); }
        }

        private PictureCrop _crop;

        // What is hidden at the picture's edges, the rest filling the rectangle the whole picture had; nothing by default - see EmuSen_Serenity.md §2.10.
        public PictureCrop Crop
        {
            get => _crop;
            set { _crop = value.IsNone ? PictureCrop.None : value; InvalidateVisual(); }
        }

        // Fits a picture of a given shape inside the bounds, or the frame's own shape when the aspect is 0 - see EmuSen_Serenity.md §2.9.
        public static (double X, double Y, double Width, double Height) ComputeLetterboxRect(
            double sourceWidth, double sourceHeight, double aspect, double actualWidth, double actualHeight) =>
            aspect > 0 ? ComputeLetterboxRect(aspect, 1, actualWidth, actualHeight) : ComputeLetterboxRect(sourceWidth, sourceHeight, actualWidth, actualHeight);

        // Fits the frame, centered, inside the control's real bounds - see EmuSen_Serenity.md §2.1.
        public static (double X, double Y, double Width, double Height) ComputeLetterboxRect(
            double sourceWidth, double sourceHeight, double actualWidth, double actualHeight)
        {
            if (sourceWidth <= 0 || sourceHeight <= 0 || actualWidth <= 0 || actualHeight <= 0)
            {
                return (0, 0, 0, 0);
            }

            double scale = Math.Min(actualWidth / sourceWidth, actualHeight / sourceHeight);
            double w = sourceWidth * scale;
            double h = sourceHeight * scale;
            return ((actualWidth - w) * 0.5, (actualHeight - h) * 0.5, w, h);
        }

        // The picture handed to the compositor on the device where it can be, Skia drawing it otherwise; EMUSEN_DEVICE_PICTURE=0 keeps Skia's - see EmuSen_DevicePicture.md §4 and §18.
        public static bool DevicePictureAllowed { get; set; } = Environment.GetEnvironmentVariable("EMUSEN_DEVICE_PICTURE") != "0";

        // Why the picture is not handed over on the device, said once per window; null when it is.
        public event Action<string>? DevicePictureReport;

        private DevicePresenter? _presenter;
        private System.Threading.Tasks.Task? _presenterStart;

        // Whether the picture is being handed to the compositor on the device, for a frontend's readout and a test.
        public bool PresentsOnDevice => _presenter is { Visible: true };

        public DevicePresenter? Presenter => _presenter;

        // The device the picture is presented on, for the frontend API's runtime to lend each machine so its pictures reach the chain there; null while the picture is not presented on the device - see EmuSen_DevicePicture.md §17 and EmuSen_Backend.md §14.12.
        public Slang.SlangVulkan? LendableDevice
        {
            get
            {
                DevicePresenter? presenter;
                lock (_cacheLock) presenter = _presenter;
                if (presenter is null || !DevicePictureAllowed || !DeviceLendAllowed) return null;
                if (presenter.Device.IsNative) return presenter.Device;
                DevicePictureReport?.Invoke("No device is lent to the core: the C#'s device lends nothing");
                return null;
            }
        }

        // EMUSEN_DEVICE_LEND=0 lends no core the device, its pictures then read back as before - see EmuSen_DevicePicture.md §17.
        public static bool DeviceLendAllowed { get; set; } = Environment.GetEnvironmentVariable("EMUSEN_DEVICE_LEND") != "0";

        // Waits for the presenter's start, for a probe that must not see the Skia picture first.
        internal System.Threading.Tasks.Task PresenterStarted => _presenterStart ?? System.Threading.Tasks.Task.CompletedTask;

        private async System.Threading.Tasks.Task StartPresenterAsync()
        {
            // The compositor is asked first, so a window whose compositor imports nothing (a headless one, EGL) makes no device.
            if (Avalonia.Rendering.Composition.ElementComposition.GetElementVisual(this)?.Compositor is not { } compositor) return;
            var interop = await compositor.TryGetCompositionGpuInterop();
            if (interop is null || !interop.SupportedImageHandleTypes.Contains(Avalonia.Platform.KnownPlatformGraphicsExternalImageHandleTypes.VulkanOpaquePosixFileDescriptor))
            {
                DevicePictureReport?.Invoke("The picture is read back: " + (interop is null ? "the compositor offers no GPU interop" : "the compositor imports no Vulkan image by file descriptor"));
                return;
            }
            Slang.SlangVulkan? device = await System.Threading.Tasks.Task.Run(() => Slang.SlangRunner.SharedDevice);
            if (device is null) { DevicePictureReport?.Invoke($"No Vulkan device: {Slang.SlangRunner.DeviceReport}"); return; }
            var (presenter, problem) = await DevicePresenter.TryCreateAsync(this, device, compositor, interop);
            if (presenter is null) { DevicePictureReport?.Invoke($"The picture is read back: {problem}"); return; }
            if (VisualRoot is null) { presenter.Dispose(); return; }
            presenter.Visible = false;
            System.Threading.Volatile.Write(ref _lastShown, System.Diagnostics.Stopwatch.GetTimestamp());
            lock (_cacheLock) _presenter = presenter;
            InvalidateVisual();
        }

        // The chain a built-in filter, a simple effect or the plain picture is drawn by, on the device or read back, and the one built to take over from it - see EmuSen_DevicePicture.md §19.
        private Slang.SlangRunner? _builtIn, _builtInNext;
        private BuiltInKey? _builtInKey, _builtInNextKey;
        private long _builtInVersion = -1, _slangVersion = -1;
        private bool _waitForBuild;

        // What a built-in's preset is written from: the filter, its console, its structural values in order, and the multiple its frame is brought back from.
        private sealed record BuiltInKey(ScreenFilter Filter, string? Console, string Structural, int By);

        // Under the lock: the filter's chain, a new one started on a pool thread when what it is written from changes and taking over once built, and the filter the one returned draws; null until the first is.
        private Slang.SlangRunner? BuiltInFor(Offer offer, ref ScreenFilter? wanted)
        {
            ScreenFilter filter = wanted!;
            bool chosen = ReferenceEquals(filter, _activeFilter);
            string? console = chosen ? _filterConsole : null;
            Dictionary<string, float>? structural = chosen ? Shaders.BuiltInPreset.StructuralOf(filter, _shaderParameters) : null;
            int by = Shaders.BuiltInPreset.ReduceBy(filter, offer.Width, offer.Height, offer.Lines);
            var key = new BuiltInKey(filter, console, structural is null ? "" : string.Join(",", structural.OrderBy(v => v.Key, StringComparer.Ordinal).Select(v => $"{v.Key}={v.Value:R}")), by);
            if (key == _builtInKey && _builtInNext is not null)
            {
                _builtInNext.Dispose();
                (_builtInNext, _builtInNextKey) = (null, null);
            }
            else if (key != _builtInKey && key != _builtInNextKey)
            {
                _builtInNext?.Dispose();
                _builtInNext = new Slang.SlangRunner(filter.Name, () => Shaders.BuiltInPreset.Write(filter, console, structural, by), problem => DevicePictureReport?.Invoke($"The built-in filter is not drawn: {problem}"),
                    () => Avalonia.Threading.Dispatcher.UIThread.Post(InvalidateVisual));
                _builtInNextKey = key;
            }
            if (_builtInNext is { } next && (next.Built || _waitForBuild))
            {
                next.WaitBuilt();
                _builtIn?.Dispose();
                (_builtIn, _builtInKey, _builtInNext, _builtInNextKey, _builtInVersion) = (next, key, null, null, -1);
            }
            _waitForBuild = false;
            if (_builtInKey is { } drawn) wanted = drawn.Filter;
            return _builtIn;
        }

        // The values a built-in's chain is given, made again only when what they come from changes.
        private (ScreenFilter? Filter, IReadOnlyDictionary<string, float>? Values, string? Console, Point Origin, PictureCrop Crop, int RawWidth, int RawHeight) _valuesKey;
        private Slang.SlangRunner? _valuesRunner;

        // Under the lock: each parameter's value and the runner's own (the picture's corner on the display, a crop the filter places, the raw frame's size) handed to the chain.
        private void GiveValues(Slang.SlangRunner runner, ScreenFilter filter, Point origin, PictureCrop crop, int rawWidth, int rawHeight)
        {
            bool chosen = ReferenceEquals(filter, _activeFilter);
            IReadOnlyDictionary<string, float>? given = chosen ? _shaderParameters : null;
            string? console = chosen ? _filterConsole : null;
            var key = (filter, given, console, origin, crop, rawWidth, rawHeight);
            if (key == _valuesKey && ReferenceEquals(runner, _valuesRunner)) return;
            var values = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (Slang.SlangParameter parameter in filter.Parameters ?? Array.Empty<Slang.SlangParameter>())
                values[parameter.Id] = given is not null && given.TryGetValue(parameter.Id, out float value) ? value : filter.DefaultFor(parameter, console);
            (values[Shaders.BuiltInPreset.OriginX], values[Shaders.BuiltInPreset.OriginY]) = ((float)origin.X, (float)origin.Y);
            (values[Shaders.BuiltInPreset.RawWidth], values[Shaders.BuiltInPreset.RawHeight]) = (rawWidth, rawHeight);
            (values[Shaders.BuiltInPreset.CropLeft], values[Shaders.BuiltInPreset.CropTop], values[Shaders.BuiltInPreset.CropWidth], values[Shaders.BuiltInPreset.CropHeight]) = ((float)crop.Left, (float)crop.Top, (float)crop.Width, (float)crop.Height);
            runner.SetParameters(values);
            (_valuesKey, _valuesRunner) = (key, runner);
        }

        // On the UI thread, under the lock: the newest offer run through the device's chain into the compositor; false when the device path cannot draw it now - see EmuSen_DevicePicture.md §4.3.
        private bool PresentOnDevice(Offer offer)
        {
            if (_presenter is not { } presenter) return false;
            Slang.SlangRunner? runner = _slang;
            ScreenFilter? filter = runner is null ? _activeFilter ?? BuiltInShaders.AsFilter(ActiveEffect, BilinearFiltering) : null;
            if (filter is not null && (runner = BuiltInFor(offer, ref filter)) is null) return false;

            Rect whole = DestinationFor(offer, filter?.Aspect), shown = whole;
            bool cropsItself = filter?.CropsItself == true;
            if (!cropsItself && !_crop.IsNone)
            {
                var (x, y, w, h) = _crop.Whole(whole.X, whole.Y, whole.Width, whole.Height);
                whole = new Rect(x, y, w, h);
            }
            double scaling = TopLevel.GetTopLevel(this)?.RenderScaling ?? 1;
            Rect snapped = new(Math.Round(whole.X * scaling) / scaling, Math.Round(whole.Y * scaling) / scaling, Math.Round(whole.Width * scaling) / scaling, Math.Round(whole.Height * scaling) / scaling);
            if (filter is not null)
            {
                Point origin = TopLevel.GetTopLevel(this) is { } top && this.TranslatePoint(snapped.Position, top) is { } p ? new Point(p.X * scaling, p.Y * scaling) : snapped.Position;
                bool raw = RawImage(offer, filter) is not null;
                GiveValues(runner!, filter, origin, cropsItself ? _crop : PictureCrop.None, raw ? offer.Width : 0, raw ? offer.Height : 0);
            }

            bool newFrame = offer.Version != (runner == _slang ? _slangVersion : _builtInVersion);
            int repeat = filter is { RowsOnce: false } ? offer.RowRepeat : 1;
            if (offer.Lend is { } lend && !ReferenceEquals(lend.Owner, presenter.Device)) return false;
            if (!runner!.Present(presenter, offer.Rgba, offer.Width, offer.Height, repeat, newFrame, (int)Math.Round(snapped.Width * scaling), (int)Math.Round(snapped.Height * scaling), RawImage(offer, filter), offer.Lend)) return false;
            if (runner == _slang) _slangVersion = offer.Version;
            else _builtInVersion = offer.Version;
            presenter.Place(snapped, cropsItself ? null : shown);
            presenter.Visible = true;
            return true;
        }

        // The rectangle a picture is drawn in, as the draw operation decides it - see EmuSen_Serenity.md §2.9.
        private Rect DestinationFor(Offer source, double? filterAspect)
        {
            double aspect = filterAspect ?? (_squarePixels ? 0 : source.Aspect);
            PictureCrop crop = _crop;
            var (x, y, w, h) = ComputeLetterboxRect(source.Width * crop.Width, source.Height * source.RowRepeat * crop.Height, aspect, Bounds.Width, Bounds.Height);
            return new Rect(x, y, Math.Max(1, Math.Round(w)), Math.Max(1, Math.Round(h)));
        }

        // The cached image is native memory the size of a frame, so a control leaving the window gives it back; the offer it came from stays, to be copied again.
        protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
        {
            base.OnDetachedFromVisualTree(e);
            lock (_cacheLock)
            {
                _presenter?.Dispose();
                _presenter = null;
                _presenterStart = null;
                _builtIn?.Dispose();
                _builtInNext?.Dispose();
                (_builtIn, _builtInKey, _builtInNext, _builtInNextKey, _builtInVersion, _valuesRunner) = (null, null, null, null, -1, null);
                _cachedImage?.Dispose();
                _cachedImage = null;
            }
        }

        // Shown again, the device is asked for again; the filter's chain, let go on leaving, is built again at the next draw.
        protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
        {
            base.OnAttachedToVisualTree(e);
            if (DevicePictureAllowed && _presenterStart is null) _presenterStart = StartPresenterAsync();
        }

        public override void Render(DrawingContext context)
        {
            base.Render(context);
            if (_presenter is not null && TryRenderOnDevice(context)) return;
            if (CaptureDrawOp(Bounds.Size) is { } op) context.Custom(op);
        }

        // The device path's turn of Render: the picture handed over, and a draw that only keeps the display's clock - see EmuSen_DevicePicture.md §4.3.
        private bool TryRenderOnDevice(DrawingContext context)
        {
            Offer? offer = TakeReader(null);
            if (offer is null) return false;
            long started = System.Diagnostics.Stopwatch.GetTimestamp();
            bool shown;
            lock (_cacheLock)
            {
                try { shown = PresentOnDevice(offer); }
                catch (Exception e) when (e is InvalidOperationException or ArgumentException or NotSupportedException)
                {
                    DevicePictureReport?.Invoke($"The picture is read back from now on: {e.Message}");
                    _presenter?.Dispose();
                    _presenter = null;
                    shown = false;
                }
                if (!shown) { if (_presenter is { } p) p.Visible = false; }
            }
            if (!shown) { PutReader(offer); return false; }
            bool fresh;
            lock (_offerLock) fresh = offer.Version > _copiedVersion;
            if (fresh) Copied(offer);
            Presented(fresh, 0, System.Diagnostics.Stopwatch.GetTimestamp() - started, true, offer.Width, offer.Height);
            context.Custom(new ClockOp(new Rect(Bounds.Size), this, offer.Sequence, fresh));
            PutReader(offer);
            return true;
        }

        // A draw that does nothing but tell the display's clock that the render thread drew, as every draw of the Skia path does - see EmuSen_Settings_Reference.md §4.87.2.
        private sealed class ClockOp(Rect bounds, GameFrameControl owner, long sequence, bool fresh) : ICustomDrawOperation
        {
            public Rect Bounds { get; } = bounds;
            public bool HitTest(Point p) => Bounds.Contains(p);
            public bool Equals(ICustomDrawOperation? other) => false;
            public void Dispose() { }
            public void Render(ImmediateDrawingContext context) => owner.SampleDisplay(System.Diagnostics.Stopwatch.GetTimestamp(), sequence, fresh);
        }

        // What Render hands the compositor, holding the newest offer until disposed; the tests hold one and draw it when they like.
        internal DrawOp? CaptureDrawOp(Size size)
        {
            Offer? offer = TakeReader(null);
            if (offer is null) return null;
            // A picture on the device has no pixels here: the device path draws it or nothing does.
            if (offer.Width > 0 && offer.Height > 0 && offer.Lend is null) return new DrawOp(new Rect(size), this, offer, ActiveEffect);
            PutReader(offer);
            return null;
        }

        // The render thread's side of §2.8: the newest version ever copied is what a later draw shows, whichever operation draws it.
        private Offer BeginRead(Offer mine, out bool fresh)
        {
            lock (_offerLock)
            {
                fresh = mine.Version > _copiedVersion;
                Offer source = fresh ? mine : _cached!;
                source.Readers++;
                return source;
            }
        }

        private void Copied(Offer offer)
        {
            lock (_offerLock)
            {
                _cached = offer;
                _copiedVersion = offer.Version;
                ReleaseHeld();
            }
        }

        // One Render() call's offer, drawn forward only; the control's cache is under its lock - see EmuSen_Serenity.md §2.3 and §2.8.
        internal sealed class DrawOp : ICustomDrawOperation
        {
            private readonly GameFrameControl _owner;
            private readonly Offer _offer;
            private readonly ShaderEffect _effect;
            private bool _disposed;

            internal DrawOp(Rect bounds, GameFrameControl owner, Offer offer, ShaderEffect effect)
            {
                Bounds = bounds;
                _owner = owner;
                _offer = offer;
                _effect = effect;
            }

            public Rect Bounds { get; }

            public bool HitTest(Point p) => Bounds.Contains(p);

            public bool Equals(ICustomDrawOperation? other) => false; // always re-render - this is a live video feed, never cacheable

            public void Dispose()
            {
                lock (_owner._offerLock)
                {
                    if (_disposed) return;
                    _disposed = true;
                }
                _owner.PutReader(_offer);
            }

            public void Render(ImmediateDrawingContext context)
            {
                var feature = context.TryGetFeature<ISkiaSharpApiLeaseFeature>();
                if (feature == null) return; // non-Skia backend - nothing to draw through

                using ISkiaSharpApiLease lease = feature.Lease();
                RenderTo(lease.SkCanvas, lease.GrContext);
            }

            internal void RenderTo(SKCanvas canvas, GRContext? grContext)
            {
                lock (_owner._cacheLock)
                {
                    long started = System.Diagnostics.Stopwatch.GetTimestamp();
                    Offer source;
                    bool fresh;
                    lock (_owner._offerLock)
                    {
                        // In one hold with the read taken, or a Dispose between the two could give the array back first.
                        if (_disposed) return;
                        source = _owner.BeginRead(_offer, out fresh);
                    }
                    try
                    {
                        // A built-in filter or simple effect is drawn by its chain on the device and read back; until that is built, the plain picture - see EmuSen_DevicePicture.md §19.
                        ScreenFilter? builtIn = _owner._slang is null ? _owner._activeFilter ?? (_effect == ShaderEffect.None ? null : BuiltInShaders.AsFilter(_effect, _owner.BilinearFiltering)) : null;
                        Slang.SlangRunner? runner = builtIn is null ? null : _owner.BuiltInFor(source, ref builtIn);

                        // A preset or a chain that will draw reads the array itself, so no image of the source is made, and one made earlier is now stale - see EmuSen_Serenity.md §9.3.
                        bool presetOnly = _owner._slang is { Ready: true } || runner is { Ready: true };
                        if (presetOnly && fresh) { _owner._cachedImage?.Dispose(); _owner._cachedImage = null; }
                        bool copy = !presetOnly && (fresh || _owner._cachedImage is null);
                        if (copy) CopySource(source);
                        if (fresh) _owner.Copied(source);
                        long copied = System.Diagnostics.Stopwatch.GetTimestamp();

                        if (_owner._slang is { } slang && DrawSlang(canvas, slang, presetOnly ? fresh : copy, source)) { }
                        else if (runner is { Ready: true } && DrawBuiltIn(canvas, runner, builtIn!, source)) { }
                        else
                        {
                            if (_owner._cachedImage is null) { CopySource(source); copy = true; }
                            Draw(canvas, _owner._cachedImage!, source);
                        }

                        // Flushed here so the texture's upload, which Skia defers to a flush, is timed with the draw - see EmuSen_Serenity.md §2.5.
                        Slang.SlangProbe.Current?.Phase(Slang.SlangProbe.FlushBegin);
                        grContext?.Flush();
                        Slang.SlangProbe.Current?.Phase(Slang.SlangProbe.FlushEnd);
                        _owner.Presented(copy, copied - started, System.Diagnostics.Stopwatch.GetTimestamp() - copied, grContext is not null, source.Width, source.Height);
                        _owner.SampleDisplay(started, source.Sequence, fresh);
                    }
                    finally
                    {
                        _owner.PutReader(source);
                    }
                }
            }

            private void CopySource(Offer source)
            {
                var sourceInfo = new SKImageInfo(source.Width, source.Height, SKColorType.Rgba8888, SKAlphaType.Unpremul);
                SKImage? previous = _owner._cachedImage;
                _owner._cachedImage = SKImage.FromPixelCopy(sourceInfo, source.Rgba);
                previous?.Dispose();
            }

            // A filter that states a shape gets it; otherwise the screen's the core reports, unless square pixels are asked for - see EmuSen_Serenity.md §2.9 and §3.9.
            private SKRect Destination(Offer source, double? filterAspect = null)
            {
                double aspect = filterAspect ?? (_owner._squarePixels ? 0 : source.Aspect);
                // With no shape given the part kept decides it, its pixels square - see EmuSen_Serenity.md §2.10.
                PictureCrop crop = _owner._crop;
                var (x, y, w, h) = ComputeLetterboxRect(source.Width * crop.Width, source.Height * source.RowRepeat * crop.Height, aspect, Bounds.Width, Bounds.Height);
                return new SKRect((float)x, (float)y, (float)x + Math.Max(1, (int)Math.Round(w)), (float)y + Math.Max(1, (int)Math.Round(h)));
            }

            // The whole frame's rectangle for the part kept to fill the one shown, drawing clipped to that one; false when nothing is hidden - see EmuSen_Serenity.md §2.10.
            private bool BeginCrop(SKCanvas canvas, ref SKRect destination)
            {
                PictureCrop crop = _owner._crop;
                if (crop.IsNone) return false;
                canvas.Save();
                canvas.ClipRect(destination);
                var (x, y, w, h) = crop.Whole(destination.Left, destination.Top, destination.Width, destination.Height);
                destination = new SKRect((float)x, (float)y, (float)(x + w), (float)(y + h));
                return true;
            }

            private bool DrawSlang(SKCanvas canvas, Slang.SlangRunner slang, bool newFrame, Offer source)
            {
                SKRect destination = Destination(source);
                bool cropped = BeginCrop(canvas, ref destination);
                bool drawn = slang.Draw(canvas, source.Rgba, source.Width, source.Height, newFrame, destination);
                if (cropped) canvas.Restore();
                return drawn;
            }

            // The filter's chain rendered for the rectangle in the display's pixels and read back, then drawn there; a filter that models the screen places a crop itself - see EmuSen_DevicePicture.md §19.
            private bool DrawBuiltIn(SKCanvas canvas, Slang.SlangRunner runner, ScreenFilter filter, Offer source)
            {
                SKRect destination = Destination(source, filter.Aspect);
                bool cropped = !filter.CropsItself && BeginCrop(canvas, ref destination);
                SKMatrix total = canvas.TotalMatrix;
                var origin = new Point(total.TransX + destination.Left * total.ScaleX, total.TransY + destination.Top * total.ScaleY);
                byte[]? raw = _owner.RawImage(source, filter);
                _owner.GiveValues(runner, filter, origin, filter.CropsItself ? _owner._crop : PictureCrop.None, raw is null ? 0 : source.Width, raw is null ? 0 : source.Height);
                bool drawn = runner.Draw(canvas, source.Rgba, source.Width, source.Height, source.Version != _owner._builtInVersion, destination, filter.RowsOnce ? 1 : source.RowRepeat, raw);
                if (drawn) _owner._builtInVersion = source.Version;
                if (cropped) canvas.Restore();
                return drawn;
            }

            private void Draw(SKCanvas canvas, SKImage sourceImage, Offer source)
            {
                // The control's BilinearFiltering - see EmuSen_Settings_Reference.md §3.
                SKSamplingOptions sampling = _owner.BilinearFiltering
                    ? new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.None)
                    : new SKSamplingOptions(SKFilterMode.Nearest, SKMipmapMode.None);

                // While a chosen filter is built the plain picture takes its shape, so nothing moves when it takes over.
                SKRect destRect = Destination(source, _owner._activeFilter?.Aspect);
                bool cropped = BeginCrop(canvas, ref destRect);
                canvas.DrawImage(sourceImage, destRect, sampling);
                if (cropped) canvas.Restore();
            }
        }
    }
}

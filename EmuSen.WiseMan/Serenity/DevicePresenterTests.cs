using System;
using System.Linq;
using EmuSen.Serenity;
using EmuSen.Serenity.Slang;
using Avalonia.Platform;
using Avalonia.Rendering.Composition;

namespace EmuSen.WiseMan.Serenity
{
    // When the picture is handed to the compositor on the device and when it is read back: §4.4's conditions, each alone - see EmuSen_DevicePicture.md §4.4.
    public class DevicePresenterTests
    {
        private static readonly string[] Fd = { KnownPlatformGraphicsExternalImageHandleTypes.VulkanOpaquePosixFileDescriptor };
        private static readonly string[] SemaphoreFd = { KnownPlatformGraphicsExternalSemaphoreHandleTypes.VulkanOpaquePosixFileDescriptor };
        private static readonly byte[] Card = { 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
        private static readonly byte[] Other = { 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
        private const CompositionGpuImportedImageSynchronizationCapabilities Semaphores = CompositionGpuImportedImageSynchronizationCapabilities.Semaphores;

        [Fact]
        public void Every_condition_must_hold_for_the_device_path_and_each_alone_refuses_it()
        {
            Assert.Null(DevicePresenter.Refusal(Fd, SemaphoreFd, Semaphores, Card, (Card, true)));
            Assert.Equal("the compositor offers no GPU interop", DevicePresenter.Refusal(null, null, default, null, (Card, true)));
            Assert.Equal("the compositor imports no Vulkan image by file descriptor",
                DevicePresenter.Refusal(new[] { KnownPlatformGraphicsExternalImageHandleTypes.D3D11TextureNtHandle }, SemaphoreFd, Semaphores, Card, (Card, true)));
            Assert.Equal("the compositor imports no Vulkan semaphore by file descriptor", DevicePresenter.Refusal(Fd, Array.Empty<string>(), Semaphores, Card, (Card, true)));
            Assert.Equal("the compositor cannot synchronise an import by semaphores",
                DevicePresenter.Refusal(Fd, SemaphoreFd, CompositionGpuImportedImageSynchronizationCapabilities.KeyedMutex, Card, (Card, true)));
            Assert.Equal("the Vulkan device cannot export its images", DevicePresenter.Refusal(Fd, SemaphoreFd, Semaphores, Card, (Card, false)));
            Assert.Equal("the compositor draws on another device", DevicePresenter.Refusal(Fd, SemaphoreFd, Semaphores, Other, (Card, true)));
            Assert.Equal("the compositor draws on another device", DevicePresenter.Refusal(Fd, SemaphoreFd, Semaphores, null, (Card, true)));
        }

        // The library's device says its UUID and that it exports on the RX 6800; the C#'s exports nothing, so a frontend switched to it reads back.
        [Fact]
        public void The_library_s_device_names_its_card_and_exports_and_the_csharp_s_does_not()
        {
            using SlangVulkan? native = SlangVulkan.TryCreate(out _, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800", null, native: true);
            if (native is null) return;
            var (uuid, exports) = native.Identity();
            Assert.True(exports, "RADV offers the two external extensions");
            Assert.Contains(uuid, b => b != 0);
            using SlangVulkan? managed = SlangVulkan.TryCreate(out _, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800", null, native: false);
            if (managed is null) return;
            Assert.False(managed.Identity().Exports);
            Assert.Null(SlangShown.TryCreate(managed, 4, 4, true, out string? problem));
            Assert.NotNull(problem);
        }

        // An exported target is made on the library's device at either layout, with three descriptors, and freed.
        [Fact]
        public void A_target_is_made_with_its_memory_and_two_semaphores_as_descriptors()
        {
            using SlangVulkan? device = SlangVulkan.TryCreate(out _, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800", null, native: true);
            if (device is null) return;
            foreach (bool linear in new[] { true, false })
            {
                using SlangShown shown = SlangShown.TryCreate(device, 640, 480, linear, out string? problem) ?? throw new InvalidOperationException(problem);
                Assert.Equal((640, 480), (shown.Width, shown.Height));
                Assert.True(shown.MemoryBytes >= 640 * 480 * 4, $"{shown.MemoryBytes} bytes");
                Assert.True(shown.MemoryFd >= 0 && shown.ReadyFd >= 0 && shown.FreeFd >= 0);
                Assert.Equal(3, new[] { shown.MemoryFd, shown.ReadyFd, shown.FreeFd }.Distinct().Count());
            }
        }
    }
}

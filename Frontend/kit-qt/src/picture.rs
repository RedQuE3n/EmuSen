//! The game picture's two surfaces in Rust (EmuSen_Kit.md §7.16): the feed a host hands `GamePicture`, laid out as
//! `cpp/game_picture.h` declares it, and the window's scene graph made on a device the host made.

use core::ffi::c_void;

/// The image a feed drew: Vulkan's `VkImage` as an integer, its size, `VkFormat` and `VkImageLayout`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Picture {
    pub image: u64,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub layout: u32,
}

/// A host's picture. `draw` is called on the render thread at the item's sync, 1 with `out` filled and anything else
/// for nothing to show; `swapped` on the render thread after each swap, with the steady clock's nanoseconds.
#[repr(C)]
pub struct Feed {
    pub context: *mut c_void,
    pub draw: extern "C" fn(context: *mut c_void, width: i32, height: i32, out: *mut Picture) -> i32,
    pub swapped: extern "C" fn(context: *mut c_void, nanoseconds: i64),
}

impl Feed {
    /// The number `GamePicture.feed` takes: the feed's address, which must stay valid while an item names it.
    pub fn address(&self) -> u64 {
        core::ptr::from_ref(self) as u64
    }
}

/// Vulkan's handles a window's scene graph adopts, and the instance's extensions as Qt is told them.
pub struct Adopted<'a> {
    pub instance: u64,
    pub physical: u64,
    pub device: u64,
    pub queue_family: u32,
    pub queue_index: u32,
    pub extensions: &'a [&'a str],
}

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("game_picture.h");
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qstringlist.h");
        type QString = cxx_qt_lib::QString;
        type QStringList = cxx_qt_lib::QStringList;
        include!(<QtQuick/QQuickWindow>);
        type QQuickWindow;

        unsafe fn kit_adopt_vulkan(
            window: Pin<&mut QQuickWindow>,
            instance: u64,
            physical: u64,
            device: u64,
            family: u32,
            index: u32,
            extensions: &QStringList,
        ) -> QString;
    }
}

/// The window's scene graph put on the adopted device, before the window is first shown; the words when Qt refuses.
///
/// # Safety
/// `window` is a live `QQuickWindow` that has never been shown, and the handles are one live device's.
pub unsafe fn adopt(window: *mut c_void, adopted: &Adopted<'_>) -> Result<(), String> {
    let mut names = cxx_qt_lib::QList::<cxx_qt_lib::QString>::default();
    for name in adopted.extensions {
        names.append(cxx_qt_lib::QString::from(*name));
    }
    let names = cxx_qt_lib::QStringList::from(&names);
    let window = unsafe { core::pin::Pin::new_unchecked(&mut *window.cast::<ffi::QQuickWindow>()) };
    let words = unsafe {
        ffi::kit_adopt_vulkan(window, adopted.instance, adopted.physical, adopted.device, adopted.queue_family, adopted.queue_index, &names)
    };
    if words.is_empty() {
        Ok(())
    } else {
        Err(words.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_feed_and_the_picture_are_laid_out_as_the_header_declares() {
        assert_eq!(size_of::<Picture>(), 24);
        assert_eq!(core::mem::offset_of!(Picture, layout), 20);
        assert_eq!(size_of::<Feed>(), 3 * size_of::<usize>());
        assert_eq!(core::mem::offset_of!(Feed, swapped), 2 * size_of::<usize>());
    }
}

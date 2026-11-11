//! `ResetGate`: whether a list model is settled, false from its `modelAboutToBeReset` to its `modelReset`, so a view can
//! let the model go across a reset (EmuSen_Kit.md §7.58).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractItemModel>);
        type QAbstractItemModel = crate::item_model::ffi::QAbstractItemModel;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(*mut QAbstractItemModel, model, READ, WRITE = set_model, NOTIFY)]
        #[qproperty(bool, settled, READ, NOTIFY)]
        type ResetGate = super::ResetGateRust;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "setModel"]
        unsafe fn set_model(self: Pin<&mut ResetGate>, model: *mut QAbstractItemModel);
    }
}

use core::pin::Pin;
use cxx_qt::{CxxQtType, QMetaObjectConnectionGuard};

pub struct ResetGateRust {
    model: *mut qobject::QAbstractItemModel,
    settled: bool,
    heard: Vec<QMetaObjectConnectionGuard>,
}

impl Default for ResetGateRust {
    fn default() -> Self {
        Self { model: core::ptr::null_mut(), settled: true, heard: Vec::new() }
    }
}

/// The gate as its model's two connections reach it; they are the gate's own guards, dropped before the gate is.
#[derive(Clone, Copy)]
struct Gate(*mut qobject::ResetGate);

// SAFETY: the connections are direct, made and heard on the gate's thread.
unsafe impl Send for Gate {}

impl Gate {
    fn hear(self, settled: bool) {
        // SAFETY: the gate outlives the connections it holds (see `Gate`).
        unsafe { Pin::new_unchecked(&mut *self.0) }.set_settled_to(settled);
    }
}

impl qobject::ResetGate {
    /// Hears the new model's resets in place of the old one's; a model handed over is settled, even one handed over
    /// from inside the old model's reset, whose end this gate no longer hears.
    ///
    /// # Safety
    /// `model` is null or a live model, as QML hands it.
    pub unsafe fn set_model(mut self: Pin<&mut Self>, model: *mut qobject::QAbstractItemModel) {
        if self.model == model {
            return;
        }
        let gate = Gate(unsafe { self.as_mut().get_unchecked_mut() } as *mut Self);
        let mut heard = Vec::new();
        if !model.is_null() {
            // SAFETY: the caller hands a live model; Qt drops each connection with the model.
            let mut target = unsafe { Pin::new_unchecked(&mut *model) };
            heard.push(target.as_mut().on_model_about_to_be_reset(move |_| gate.hear(false)));
            heard.push(target.on_model_reset(move |_| gate.hear(true)));
        }
        let mut rust = self.as_mut().rust_mut();
        rust.heard = heard;
        rust.model = model;
        self.as_mut().model_changed();
        self.set_settled_to(true);
    }

    fn set_settled_to(mut self: Pin<&mut Self>, settled: bool) {
        if self.settled != settled {
            self.as_mut().rust_mut().settled = settled;
            self.settled_changed();
        }
    }
}

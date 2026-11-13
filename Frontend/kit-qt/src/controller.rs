//! `ControllerLayout`: kit-core's controller drawing on Qt (EmuSen_Kit.md §7.62): a drawing file and the controls a
//! host declares in, the drawing's place and scale, what is painted, the words printed, each control's label box and
//! the leaders out, read by `ControllerDrawing.qml`.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QList, QRectF, QString, QStringList, QVariant};
use emusen_kit_core::controller::{data_url, drawing_for, knob, lay_out, layout, paints, svg, walk, Drawing, Laid, Lit};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qrectf.h");
        type QRectF = cxx_qt_lib::QRectF;
        include!("cxx-qt-lib/qlist.h");
        type QList_f64 = cxx_qt_lib::QList<f64>;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, drawing)]
        #[qproperty(QStringList, controls)]
        #[qproperty(i32, focused)]
        #[qproperty(QStringList, held)]
        /// How far each control is pushed, 0 to 1, in the controls' order: a stick's directions place its knob.
        #[qproperty(QList_f64, pushes)]
        #[qproperty(f64, width)]
        #[qproperty(f64, height)]
        #[qproperty(f64, label_height)]
        #[qproperty(QColor, accent)]
        /// What each label says beside its control's name, in the controls' order.
        #[qproperty(QStringList, texts)]
        #[qproperty(QString, name, READ, NOTIFY = laid_out)]
        #[qproperty(bool, generic, READ, NOTIFY = laid_out)]
        #[qproperty(bool, fits, READ, NOTIFY = laid_out)]
        #[qproperty(QRectF, place, READ, NOTIFY = laid_out)]
        #[qproperty(f64, scale, READ, NOTIFY = laid_out)]
        #[qproperty(f64, design_width, READ, NOTIFY = laid_out)]
        #[qproperty(f64, design_height, READ, NOTIFY = laid_out)]
        #[qproperty(QString, leaders, READ, NOTIFY = laid_out)]
        #[qproperty(QString, focus_leader, READ, NOTIFY = laid_out)]
        /// Each label's words (its region's name, then its text) and its box as x, y, width and height in turn.
        #[qproperty(QStringList, label_words, READ, NOTIFY = worded)]
        #[qproperty(QVariant, label_boxes, READ, NOTIFY = laid_out)]
        /// The drawing as a picture, an SVG document in a `data:` URL, drawn at `place`.
        #[qproperty(QString, picture, READ, NOTIFY = painted)]
        type ControllerLayout = super::ControllerLayoutRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// The place, the labels and the leaders changed.
        #[qsignal]
        fn laid_out(self: Pin<&mut ControllerLayout>);

        /// The labels' words changed: the drawing's names or the host's texts, never a layout alone.
        #[qsignal]
        fn worded(self: Pin<&mut ControllerLayout>);

        /// What is painted changed: the focus, the held controls or the accent.
        #[qsignal]
        fn painted(self: Pin<&mut ControllerLayout>);

        /// A label measured: its width, which the layout places it by.
        #[qinvokable]
        fn set_label_width(self: Pin<&mut ControllerLayout>, index: i32, width: f64);

        #[qinvokable]
        fn label_rect(self: &ControllerLayout, index: i32) -> QRectF;

        /// What a control's label and a reader call it: its region's name on the drawing.
        #[qinvokable]
        fn label_name(self: &ControllerLayout, index: i32) -> QString;

        /// The control the walk focuses next from the one focused, by -1 or +1, round from either end.
        #[qinvokable]
        fn step(self: &ControllerLayout, by: i32) -> i32;
    }

    impl cxx_qt::Initialize for ControllerLayout {}
}

/// The drawing read for the controls, laid out in its box, with what is painted now.
pub struct ControllerLayoutRust {
    drawing: QString,
    controls: QStringList,
    focused: i32,
    held: QStringList,
    pushes: QList<f64>,
    width: f64,
    height: f64,
    label_height: f64,
    accent: QColor,
    texts: QStringList,
    name: QString,
    generic: bool,
    fits: bool,
    place: QRectF,
    scale: f64,
    design_width: f64,
    design_height: f64,
    leaders: QString,
    focus_leader: QString,
    label_words: QStringList,
    label_boxes: QVariant,
    picture: QString,
    read: Drawing,
    names: Vec<String>,
    widths: Vec<f64>,
    laid: Laid,
}

impl Default for ControllerLayoutRust {
    fn default() -> Self {
        ControllerLayoutRust {
            drawing: QString::default(),
            controls: QStringList::default(),
            focused: -1,
            held: QStringList::default(),
            pushes: QList::default(),
            width: 0.0,
            height: 0.0,
            label_height: 30.0,
            accent: QColor::from_rgb(0x4C, 0x9A, 0xE8),
            texts: QStringList::default(),
            name: QString::default(),
            generic: true,
            fits: false,
            place: QRectF::default(),
            scale: 0.0,
            design_width: 0.0,
            design_height: 0.0,
            leaders: QString::default(),
            focus_leader: QString::default(),
            label_words: QStringList::default(),
            label_boxes: QVariant::default(),
            picture: QString::default(),
            read: Drawing::default(),
            names: Vec::new(),
            widths: Vec::new(),
            laid: Laid::default(),
        }
    }
}

fn strings(list: &QStringList) -> Vec<String> {
    cxx_qt_lib::QList::<QString>::from(list).iter().map(ToString::to_string).collect()
}

fn at<T>(list: &[T], index: i32) -> Option<&T> {
    usize::try_from(index).ok().and_then(|i| list.get(i))
}

impl cxx_qt::Initialize for qobject::ControllerLayout {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_drawing_changed(|l| l.read_drawing()).release();
        self.as_mut().on_controls_changed(|l| l.read_drawing()).release();
        self.as_mut().on_width_changed(|l| l.lay()).release();
        self.as_mut().on_height_changed(|l| l.lay()).release();
        self.as_mut().on_label_height_changed(|l| l.lay()).release();
        self.as_mut().on_focused_changed(|l| l.lay()).release();
        self.as_mut().on_held_changed(|l| l.paint()).release();
        self.as_mut().on_pushes_changed(|l| l.paint()).release();
        self.as_mut().on_accent_changed(|l| l.paint()).release();
        self.as_mut().on_texts_changed(|l| l.word()).release();
    }
}

impl qobject::ControllerLayout {
    /// The file read again for the controls: its own drawing where it covers them, else the generic one.
    fn read_drawing(mut self: Pin<&mut Self>) {
        let controls = strings(&self.controls);
        let file = self.drawing.to_string();
        let (drawing, generic) = drawing_for(Some(file.as_str()).filter(|f| !f.trim().is_empty()), &controls);
        let shown = layout::shown(&drawing, &controls);
        let names = shown.iter().zip(&controls).map(|(r, c)| r.map_or_else(|| c.clone(), |r| drawing.regions[r].name.clone())).collect();
        {
            let mut r = self.as_mut().rust_mut();
            r.names = names;
            // A label keeps the width it was measured at until it says otherwise; its words change with the drawing.
            r.widths.resize(controls.len(), 0.0);
            r.generic = generic;
            r.name = QString::from(drawing.name.as_str());
            r.design_width = drawing.width;
            r.design_height = drawing.height;
            r.read = drawing;
        }
        self.as_mut().word();
        self.lay();
    }

    /// Each label's words, its region's name then the host's text for it, told only when they change.
    fn word(mut self: Pin<&mut Self>) {
        let texts = strings(&self.texts);
        let words = self.names.iter().enumerate().map(|(i, n)| match texts.get(i).filter(|t| !t.is_empty()) {
            Some(t) => format!("{n}  {t}"),
            None => n.clone(),
        });
        let words: QList<QString> = words.map(|w| QString::from(w.as_str())).collect();
        let words = QStringList::from(&words);
        if words != self.label_words {
            self.as_mut().rust_mut().label_words = words;
            self.as_mut().worded();
        }
    }

    fn lay(mut self: Pin<&mut Self>) {
        let controls = strings(&self.controls);
        let laid = lay_out(&self.read, &controls, &self.widths, self.label_height, self.width, self.height);
        let focus_leader = usize::try_from(self.focused).map(|i| laid.leader(i)).unwrap_or_default();
        let boxes: QList<f64> = laid.labels.iter().flat_map(|b| [b.x, b.y, b.w, b.h]).collect();
        {
            let mut r = self.as_mut().rust_mut();
            r.place = QRectF::new(laid.drawing.x, laid.drawing.y, laid.drawing.w, laid.drawing.h);
            r.scale = laid.scale;
            r.fits = laid.fits;
            r.leaders = QString::from(laid.leaders.as_str());
            r.focus_leader = QString::from(focus_leader.as_str());
            r.label_boxes = QVariant::from(&boxes);
            r.laid = laid;
        }
        self.as_mut().laid_out();
        self.paint();
    }

    fn paint(mut self: Pin<&mut Self>) {
        let controls = strings(&self.controls);
        let held = strings(&self.held);
        let focused = at(&controls, self.focused).cloned();
        let accent = format!("#{:02X}{:02X}{:02X}", self.accent.red(), self.accent.green(), self.accent.blue());
        let drawing = &self.read;
        let lit = |i: usize| {
            let control = &drawing.regions[i].control;
            Lit { focused: focused.as_ref() == Some(control), held: held.contains(control) }
        };
        let pushes: Vec<f64> = self.pushes.iter().copied().collect();
        let push = |control: &str| controls.iter().position(|c| c == control).and_then(|i| pushes.get(i)).copied().unwrap_or(0.0);
        let knobs = |k: usize| drawing.sticks.get(k).map_or((0.0, 0.0), |s| knob(s, &push));
        let painted = paints(drawing, &lit, &knobs, &accent);
        let picture = if drawing.width > 0.0 { data_url("image/svg+xml;utf8", &svg(drawing, &painted)) } else { String::new() };
        self.as_mut().rust_mut().picture = QString::from(picture.as_str());
        self.as_mut().painted();
    }

    fn set_label_width(mut self: Pin<&mut Self>, index: i32, width: f64) {
        let Ok(i) = usize::try_from(index) else { return };
        if self.widths.get(i).is_none_or(|w| (w - width).abs() < 0.01) {
            return;
        }
        self.as_mut().rust_mut().widths[i] = width;
        self.lay();
    }

    fn label_rect(&self, index: i32) -> QRectF {
        at(&self.laid.labels, index).map_or_else(QRectF::default, |b| QRectF::new(b.x, b.y, b.w, b.h))
    }

    fn label_name(&self, index: i32) -> QString {
        at(&self.names, index).map_or_else(QString::default, |n| QString::from(n.as_str()))
    }

    fn step(&self, by: i32) -> i32 {
        let count = cxx_qt_lib::QList::<QString>::from(&self.controls).len() as usize;
        let at = usize::try_from(self.focused).unwrap_or(0);
        i32::try_from(walk(at, by, count)).unwrap_or(0)
    }
}

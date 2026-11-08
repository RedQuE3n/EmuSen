//! `QuestionModel`, a question's words from question.rs for the kit's `Question`, so no screen writes them in QML: the
//! Resume question for the game it names.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// The game the Resume question is asked about, by its name.
        #[qproperty(QString, subject, READ, WRITE = set_subject, NOTIFY)]
        #[qproperty(QString, title, READ, NOTIFY)]
        #[qproperty(QStringList, sentences, READ, NOTIFY)]
        #[qproperty(QString, caption, READ, NOTIFY)]
        #[qproperty(QStringList, choices, READ, NOTIFY)]
        #[qproperty(i32, back_index, cxx_name = "backIndex", READ, NOTIFY)]
        type QuestionModel = super::QuestionModelRust;

        #[cxx_name = "setSubject"]
        fn set_subject(self: Pin<&mut QuestionModel>, subject: QString);
    }
}

use crate::question::{back_index, resume};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList};

pub struct QuestionModelRust {
    subject: QString,
    title: QString,
    sentences: QStringList,
    caption: QString,
    choices: QStringList,
    back_index: i32,
}

impl Default for QuestionModelRust {
    fn default() -> Self {
        let mut model = QuestionModelRust {
            subject: QString::default(),
            title: QString::default(),
            sentences: QStringList::default(),
            caption: QString::default(),
            choices: QStringList::default(),
            back_index: -1,
        };
        model.fill();
        model
    }
}

fn list(words: &[String]) -> QStringList {
    let items: QList<QString> = words.iter().map(|w| QString::from(w.as_str())).collect();
    QStringList::from(&items)
}

impl QuestionModelRust {
    fn fill(&mut self) {
        let question = resume(&self.subject.to_string());
        self.title = QString::from(question.title.as_str());
        self.sentences = list(&question.sentences);
        self.caption = QString::from(question.caption.as_str());
        self.choices = list(&question.choices);
        self.back_index = back_index(&question);
    }
}

impl qobject::QuestionModel {
    pub fn set_subject(mut self: Pin<&mut Self>, subject: QString) {
        if self.subject == subject {
            return;
        }
        self.as_mut().rust_mut().subject = subject;
        self.as_mut().rust_mut().fill();
        self.as_mut().subject_changed();
        self.as_mut().title_changed();
        self.as_mut().sentences_changed();
        self.as_mut().caption_changed();
        self.as_mut().choices_changed();
        self.back_index_changed();
    }
}

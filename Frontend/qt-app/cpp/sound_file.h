// EmuSen.App.SoundFile: a theme's sound read whole through Qt Multimedia's decoder, from a file or the resources, in the format a player asks (EmuSen_Qt.md §3.5.3).
#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QFile>
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtCore/QUrl>
#include <QtMultimedia/QAudioDecoder>
#include <QtQml/qqmlregistration.h>

#include <memory>

class SoundFile : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QUrl source READ source WRITE setSource NOTIFY sourceChanged)
    // The samples' rate and channels, 32-bit float interleaved; the source is read once both are set.
    Q_PROPERTY(int rate READ rate WRITE setRate NOTIFY formatChanged)
    Q_PROPERTY(int channels READ channels WRITE setChannels NOTIFY formatChanged)
    // Read to its end, or failed with the words in `problem`; never for no source.
    Q_PROPERTY(bool read READ read NOTIFY readChanged)
    Q_PROPERTY(QString problem READ problem NOTIFY readChanged)
    // The samples' SHA-256 in hex, worked out when asked, so a reader can hold them to a sound's own.
    Q_PROPERTY(QString digest READ digest NOTIFY readChanged)

public:
    explicit SoundFile(QObject* parent = nullptr);
    ~SoundFile() override;

    QUrl source() const { return m_source; }
    void setSource(const QUrl& source);
    int rate() const { return m_rate; }
    void setRate(int rate);
    int channels() const { return m_channels; }
    void setChannels(int channels);
    bool read() const { return m_read; }
    QString problem() const { return m_problem; }
    QString digest() const;
    const QByteArray& samples() const { return m_samples; }

Q_SIGNALS:
    void sourceChanged();
    void formatChanged();
    void readChanged();

private:
    void start();
    void take();
    void finish(const QString& problem);

    QUrl m_source;
    int m_rate = 0;
    int m_channels = 0;
    bool m_read = false;
    QString m_problem;
    QByteArray m_samples;
    std::unique_ptr<QFile> m_file;
    std::unique_ptr<QAudioDecoder> m_decoder;
};

// The samples read, for the player that keeps them, and the words for why none were.
QByteArray soundFileSamples(const SoundFile& file);
QString soundFileProblem(const SoundFile& file);

#include "sound_file.h"

#include <QtCore/QCryptographicHash>
#include <QtMultimedia/QAudioBuffer>
#include <QtMultimedia/QAudioFormat>

SoundFile::SoundFile(QObject* parent)
    : QObject(parent)
{
}

SoundFile::~SoundFile() = default;

void SoundFile::setSource(const QUrl& source)
{
    if (source == m_source) {
        return;
    }
    m_source = source;
    emit sourceChanged();
    start();
}

void SoundFile::setRate(int rate)
{
    if (rate == m_rate) {
        return;
    }
    m_rate = rate;
    emit formatChanged();
    start();
}

void SoundFile::setChannels(int channels)
{
    if (channels == m_channels) {
        return;
    }
    m_channels = channels;
    emit formatChanged();
    start();
}

// A file on disk or in the resources, read by QFile, decoded by QAudioDecoder into the format asked (Qt 6, "QAudioDecoder").
void SoundFile::start()
{
    m_decoder.reset();
    m_file.reset();
    m_samples.clear();
    m_read = false;
    m_problem.clear();
    if (m_rate <= 0 || m_channels <= 0 || m_source.isEmpty()) {
        return;
    }
    const QString path = m_source.isLocalFile() ? m_source.toLocalFile() : m_source.scheme() == QLatin1String("qrc") ? QLatin1Char(':') + m_source.path() : QString();
    m_file = std::make_unique<QFile>(path);
    if (path.isEmpty() || !m_file->open(QIODevice::ReadOnly)) {
        finish(QStringLiteral("The theme's sound %1 could not be opened.").arg(m_source.toString()));
        return;
    }
    QAudioFormat format;
    format.setSampleRate(m_rate);
    format.setChannelCount(m_channels);
    format.setSampleFormat(QAudioFormat::Float);
    m_decoder = std::make_unique<QAudioDecoder>();
    // With no media backend `start` does nothing and neither signal comes (EmuSen_Qt.md §3.5.3).
    if (!m_decoder->isSupported()) {
        finish(QStringLiteral("The theme's sound %1 could not be read: %2").arg(m_source.toString(), m_decoder->errorString()));
        return;
    }
    m_decoder->setAudioFormat(format);
    m_decoder->setSourceDevice(m_file.get());
    connect(m_decoder.get(), &QAudioDecoder::bufferReady, this, &SoundFile::take);
    connect(m_decoder.get(), &QAudioDecoder::finished, this, [this] {
        take();
        finish(QString());
    });
    connect(m_decoder.get(), qOverload<QAudioDecoder::Error>(&QAudioDecoder::error), this, [this](QAudioDecoder::Error) {
        finish(QStringLiteral("The theme's sound %1 could not be read: %2").arg(m_source.toString(), m_decoder->errorString()));
    });
    m_decoder->start();
}

QString SoundFile::digest() const
{
    return QString::fromLatin1(QCryptographicHash::hash(m_samples, QCryptographicHash::Sha256).toHex());
}

void SoundFile::take()
{
    while (m_decoder && m_decoder->bufferAvailable()) {
        const QAudioBuffer buffer = m_decoder->read();
        m_samples.append(buffer.constData<char>(), buffer.byteCount());
    }
}

void SoundFile::finish(const QString& problem)
{
    if (m_read) {
        return;
    }
    if (!problem.isEmpty()) {
        m_samples.clear();
    }
    m_problem = problem;
    m_read = true;
    emit readChanged();
}

QByteArray soundFileSamples(const SoundFile& file)
{
    return file.samples();
}

QString soundFileProblem(const SoundFile& file)
{
    return file.problem();
}

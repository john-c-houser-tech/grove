#pragma once

#include <QFileSystemWatcher>
#include <QHash>
#include <QJsonValue>
#include <QLocale>
#include <QTimer>
#include <QTranslator>

// Follows the language picked under Omarchy's Setup > Language. Catalogs are
// flat JSON objects from English source text to translation, one per Omarchy
// language code (i18n/da.json, i18n/pt-BR.json), so QML keeps plain
// qsTr("Save") and C++ keeps tr("Save"). A counted string,
// qsTr("%n clips", "", count), maps to an object of plural forms keyed by
// CLDR category ({"one": "%n klip", "other": "%n klip"}).
class Translator : public QTranslator {
    Q_OBJECT
    Q_PROPERTY(QString language READ languageCode NOTIFY changed)
    // Whether the language reads right to left, so the window mirrors.
    Q_PROPERTY(bool rightToLeft READ rightToLeft NOTIFY changed)

public:
    // `languageFile` is where Omarchy keeps the language code (empty for
    // ~/.config/omarchy/language), watched for changes; `catalogDirectory`
    // holds the catalogs. Tests pass files of their own.
    explicit Translator(const QString &languageFile = {}, QString catalogDirectory = ":/i18n",
                        QObject *parent = nullptr);

    // The Omarchy language code ("en", "pt-BR"). Not QTranslator::language(),
    // which names the language of a compiled .qm catalog.
    QString languageCode() const { return m_language; }
    bool rightToLeft() const { return m_rightToLeft; }
    bool isEmpty() const override { return m_strings.isEmpty(); }
    QString translate(const char *context, const char *sourceText, const char *disambiguation = nullptr,
                      int n = -1) const override;

    // The registry code on the first line of the language file; anything
    // else, or no file, is English.
    static QString readLanguage(const QString &path);
    // The CLDR plural category ("one", "few", "other"...) of a whole count in
    // a language, by the same rules as Omarchy's shell/Commons/Plural.js.
    static QString pluralCategory(const QString &language, long long count);
    // The locale Omarchy formats the language with, from its registry, so
    // numbers and dates match the desktop's (Arabic there uses Western
    // digits); without Omarchy, Qt's own locale for the code.
    static QLocale formatLocale(const QString &language);

signals:
    // The language changed; QML retranslates, and problems reported from C++
    // are asked for again (see problem.h).
    void changed();

private:
    void reload();

    QString m_languageFile;
    QString m_catalogDirectory;
    QString m_language;
    bool m_rightToLeft = false;
    QHash<QString, QJsonValue> m_strings;
    QFileSystemWatcher m_watcher;
    QTimer m_debounce;
};

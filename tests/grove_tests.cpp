#include <QDir>
#include <QFile>
#include <QTemporaryDir>
#include <QtTest>

#include "backend.h"
#include "theme.h"
#include "translator.h"

class GroveTests : public QObject {
    Q_OBJECT

private slots:
    void startsIdle()
    {
        Backend backend(10);
        QCOMPARE(backend.state(), QString("idle"));
        QCOMPARE(backend.progress(), 0.0);
    }

    void growsWithTime()
    {
        Backend backend(10);
        backend.start();
        QCOMPARE(backend.state(), QString("growing"));
        backend.advance(2500);
        QCOMPARE(backend.progress(), 0.25);
        QCOMPARE(backend.remaining(), 8);
    }

    void finishesWhenTheTimeIsUp()
    {
        Backend backend(10);
        backend.start();
        backend.advance(10000);
        QCOMPARE(backend.state(), QString("done"));
        QCOMPARE(backend.progress(), 1.0);
        QCOMPARE(backend.remaining(), 0);
    }

    void givingUpWiltsThePlant()
    {
        Backend backend(10);
        backend.start();
        backend.advance(4000);
        backend.giveUp();
        QCOMPARE(backend.state(), QString("wilted"));
        // The plant keeps the size it reached, and time stops.
        backend.advance(4000);
        QCOMPARE(backend.progress(), 0.4);
    }

    void giveUpDoesNothingWhenIdle()
    {
        Backend backend(10);
        backend.giveUp();
        QCOMPARE(backend.state(), QString("idle"));
    }

    void startingAgainResets()
    {
        Backend backend(10);
        backend.start();
        backend.advance(5000);
        backend.giveUp();
        backend.start();
        QCOMPARE(backend.state(), QString("growing"));
        QCOMPARE(backend.progress(), 0.0);
    }

    void startingWhileGrowingKeepsTheSession()
    {
        Backend backend(10);
        backend.start();
        backend.advance(5000);
        backend.start();
        QCOMPARE(backend.progress(), 0.5);
    }

    void themeFallsBack()
    {
        QTemporaryDir dir;
        Theme theme(dir.path());
        QCOMPARE(theme.accent(), QString("#FFD60A"));
        QCOMPARE(theme.accentForeground(), QString("black"));
    }

    void themeReadsAccent()
    {
        QTemporaryDir dir;
        QDir(dir.path()).mkpath("theme");
        QFile colors(dir.path() + "/theme/colors.toml");
        QVERIFY(colors.open(QIODevice::WriteOnly));
        colors.write("accent = \"#112233\"\n");
        colors.close();

        Theme theme(dir.path());
        QCOMPARE(theme.accent(), QString("#112233"));
        QCOMPARE(theme.accentForeground(), QString("white"));
    }

    void translatorFallsBackToEnglish()
    {
        QTemporaryDir dir;
        Translator translator(dir.path() + "/language", dir.path());
        QCOMPARE(translator.language(), QString("en"));
        QVERIFY(translator.translate("Main", "Quit").isNull());
    }

    void translatorFollowsTheLanguageFile()
    {
        QTemporaryDir dir;
        write(dir.path() + "/language", "pt-BR\n");
        write(dir.path() + "/pt.json", R"({"Quit": "Sair", "Keys": "Teclas"})");
        write(dir.path() + "/pt-BR.json", R"({"Quit": "Fechar"})");

        Translator translator(dir.path() + "/language", dir.path());
        QCOMPARE(translator.language(), QString("pt-BR"));
        QCOMPARE(translator.translate("Main", "Quit"), QString("Fechar"));
        QCOMPARE(translator.translate("Main", "Keys"), QString("Teclas"));
    }

private:
    static void write(const QString &path, const QByteArray &contents)
    {
        QFile file(path);
        QVERIFY(file.open(QIODevice::WriteOnly));
        file.write(contents);
    }
};

QTEST_MAIN(GroveTests)
#include "grove_tests.moc"

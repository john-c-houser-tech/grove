#include <QDir>
#include <QFile>
#include <QTemporaryDir>
#include <QtTest>

#include "backend.h"
#include "light.h"
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

    void sunriseIsEarlierInSummer()
    {
        double summerRise = 0, summerSet = 0, winterRise = 0, winterSet = 0;
        Light::sunTimes(QDate(2026, 6, 21), summerRise, summerSet);
        Light::sunTimes(QDate(2026, 12, 21), winterRise, winterSet);
        QVERIFY(summerRise < winterRise);
        QVERIFY(summerSet > winterSet);
        QVERIFY(summerSet - summerRise > 15);
        QVERIFY(winterSet - winterRise < 9);
    }

    void noonIsBrightAndMidnightIsDark()
    {
        const QColor accent("#FFD60A");
        const Sky noon = Light::sky(QDateTime(QDate(2026, 6, 21), QTime(12, 30)), accent);
        const Sky midnight = Light::sky(QDateTime(QDate(2026, 6, 21), QTime(0, 30)), accent);
        QVERIFY(noon.sunUp);
        QVERIFY(!midnight.sunUp);
        QCOMPARE(noon.ambient, 1.0);
        QVERIFY(midnight.ambient < 0.3);
        QCOMPARE(noon.stars, 0.0);
        QCOMPARE(midnight.stars, 1.0);
        QVERIFY(noon.top.lightnessF() > midnight.top.lightnessF());
    }

    void lightTurnsGoldenAtSunset()
    {
        double rise = 0, set = 0;
        Light::sunTimes(QDate(2026, 11, 6), rise, set);
        const QColor accent("#FFD60A");
        auto at = [&](double hour) {
            return Light::sky(QDateTime(QDate(2026, 11, 6), QTime(int(hour), int((hour - int(hour)) * 60))), accent);
        };
        QVERIFY(at(set - 0.3).warmth > 0.5);
        QVERIFY(at(12.5).warmth < 0.05);
        QVERIFY(at(1.0).warmth < 0.05);
    }

    void moonSharesTheNightSky()
    {
        const Sky midnight = Light::sky(QDateTime(QDate(2026, 11, 6), QTime(0, 0)), QColor("#FFD60A"));
        QVERIFY(midnight.moonElevation > 0.5);
        QVERIFY(midnight.moonX > 0 && midnight.moonX < 1);
    }

    void fixedTimeFreezesTheLight()
    {
        Light light(QDateTime(QDate(2026, 6, 21), QTime(12, 30)));
        QVERIFY(light.sunUp());
        const QColor before = light.skyTop();
        light.setAccent(QColor("#112233"));
        QCOMPARE(light.skyTop(), before);
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

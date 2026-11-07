#include <QDir>
#include <QPainter>
#include <QFile>
#include <QTemporaryDir>
#include <QtTest>

#include "backend.h"
#include "field.h"
#include "garden.h"
#include "hills.h"
#include "layer.h"
#include "light.h"
#include "notifier.h"
#include "sounds.h"
#include "theme.h"
#include "trackbed.h"
#include "translator.h"
#include "weather.h"

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

    void fullSessionsAnnounceProgressOnceASecond()
    {
        Backend backend(25 * 60);
        backend.start();
        QSignalSpy spy(&backend, &Backend::progressChanged);
        // A second of 50 ms ticks: only the countdown moving on is worth a redraw.
        for (int i = 0; i < 20; ++i)
            backend.advance(50);
        QCOMPARE(spy.count(), 1);
        QCOMPARE(backend.remaining(), 25 * 60 - 1);
    }

    void shortSessionsStillGrowSmoothly()
    {
        Backend backend(10);
        backend.start();
        QSignalSpy spy(&backend, &Backend::progressChanged);
        for (int i = 0; i < 20; ++i)
            backend.advance(50);
        QCOMPARE(spy.count(), 20);
    }

    void theEndOfASessionIsAlwaysAnnounced()
    {
        Backend backend(25 * 60);
        backend.start();
        backend.advance(25 * 60 * 1000 - 10);
        QSignalSpy spy(&backend, &Backend::progressChanged);
        backend.advance(10);
        QCOMPARE(spy.count(), 1);
        QCOMPARE(backend.progress(), 1.0);
    }

    void aLayerKeepsOnlyTheRowsWithSomethingInThem()
    {
        const auto band = [](QPainter &p, double width, double) { p.fillRect(QRectF(0, 30, width, 20), Qt::red); };
        const PaintedLayer::Picture picture = PaintedLayer::draw(band, QSizeF(100, 100), -1, 1);
        QCOMPARE(picture.top, 30.0);
        QCOMPARE(picture.image.size(), QSize(100, 20));
    }

    void aLayerIsDrawnAtTheDisplaysScale()
    {
        const auto band = [](QPainter &p, double width, double) { p.fillRect(QRectF(0, 30, width, 20), Qt::red); };
        const PaintedLayer::Picture picture = PaintedLayer::draw(band, QSizeF(100, 100), -1, 2);
        QCOMPARE(picture.top, 30.0);
        QCOMPARE(picture.image.size(), QSize(200, 40));
        QCOMPARE(picture.image.devicePixelRatio(), 2.0);
    }

    void nothingIsDrawnWhereTheGroundCoversIt()
    {
        const auto all = [](QPainter &p, double width, double height) { p.fillRect(QRectF(0, 0, width, height), Qt::red); };
        QCOMPARE(PaintedLayer::draw(all, QSizeF(100, 100), 60, 1).image.size(), QSize(100, 60));
        // Nothing drawn, nothing kept.
        const auto none = [](QPainter &, double, double) {};
        QVERIFY(PaintedLayer::draw(none, QSizeF(100, 100), -1, 1).image.isNull());
    }

    void theFieldIsSeenFromStandingHeight()
    {
        Field field;
        field.setSize(QSizeF(800, 1000));
        // The far edge is 40 m off and the plant 4 m, seen from 1.6 m up.
        QCOMPARE(field.farEdgeY(), 600 + 1000 * 0.21 * 4 * 1.6 / 40);
        QCOMPARE(field.plantBaseY(), 600 + 1000 * 0.21 * 4 * 1.6 / 4);
    }

    void eachPartOfTheLandscapeKeepsToItsOwnBand()
    {
        const QSizeF size(400, 500);
        const double ground = 500 * 0.6 + 500 * 0.21 * 4 * 1.6 / 40;
        const auto field = [&](const QString &part) {
            const Field::Look look{0.6, ground, 0.5, 0.33, part, QColor("#2a2018"), QColor("#9fb7c4"), 0, false};
            return PaintedLayer::draw([look](QPainter &p, double w, double h) { Field::paint(p, look, w, h); }, size, -1, 1);
        };
        // The far field starts at its far edge and runs to the bottom.
        const PaintedLayer::Picture far = field("far");
        QVERIFY(qAbs(far.top - ground) < 1);
        QCOMPARE(far.top + far.image.height(), 500.0);
        // The rows nearer than the plant are only the bottom of the window.
        const PaintedLayer::Picture nearer = field("near");
        QVERIFY(nearer.top > ground + 100);

        // The hills stop where the ground covers them, and the near hills
        // are lower than the far range.
        const auto hills = [&](const QString &part) {
            const Hills::Look look{0.6, part, 0.6, QColor("#6b7f9c"), QColor("#46605a"), QColor("#30493b"),
                                   QColor("#9fb7c4"), QColor("#4d5a6c"), QColor("#2f3e35"), 1, 0.5, 0.3, QColor("#f2f6fa")};
            return PaintedLayer::draw([look](QPainter &p, double w, double h) { Hills::paint(p, look, w, h); }, size, ground, 1);
        };
        const PaintedLayer::Picture back = hills("back"), front = hills("front");
        QCOMPARE(back.top + back.image.height(), std::ceil(ground));
        QVERIFY(back.top > 500 * 0.6 - 500 * 0.36);
        QVERIFY(front.top > back.top + 500 * 0.15);
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

    void seasonsFollowTheCalendar()
    {
        QCOMPARE(Weather::seasonOf(QDate(2026, 4, 1)), QString("spring"));
        QCOMPARE(Weather::seasonOf(QDate(2026, 7, 1)), QString("summer"));
        QCOMPARE(Weather::seasonOf(QDate(2026, 11, 6)), QString("autumn"));
        QCOMPARE(Weather::seasonOf(QDate(2026, 1, 15)), QString("winter"));
    }

    void pinnedWeatherNeverChanges()
    {
        Weather weather("fog");
        weather.begin(5, QDate(2026, 11, 6));
        QCOMPARE(weather.kind(), QString("fog"));
        weather.setProgress(0.5);
        QCOMPARE(weather.kind(), QString("fog"));
        weather.setProgress(1.0);
        QCOMPARE(weather.kind(), QString("fog"));
        QVERIFY(weather.fog() > 0.8);
    }

    void sameSeedGivesTheSameWeather()
    {
        Weather a, b;
        a.begin(1234, QDate(2026, 11, 6));
        b.begin(1234, QDate(2026, 11, 6));
        for (double p : {0.0, 0.5, 1.0}) {
            a.setProgress(p);
            b.setProgress(p);
            QCOMPARE(a.kind(), b.kind());
            QCOMPARE(a.cloud(), b.cloud());
        }
    }

    void weatherCanChangeDuringASession()
    {
        // Some seed in a hundred must start and end differently.
        bool changed = false;
        for (int seed = 1; seed < 100 && !changed; ++seed) {
            Weather weather;
            weather.begin(seed, QDate(2026, 11, 6));
            weather.setProgress(0.0);
            const QString start = weather.kind();
            weather.setProgress(1.0);
            changed = weather.kind() != start;
        }
        QVERIFY(changed);
    }

    void autumnIsFoggierThanSummer()
    {
        auto share = [](const QString &season) {
            QRandomGenerator random(99);
            int fog = 0;
            for (int i = 0; i < 4000; ++i)
                fog += Weather::pickKind(random, season) == "fog";
            return fog / 4000.0;
        };
        QVERIFY(share("autumn") > share("summer") * 2);
    }

    void rainSpeedsGrowthButFinishesOnTime()
    {
        Weather weather("rain");
        weather.begin(1, QDate(2026, 11, 6));
        QCOMPARE(weather.growth(), 0.0);
        weather.setProgress(0.5);
        QVERIFY(weather.growth() > 0.5);
        QVERIFY(weather.growth() < 0.62);
        weather.setProgress(1.0);
        QCOMPARE(weather.growth(), 1.0);

        Weather dry("clear");
        dry.begin(1, QDate(2026, 11, 6));
        dry.setProgress(0.5);
        QVERIFY(qAbs(dry.growth() - 0.5) < 0.01);
    }

    void growthNeverGoesBackwards()
    {
        Weather weather("rain");
        weather.begin(1, QDate(2026, 11, 6));
        double last = 0;
        for (int i = 0; i <= 100; ++i) {
            weather.setProgress(i / 100.0);
            QVERIFY(weather.growth() >= last);
            last = weather.growth();
        }
    }

    void growingDoesNotRedrawSteadyWeather()
    {
        Weather weather("rain");
        weather.begin(1, QDate(2026, 11, 6));
        QSignalSpy sky(&weather, &Weather::changed);
        QSignalSpy growth(&weather, &Weather::growthChanged);
        for (int i = 1; i <= 100; ++i)
            weather.setProgress(i / 100.0);
        QCOMPARE(sky.count(), 0);
        QCOMPARE(growth.count(), 100);
    }

    void changingWeatherIsAnnouncedInVisibleStepsAndSettlesExactly()
    {
        for (int seed = 1; seed < 100; ++seed) {
            Weather weather, jumped;
            weather.begin(seed, QDate(2026, 11, 6));
            jumped.begin(seed, QDate(2026, 11, 6));
            jumped.setProgress(1.0);
            if (jumped.kind() == weather.kind())
                continue;
            QSignalSpy sky(&weather, &Weather::changed);
            // A full session's worth of announced ticks.
            for (int i = 1; i <= 1500; ++i)
                weather.setProgress(i / 1500.0);
            QVERIFY(sky.count() > 5);
            QVERIFY(sky.count() <= 110);
            QCOMPARE(weather.kind(), jumped.kind());
            QCOMPARE(weather.cloud(), jumped.cloud());
            QCOMPARE(weather.rain(), jumped.rain());
            QCOMPARE(weather.fog(), jumped.fog());
            QCOMPARE(weather.snow(), jumped.snow());
            return;
        }
        QFAIL("no seed changed the weather");
    }

    void trainFillsItsLengthAndFadesInAndOut()
    {
        const QByteArray pcm = Sounds::synthesizeTrain(44100);
        QCOMPARE(pcm.size(), Sounds::trainMilliseconds() / 1000 * 44100 * 2);
        QVERIFY(peakOf(pcm, 0, 44100 * 44) > 15000);
        // No click at either end.
        QVERIFY(peakOf(pcm, 0, 200) < 300);
        QVERIFY(peakOf(pcm, pcm.size() / 2 - 200, pcm.size() / 2) < 300);
    }

    void trainHornsAreLoudestAsItArrives()
    {
        const QByteArray pcm = Sounds::synthesizeTrain(44100);
        // The first signal is far off, the second close, and the rails alone are quiet.
        const double rails = rmsOf(pcm, 0.5, 2.5), far = rmsOf(pcm, 3.2, 5.2), close = rmsOf(pcm, 16.2, 18.2);
        QVERIFY(far > rails * 2);
        QVERIFY(close > far * 1.5);
        // It has gone by well before the end.
        QVERIFY(rmsOf(pcm, 40, 42) < close * 0.3);
    }

    void planeIsMuchFainterThanTheTrain()
    {
        const QByteArray plane = Sounds::synthesizePlane(44100);
        const QByteArray train = Sounds::synthesizeTrain(44100);
        QCOMPARE(plane.size(), Sounds::planeMilliseconds() / 1000 * 44100 * 2);
        QVERIFY(peakOf(plane, 0, plane.size() / 2) > 3000);
        QVERIFY(peakOf(plane, 0, plane.size() / 2) < peakOf(train, 0, train.size() / 2) / 2);
    }

    void planePassesOverheadInTheMiddle()
    {
        const QByteArray pcm = Sounds::synthesizePlane(44100);
        const double middle = rmsOf(pcm, 13, 17), early = rmsOf(pcm, 2, 5), late = rmsOf(pcm, 25, 28);
        QVERIFY(middle > early * 3);
        QVERIFY(middle > late * 3);
    }

    void trainsHappenOneToThreeTimesAndPlanesZeroToTwo()
    {
        QRandomGenerator random(8);
        QSet<int> trains, planes;
        for (int i = 0; i < 500; ++i) {
            const int t = Sounds::eventCount(random, "train"), p = Sounds::eventCount(random, "plane");
            QVERIFY(t >= 1 && t <= 3);
            QVERIFY(p >= 0 && p <= 2);
            trains.insert(t);
            planes.insert(p);
        }
        QCOMPARE(trains.size(), 3);
        QCOMPARE(planes.size(), 3);
    }

    void eventsAreSpreadWithinTheSession()
    {
        QRandomGenerator random(4);
        for (int i = 0; i < 200; ++i) {
            const QList<double> times = Sounds::eventTimes(random, 3, 1500, 44);
            QCOMPARE(times.size(), 3);
            for (int j = 0; j < times.size(); ++j) {
                QVERIFY(times[j] >= 20);
                QVERIFY(times[j] + 44 <= 1500 - 20);
                if (j > 0)
                    QVERIFY(times[j] - times[j - 1] >= 44 + 90);
            }
        }
    }

    void tooShortASessionGetsNoSounds()
    {
        QRandomGenerator random(1);
        QVERIFY(Sounds::eventTimes(random, 2, 30, 44).isEmpty());
        // Room for one but not three: fewer come back rather than crowding.
        const QList<double> some = Sounds::eventTimes(random, 3, 200, 44);
        QVERIFY(some.size() >= 1 && some.size() < 3);
    }

    void soundsMuteIsRemembered()
    {
        Sounds sounds(false);
        QVERIFY(!sounds.muted());
        sounds.setMuted(true);
        QVERIFY(sounds.muted());
    }

    void aPlaneFlyoverShowsEvenWhenMuted()
    {
        Sounds sounds(false, "plane");
        sounds.setMuted(true);
        QSignalSpy spy(&sounds, &Sounds::planeFlyover);
        sounds.begin(1500);
        QVERIFY(spy.wait(6000));
        QCOMPARE(spy.first().at(1).toInt(), Sounds::planeMilliseconds());
        const int direction = spy.first().at(0).toInt();
        QVERIFY(direction == 1 || direction == -1);
    }

    void endingASessionFadesOutWhatIsPassing()
    {
        Sounds sounds(false, "train");
        QSignalSpy fading(&sounds, &Sounds::fadingOut);
        QSignalSpy trains(&sounds, &Sounds::trainPassing);
        sounds.begin(60);
        sounds.end();
        QCOMPARE(fading.count(), 2);
        // The train due three seconds in never comes.
        QTest::qWait(3500);
        QCOMPARE(trains.count(), 0);
    }

    void aTrainShowsEvenWhenMuted()
    {
        Sounds sounds(false, "train");
        sounds.setMuted(true);
        QSignalSpy spy(&sounds, &Sounds::trainPassing);
        sounds.begin(1500);
        QVERIFY(spy.wait(6000));
        QCOMPARE(spy.first().at(0).toInt(), Sounds::trainMilliseconds());
    }

    void noSoundsAtDawnOrDusk()
    {
        Sounds sounds(false);
        sounds.setPeriodSource([] { return QString(); });
        QSignalSpy spy(&sounds, &Sounds::planeFlyover);
        sounds.begin(1500);
        QVERIFY(!spy.wait(500));
    }

    void emptyGardenStartsEmpty()
    {
        QTemporaryDir dir;
        Garden garden(dir.filePath("garden.json"));
        QCOMPARE(garden.rowCount(), 0);
        QCOMPARE(garden.grown(), 0);
        QVERIFY(garden.problem().isEmpty());
        // Nothing is written until there is something to keep.
        QVERIFY(!QFile::exists(dir.filePath("garden.json")));
    }

    void recordedPlantsSurviveARestartNewestFirst()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("nested/garden.json");
        {
            Garden garden(path);
            garden.record(plant(11, "done", 1.0, 25));
            garden.record(plant(22, "wilted", 0.4, 0));
            garden.record(plant(33, "done", 1.0, 25));
            QCOMPARE(garden.grown(), 2);
            QCOMPARE(garden.wilted(), 1);
            QCOMPARE(garden.focusedMinutes(), 50);
            QCOMPARE(garden.data(garden.index(0), Garden::SeedRole).toInt(), 33);
        }
        Garden again(path);
        QCOMPARE(again.rowCount(), 3);
        QCOMPARE(again.data(again.index(0), Garden::SeedRole).toInt(), 33);
        QCOMPARE(again.data(again.index(1), Garden::OutcomeRole).toString(), QString("wilted"));
        QCOMPARE(again.data(again.index(1), Garden::ProgressRole).toDouble(), 0.4);
        QCOMPARE(again.data(again.index(2), Garden::SeedRole).toInt(), 11);
        QCOMPARE(again.focusedMinutes(), 50);
        QVERIFY(again.problem().isEmpty());
    }

    void twoGrovesOpenTogetherKeepEachOthersPlants()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("garden.json");
        Garden first(path), second(path);
        first.record(plant(11, "done", 1.0, 25));
        // The second read the garden before the first saved its plant.
        second.record(plant(22, "wilted", 0.4, 0));
        QCOMPARE(second.rowCount(), 2);
        QCOMPARE(second.data(second.index(0), Garden::SeedRole).toInt(), 22);
        QCOMPARE(second.data(second.index(1), Garden::SeedRole).toInt(), 11);
        first.record(plant(33, "done", 1.0, 25));
        Garden again(path);
        QCOMPARE(again.rowCount(), 3);
        QCOMPARE(again.focusedMinutes(), 50);
        QVERIFY(!QFile::exists(path + ".lock"));
    }

    void aGardenSpoiledWhileOpenIsNotSavedOver()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("garden.json");
        Garden garden(path);
        garden.record(plant(11, "done", 1.0, 25));
        write(path, "this is not json");
        garden.record(plant(22, "done", 1.0, 25));
        QVERIFY(!garden.problem().isEmpty());
        QFile file(path);
        QVERIFY(file.open(QIODevice::ReadOnly));
        QCOMPARE(file.readAll(), QByteArray("this is not json"));
        // The plant is still shown, though it could not be kept.
        QCOMPARE(garden.rowCount(), 2);
    }

    void unreadableGardenIsLeftUntouched()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("garden.json");
        write(path, "this is not json");
        Garden garden(path);
        QVERIFY(!garden.problem().isEmpty());
        garden.record(plant(5, "done", 1.0, 25));
        // The new plant shows in the window but must not replace the old file.
        QCOMPARE(garden.rowCount(), 1);
        QFile file(path);
        QVERIFY(file.open(QIODevice::ReadOnly));
        QCOMPARE(file.readAll(), QByteArray("this is not json"));
    }

    void gardenWithABadEntryIsNotOverwritten()
    {
        QTemporaryDir dir;
        const QString path = dir.filePath("garden.json");
        const QByteArray bad = R"({"version":1,"plants":[{"seed":1,"outcome":"sprouted"}]})";
        write(path, bad);
        Garden garden(path);
        QVERIFY(!garden.problem().isEmpty());
        garden.record(plant(5, "done", 1.0, 25));
        QFile file(path);
        QVERIFY(file.open(QIODevice::ReadOnly));
        QCOMPARE(file.readAll(), bad);
    }

    void snowOnlyFallsInWinter()
    {
        auto snowShare = [](const QString &season) {
            QRandomGenerator random(5);
            int snow = 0;
            for (int i = 0; i < 4000; ++i)
                snow += Weather::pickKind(random, season) == "snow";
            return snow / 4000.0;
        };
        QCOMPARE(snowShare("spring"), 0.0);
        QCOMPARE(snowShare("summer"), 0.0);
        QCOMPARE(snowShare("autumn"), 0.0);
        QVERIFY(snowShare("winter") > 0.15);
    }

    void snowIsSnowNotRain()
    {
        Weather weather("snow");
        weather.begin(1, QDate(2026, 1, 15));
        QCOMPARE(weather.kind(), QString("snow"));
        QVERIFY(weather.snow() > 0.8);
        QCOMPARE(weather.rain(), 0.0);
        // Snow does not hurry the plant along the way rain does.
        weather.setProgress(0.5);
        QVERIFY(qAbs(weather.growth() - 0.5) < 0.01);
    }

    void notifierSaysSoWhenThereIsNoService()
    {
        Notifier notifier("org.grove.NoSuchService");
        QVERIFY(notifier.problem().isEmpty());
        notifier.plantGrown();
        QTRY_VERIFY_WITH_TIMEOUT(!notifier.problem().isEmpty(), 5000);
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
    static int peakOf(const QByteArray &pcm, int from, int to)
    {
        const auto *samples = reinterpret_cast<const qint16 *>(pcm.constData());
        int peak = 0;
        for (int i = from; i < to && i < pcm.size() / 2; ++i)
            peak = qMax(peak, qAbs(int(samples[i])));
        return peak;
    }

    static double rmsOf(const QByteArray &pcm, double fromSeconds, double toSeconds)
    {
        const auto *samples = reinterpret_cast<const qint16 *>(pcm.constData());
        const int from = int(fromSeconds * 44100), to = int(toSeconds * 44100);
        double sum = 0;
        for (int i = from; i < to; ++i)
            sum += double(samples[i]) * samples[i];
        return qSqrt(sum / (to - from));
    }

    static Garden::Plant plant(int seed, const QString &outcome, double progress, int minutes)
    {
        Garden::Plant p;
        p.seed = seed;
        p.outcome = outcome;
        p.season = "autumn";
        p.weather = "fog";
        p.progress = progress;
        p.when = QDateTime(QDate(2026, 11, 6), QTime(9, 30));
        p.minutes = minutes;
        return p;
    }

    static void write(const QString &path, const QByteArray &contents)
    {
        QFile file(path);
        QVERIFY(file.open(QIODevice::WriteOnly));
        file.write(contents);
    }
};

QTEST_MAIN(GroveTests)
#include "grove_tests.moc"

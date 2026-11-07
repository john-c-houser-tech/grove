// The whole app, its real window drawn offscreen and driven by keys, as a user
// would: nothing appears on the desktop, and the same run is what the leak
// check watches. bin/test runs it on the offscreen platform with the GPU
// renderer, so particles and textures are drawn as they are on screen.

#include <QColor>
#include <QDir>
#include <QImage>
#include <QQuickItem>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QtTest>
#include <memory>

#include "app.h"

class SceneTests : public QObject {
    Q_OBJECT

private slots:
    // Each test starts from nothing: a garden, theme and language of its own.
    void init()
    {
        m_dir = std::make_unique<QTemporaryDir>();
        QVERIFY(m_dir->isValid());
    }
    void cleanup()
    {
        m_app.reset();
        m_dir.reset();
    }

    void spacePlantsASeed()
    {
        start();
        QCOMPARE(m_app->backend().state(), QString("idle"));
        press(Qt::Key_Space);
        QCOMPARE(m_app->backend().state(), QString("growing"));
    }

    void aSessionRunsToTheEndAndIsKept()
    {
        start(2);
        press(Qt::Key_Space);
        QTRY_COMPARE_WITH_TIMEOUT(m_app->backend().state(), QString("done"), 5000);
        QCOMPARE(m_app->garden().grown(), 1);
    }

    void givingUpAsksFirstAndWiltsThePlant()
    {
        start();
        press(Qt::Key_Space);
        press(Qt::Key_Space);
        QObject *confirm = find<QObject>("confirm");
        QTRY_VERIFY(confirm->property("opened").toBool());
        // Still growing until the answer is yes.
        QCOMPARE(m_app->backend().state(), QString("growing"));
        press(Qt::Key_Return);
        QCOMPARE(m_app->backend().state(), QString("wilted"));
        QCOMPARE(m_app->garden().wilted(), 1);
    }

    void theGardenOpensTakesTheKeysAndCloses()
    {
        start();
        for (int seed = 1; seed <= 3; ++seed)
            m_app->garden().record(plant(seed));
        auto *loader = find<QQuickItem>("gardenLoader");
        QVERIFY(!loader->property("item").value<QObject *>());

        press(Qt::Key_G);
        QTRY_VERIFY(loader->property("item").value<QObject *>());
        auto *grid = find<QQuickItem>("gardenGrid");
        QTRY_VERIFY(grid->hasActiveFocus());
        QCOMPARE(grid->property("currentIndex").toInt(), 0);
        press(Qt::Key_Right);
        QCOMPARE(grid->property("currentIndex").toInt(), 1);
        // The tile with the keys wears the accent.
        QTRY_COMPARE(currentTileBorder(grid), QColor(m_accent));

        press(Qt::Key_Escape);
        QTRY_VERIFY(!loader->property("item").value<QObject *>());
    }

    void theLandscapeIsDrawn()
    {
        start();
        // The layers draw on another thread; the field brings many tones to
        // the bottom of the window once it is there.
        QTRY_VERIFY_WITH_TIMEOUT(tonesIn(grab(), 0.8, 1.0) > 200, 5000);
        // And the hills to the middle.
        QVERIFY(tonesIn(grab(), 0.3, 0.6) > 50);
    }

    // Not a test: with GROVE_SHOTS set to a directory, saves the scene at
    // several hours, seasons and weathers there for a person to look at.
    void framesForReview()
    {
        const QString out = qEnvironmentVariable("GROVE_SHOTS");
        if (out.isEmpty())
            QSKIP("set GROVE_SHOTS to a directory to save frames");
        QDir().mkpath(out);
        struct Scene {
            const char *name, *date, *time, *weather;
        };
        const Scene scenes[] = {
            {"summer-noon", "2026-07-01", "12:00", "clear"}, {"summer-dusk", "2026-07-01", "20:40", "clear"},
            {"winter-noon", "2026-01-15", "12:00", "clear"}, {"night", "2026-07-01", "01:00", "clear"},
            {"rain", "2026-07-01", "12:00", "rain"},          {"snow", "2026-01-15", "12:00", "snow"},
            {"fog", "2026-10-15", "09:00", "fog"},
        };
        for (const Scene &scene : scenes) {
            App::Options options = baseOptions(25 * 60);
            options.date = QDate::fromString(scene.date, Qt::ISODate);
            options.time = QTime::fromString(scene.time, "HH:mm");
            options.weather = scene.weather;
            start(options);
            QTest::qWait(2500);
            QVERIFY(grab().save(out + "/" + scene.name + ".png"));
            m_app.reset();
        }
        start();
        for (int seed = 1; seed <= 6; ++seed)
            m_app->garden().record(plant(seed));
        press(Qt::Key_G);
        press(Qt::Key_Right);
        QTest::qWait(1000);
        QVERIFY(grab().save(out + "/garden.png"));
    }

private:
    App::Options baseOptions(int seconds) const
    {
        App::Options options;
        options.seconds = seconds;
        options.date = QDate(2026, 7, 1);
        options.time = QTime(12, 0);
        options.weather = "clear";
        options.gardenPath = m_dir->filePath("garden.json");
        // An empty theme directory gives the default accent, and no file
        // means English; nothing of the desktop's is read or written.
        options.themeDirectory = m_dir->filePath("theme");
        options.languageFile = m_dir->filePath("language");
        options.notificationService = "org.grove.tests.NoOneListening";
        options.rememberMute = false;
        return options;
    }

    void start(int seconds = 10) { start(baseOptions(seconds)); }
    void start(const App::Options &options)
    {
        m_app = std::make_unique<App>(options);
        m_app->sounds().setMuted(true);
        QVERIFY(m_app->load());
        QVERIFY(QTest::qWaitForWindowExposed(m_app->window()));
    }

    void press(Qt::Key key) { QTest::keyClick(m_app->window(), key); }

    template <typename T> T *find(const char *name) const
    {
        T *found = m_app->window()->findChild<T *>(name);
        if (!found)
            qFatal("no item named %s", name);
        return found;
    }

    QImage grab() const { return m_app->window()->grabWindow(); }

    // How many different colours there are in a band of the window, given as
    // fractions of its height.
    static int tonesIn(const QImage &image, double from, double to)
    {
        QSet<QRgb> tones;
        for (int y = int(image.height() * from); y < int(image.height() * to); y += 2)
            for (int x = 0; x < image.width(); x += 2)
                tones.insert(image.pixel(x, y));
        return tones.size();
    }

    // The border colour of the tile that has the keys.
    static QColor currentTileBorder(QQuickItem *grid)
    {
        const auto *tile = grid->property("currentItem").value<QQuickItem *>();
        if (!tile || tile->childItems().isEmpty())
            return {};
        auto *border = tile->childItems().constFirst()->property("border").value<QObject *>();
        return border ? border->property("color").value<QColor>() : QColor();
    }

    static Garden::Plant plant(int seed)
    {
        Garden::Plant p;
        p.seed = seed;
        p.outcome = seed % 2 ? "done" : "wilted";
        p.season = "summer";
        p.weather = "clear";
        p.progress = seed % 2 ? 1.0 : 0.4;
        p.when = QDateTime(QDate(2026, 7, seed), QTime(9, 0));
        p.minutes = seed % 2 ? 25 : 0;
        return p;
    }

    std::unique_ptr<QTemporaryDir> m_dir;
    std::unique_ptr<App> m_app;
    // Theme's fallback accent, with no Omarchy theme to read.
    const char *m_accent = "#FFD60A";
};

QTEST_MAIN(SceneTests)
#include "scene_tests.moc"

// grove — a focus timer that grows a plant while you work.

#include <QGuiApplication>
#include <QIcon>
#ifdef __GLIBC__
#include <malloc.h>
#endif

#include "app.h"

int main(int argc, char *argv[]) {
#ifdef __GLIBC__
    // Pictures of the landscape are drawn on worker threads into buffers of a
    // few megabytes. glibc raises its threshold for giving such buffers their
    // own mapping after the first is freed, so later ones come from the heap
    // and, once freed, stay with the process. Fixing the threshold returns
    // each one to the system when it goes.
    mallopt(M_MMAP_THRESHOLD, 128 * 1024);
#endif
    const QGuiApplication application(argc, argv);
    QGuiApplication::setApplicationName("grove");
    QGuiApplication::setApplicationVersion("0.9.0");

    // Matches the window to grove.desktop, so the compositor (Wayland app_id)
    // and the launcher pick up the installed icon.
    QGuiApplication::setDesktopFileName("grove");
    QGuiApplication::setWindowIcon(QIcon::fromTheme("grove"));

    App app(App::fromEnvironment());
    if (!app.load())
        return 1;
    return QGuiApplication::exec();
}

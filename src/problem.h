#pragma once

#include <QString>
#include <functional>
#include <utility>

// Something the window should say is wrong, kept as the words to say it rather
// than as text, so it is translated each time it is shown and follows a change
// of language as the QML text does. The words are usually a lambda returning
// tr("..."), which keeps the string where bin/check-i18n finds it.
class Problem {
public:
    using Words = std::function<QString()>;

    QString text() const { return m_words ? m_words() : QString(); }

    // Returns whether what the window shows has changed.
    bool set(Words words) {
        const QString before = text();
        m_words = std::move(words);
        return text() != before;
    }

private:
    Words m_words;
};

# Languages

<!-- Reading, typing and checking other languages. Checked 2026-11-21
     against ewe 0.45.0-beta (adobe-source-han-sans-otc-fonts since 0.42;
     ewe Settings 0.25's Time and place has Display language) and the
     merged ewe 0.46 branches: ewe .wt/l-ewe 415555e (bin/ewe-ime: status,
     catalog, current, add, remove, enable, disable, apply, toggle,
     configure, sizes; [desktop.ime] {enabled, engines}; Ctrl+Space; the
     bar's layout indicator shows the engine's glyph (ImeState.qml);
     hunspell + hunspell-en_us; [desktop.language] {spellcheck,
     check_spelling}; the clock and calendar follow LC_TIME), ewe Settings
     .wt/l-settings 6f0fb9e (pages.js: input holds "Typing in other
     languages" — Add a language…, Type in these languages, Language
     options, Not installed on this computer → Install; the new page
     "language": Display language → Change…, Also for your apps (Install,
     Install all), Spell checking (Check spelling as you type, Add a
     language…, Download / Open Komble, "Not in ewe Browser yet");
     time: Formats → Region → Change…, Example, Calendar, "Same as the
     display language"), ewe Browser .wt/l-browser (staged: Browsing →
     Spelling; Chromium has no Georgian dictionary) and ewe-apps
     .wt/a-player 3011bce (names read in their own direction).
     Comments never reach a model. -->

## Typing in other languages
keys: typing in other languages, type in other languages, type in chinese, type in japanese, type in korean, type in vietnamese, type in hindi, input method, ime, pinyin, mozc, hangul, unikey, fcitx, fcitx5, ewe-ime
open: settings input
From ewe 0.46: ewe Settings → Keyboard and mouse → Typing in other languages → Add a language…: Chinese (Pinyin), Japanese (Mozc), Korean (Hangul), Vietnamese (Unikey), or Hindi, Bengali, Tamil, Thai and other Indian and Asian languages; ewe installs what it needs the first time, and asks for your password once. Type in these languages turns them all on or off. Then Ctrl+Space switches to the language and back (hold Ctrl and tap Space again to step through several), and the bar's keyboard indicator shows 拼, あ or 한 while it is on. Super+Space still switches keyboard layouts. Language options opens the input method's own settings.

Nothing for this is installed until you add a language, and your list syncs with your settings (on a computer that lacks the pieces, ewe Settings says Not installed on this computer, with Install). From a terminal: `ewe-ime status`, `ewe-ime catalog` (every language offered), `ewe-ime add mozc`, `ewe-ime remove mozc`, `ewe-ime toggle` (what Ctrl+Space does) and `ewe-ime configure` (the input method's own settings). Up to ewe 0.45 ewe has no way to type these languages.

## Spell checking
keys: spell check, spellcheck, spelling, spelling language, dictionary, add to dictionary, hunspell, typos, red underline
open: settings language
From ewe 0.46: ewe Settings → Language → Spell checking. Check spelling as you type turns it on or off, and the list holds the languages words are checked in: at first your display language's and US English, which every computer has. Add a language… adds one; its dictionary comes with Download (or Open Komble, when it is in the AUR), and × removes it. ewe Browser underlines misspelt words and its right-click offers suggestions, Add to dictionary and Ignore (its own switch: ewe Browser → Settings → Browsing → Spelling); LibreOffice and other apps that use the system's dictionaries check with them too.

A language marked Not in ewe Browser yet is checked by the other apps only: ewe Browser has no dictionary for Georgian and a few others. Up to ewe 0.45 no dictionary was installed, so nothing checked spelling.

## The display language
keys: display language, language, system language, change the language, menus in my language, locale, translation
open: settings language
ewe Settings → Language → Display language → Change… (from ewe 0.46; up to ewe 0.45 the same row is in ewe Settings → Time and place): pick a language. One that is not set up yet is prepared first, with one password prompt. Apps that are already open switch after you sign out and back in.

ewe's own bar, panels and apps are in English for now; the other apps (Firefox, LibreOffice, GTK and Qt apps) follow the display language, and from ewe 0.46 the launcher finds an app by its translated name and by its English one. How dates, times and numbers are written is the region's (Date, time and number formats), and other apps' language packs are offered next to it (Language packs for your apps).

## Date, time and number formats
keys: regional formats, region formats, date format, time format, 24-hour clock, 12-hour clock, am pm, first day of the week, week starts on monday, number format, decimal comma, metric units
open: settings time
From ewe 0.46: ewe Settings → Time and place → Formats → Region → Change…: pick a region, and dates, times, numbers, money, measurements and paper sizes are written its way (Example shows how, and Calendar says which day weeks start on). Same as the display language is the default; a change applies after you sign out. The bar's clock, the lock screen and Quick settings' calendar follow it: a 24-hour clock where the region uses one, and weeks that start on Monday or Sunday.

For a 24-hour clock with English menus, keep English as the display language and pick a region that writes 24 hours, like English (United Kingdom) or German (Germany). Up to ewe 0.45 the bar's clock always shows 12 hours with AM and PM.

## Language packs for your apps
keys: language pack, firefox in my language, libreoffice in my language, thunderbird in my language, firefox language, libreoffice language
open: settings language
From ewe 0.46, ewe Settings → Language → Also for your apps offers the language packs that make Firefox, Thunderbird and LibreOffice speak your display language: Install for one, Install all for every one. It lists only the apps you have installed, and the pack follows the display language you pick.

## Chinese, Japanese and Korean text, and right-to-left names
keys: chinese characters, japanese characters, korean characters, boxes instead of letters, squares instead of text, cjk, arabic text, hebrew text, right to left
Chinese, Japanese and Korean text shows since ewe 0.42 (the Source Han Sans font comes with ewe), so web pages, file names and messages no longer show boxes, and the Japanese, Korean or Chinese shapes are used when the text says its language. To type them, see Typing in other languages; for other alphabets (Russian, Georgian, Greek …), a keyboard layout is enough.

From ewe 0.46, Arabic and Hebrew file names, song titles and document names read right to left in ewe Files, ewe Player and ewe Viewer, with the ellipsis at their own end; ewe's own menus stay left to right.
<!-- right to left: ewe-apps a-player 3011bce (merged feat/text-images). -->

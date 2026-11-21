# Keyboard shortcuts

<!-- The defaults from ewe/dotfiles/hypr/hyprland.lua, checked against
     0.39.0-beta (origin/main db58e95; Super+Z minimizes since 0.34, which
     retired the Pen and Super+S) and the plugins' manifests. The
     four-finger swipe is wave 2 (ewe 0.30.0-beta). Mod is Super. The
     companion adds the user's own binds and the enabled plugins' binds at
     question time, so only the defaults live here.
     Refreshed 2026-11-21 against ewe 0.45.0-beta (e19bfbb hyprland.lua:
     fileManager = ewe-files-app since 0.44; Alt+F4, Ctrl+Alt+T and
     Ctrl+Alt+Delete since 0.43; a minimized app comes back when you open
     it, 0.43) and plugins/ewe.switcher 1.0.1 (Alt+Tab). Ctrl+Space for
     typing in other languages is ewe 0.46, from the 2026-11-21 wave plan
     (bin/ewe-ime on feat/media-languages). -->

## Shortcuts for opening things
keys: shortcut, keybind, hotkey, key, keyboard shortcuts, open launcher, open terminal, open settings, open files
Super is the Windows key. Tap Super alone for the Overview. Super+D opens the launcher (press it again to close), Super+Return a terminal (kitty; Ctrl+Alt+T too), Super+E the file manager (ewe Files), Super+B the browser (or brings the open one forward), Super+, ewe Settings, Super+N Quick settings, Super+C the calendar, and Super+A Shaun (the companion).

Super+Shift+B shows or hides the top bar, Super+Space switches the keyboard layout (from ewe 0.46, Ctrl+Space turns typing in other languages on and off), Super+Alt+L locks the screen, Super+Ctrl+R reloads the desktop's config, Super+Shift+R turns every screen back on (after a monitor was plugged in and the screen stayed black), and Super+Ctrl+Q or Ctrl+Alt+Delete signs out (it asks first). Super+Shift+W arranges desktop widgets (Esc ends it). ewe Settings → Keyboard shortcuts lists them all.

## Shortcuts for windows
keys: window, close window, fullscreen, maximise, maximize, float, tile, move window, resize window, focus, group, tabs, split
Super+Q (or Alt+F4) closes the window, Super+F makes it fullscreen, Super+Shift+F maximises it (the bar and gaps stay), Super+V makes it float or tile again, Super+T turns the split the other way and Super+Shift+V pseudo-tiles.

Move focus with Super+H/J/K/L or Super+arrows; add Shift to move the window, Ctrl to resize it. Super+G groups windows into tabs (Super+Shift+G takes one out, Super+[ and Super+] switch tabs, Super+Alt+1…9 jumps to a tab). Super+Tab and Super+Shift+Tab go to the next and previous window. Super with the left mouse button drags a window; with the right one it resizes.

Or ask me: "make this window fullscreen", "exit fullscreen", "maximise this window", "float this window", "tile this window", "minimise this window", "bring back the minimized window". "Close this window" or "close firefox" shows a Close button first; the app may still ask to save. "This window" is the one in front.

## Shortcuts for the scrolling layout
keys: scrolling layout, scrolling, columns, column width, strip
These work only when ewe Settings → Layout → Tiling style is Scrolling. Super+Alt+[ and Super+Alt+] move the strip one column, Super+Alt+, and Super+Alt+. swap with the column on the left or right, Super+Alt+R cycles column widths, Super+Alt+F fits the visible columns on screen, Super+Alt+P gives the window its own column and Super+Alt+C merges it into the next column or splits it out.

## Workspaces and minimizing windows
keys: workspace, desktop, virtual desktop, switch workspace, move to workspace, minimize, minimise, minimize a window, minimized, hide window, pen, stash
Super+1…8 switches to workspace 1 to 8, and Super+Shift+1…8 moves the window there and follows it. Super with the mouse wheel cycles workspaces (with the Scrolling layout it scrolls the strip instead).

Super+Z minimizes the window in front, and Super+Shift+Z brings back the one you minimized last, onto the workspace you are on. With the Dock plugin, each minimized window is a picture in the dock: click it to bring it back. Opening a minimized app again (the launcher, its dock icon) brings it back too. Minimizing replaced the Pen in ewe 0.34: there is no Pen to stash windows in any more, Super+S does nothing, and windows that were still in the Pen count as minimized.

Or ask me: "switch to workspace 3", "go to the second workspace", "next workspace", "move this window to workspace 2", "move firefox to workspace 4". Undo goes back. With several displays, the numbers are the ones of the display you are on, unless ewe Settings → Displays → Workspaces is set to Shared.

## Touchpad gestures
keys: touchpad, gesture, swipe, three fingers, four fingers, trackpad
Swipe left or right with three fingers to switch workspaces, up for the Overview, and down to go to a new empty desktop (ewe never piles up empty ones). From ewe 0.30, four fingers left or right switch ewe Browser's own workspaces while the browser is in front, and do nothing elsewhere. In ewe Browser, two fingers to the right go back and to the left go forward (with natural scrolling, ewe's default; the other way round without it). Touchpad settings (tap to click, natural scrolling, speed) are in ewe Settings → Keyboard and mouse.

## Media and hardware keys
keys: volume key, brightness key, media keys, play pause key, power button, lid, keyboard backlight, function keys
The volume keys move by 5 % (they work on the lock screen too), mute and mic-mute toggle, and play, next and previous control what plays. The brightness keys move by 5 % with the on-screen indicator, and the keyboard-light keys step the keyboard backlight.

A short press of the power button opens Quick settings asking "Shut down this computer?"; holding it still powers off, and it does nothing while the screen is locked.

## Plugin shortcuts
keys: plugin shortcut, print screen, print key, screenshot key, password shortcut, cast shortcut, super p, super a, super shift p
These exist only while the plugin is installed and on. Screenshot: Print takes the screen, Shift+Print a region, Super+Print the window. Passwords: Super+P fills a login. ewe Safe: Super+Shift+P opens Quick Access. Cast to TV: Super+Shift+C opens the TV list or hangs up. Window switcher: Alt+Tab and Alt+Shift+Tab. Shaun (the companion): Super+A. Clipboard history has no key: it is the scissors in the bar.

## Make your own shortcut
keys: own shortcut, custom shortcut, add shortcut, new shortcut, change shortcut, rebind, bind a key, custom keybind
Your own launch shortcuts are recorded in ewe.conf as `desktop.keybinds`, so they sync with it. Today you add them from a terminal, for example `ewe-conf set desktop.keybinds '[{"combo": "SUPER + U", "exec": "kitty"}]'` (the list replaces the old one). ewe Settings → Keyboard shortcuts shows shortcuts but cannot change them yet. Pick a combo ewe does not use (ewe Settings → Keyboard shortcuts lists them): nothing stops a clash, and both would run.

Apps that register global shortcuts (1Password's Quick Access, Discord push-to-talk) get them bound automatically when the combo is free; `ewe-globalshortcuts list` shows what was bound and what was skipped.

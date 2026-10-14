# Keyboard shortcuts

<!-- The defaults from ewe/dotfiles/hypr/hyprland.lua (0.25.2), checked
     against a live `hyprctl binds -j`. Mod is Super. The companion adds
     the user's own binds and the enabled plugins' binds at question time,
     so only the defaults live here. -->

## Shortcuts for opening things
keys: shortcut, shortcuts, keybind, keybinds, hotkey, key, keys, keyboard shortcut, open launcher, open terminal, open settings, open files
Super is the Windows key. Tap Super alone for the Overview. Super+D opens the launcher (press it again to close), Super+Return a terminal (kitty), Super+E the file manager (Nemo), Super+B the browser, Super+, Settings, Super+N Quick settings, Super+C the calendar, and Super+A the companion.

Super+Shift+B shows or hides the top bar, Super+Space switches the keyboard layout, Super+Alt+L locks the screen, Super+Ctrl+R reloads the desktop's config and Super+Ctrl+Q logs out. Super+Shift+W arranges desktop widgets (Esc ends it). Settings → Shortcuts lists them all.

## Shortcuts for windows
keys: window, windows, close window, fullscreen, maximise, maximize, float, floating, tile, tiling, move window, resize window, focus, group, tabs, split
Super+Q closes the window, Super+F makes it fullscreen, Super+Shift+F maximises it (the bar and gaps stay), Super+V makes it float or tile again, Super+T turns the split the other way and Super+Shift+V pseudo-tiles.

Move focus with Super+H/J/K/L or Super+arrows; add Shift to move the window, Ctrl to resize it. Super+G groups windows into tabs (Super+Shift+G takes one out, Super+[ and Super+] switch tabs, Super+Alt+1…9 jumps to a tab). Super+Tab and Super+Shift+Tab go to the next and previous window. Super with the left mouse button drags a window; with the right one it resizes.

## Shortcuts for the scrolling layout
keys: scrolling layout, scrolling, columns, column width, strip
These work only when Settings → Layout → Tiling style is Scrolling. Super+Alt+[ and Super+Alt+] move the strip one column, Super+Alt+, and Super+Alt+. swap with the column on the left or right, Super+Alt+R cycles column widths, Super+Alt+F fits the visible columns on screen, Super+Alt+P gives the window its own column and Super+Alt+C merges it into the next column or splits it out.

## Workspaces and the Pen
keys: workspace, workspaces, desktop, desktops, virtual desktop, switch workspace, move to workspace, pen, stash, hide window
Super+1…8 switches to workspace 1 to 8, and Super+Shift+1…8 moves the window there and follows it. Super with the mouse wheel cycles workspaces.

The Pen is a hidden workspace for windows you want out of the way: Super+S (or Super+Z, or Super+`) goes in and out of it, Super+Shift+S stashes the window into the Pen and Super+Shift+Z sends it back.

## Touchpad gestures
keys: touchpad, gesture, gestures, swipe, three fingers, trackpad
Swipe left or right with three fingers to switch workspaces, up for the Overview, and down to go to a new empty desktop (ewe never piles up empty ones). Touchpad settings (tap to click, natural scrolling, speed) are in Settings → Keyboard and mouse.

## Media and hardware keys
keys: volume key, brightness key, media keys, play pause key, power button, lid, keyboard backlight, function keys
The volume keys move by 5 % (they work on the lock screen too), mute and mic-mute toggle, and play, next and previous control what plays. The brightness keys move by 5 % with the on-screen indicator, and the keyboard-light keys step the keyboard backlight.

A short press of the power button opens Quick settings asking "Shut down this computer?"; holding it still powers off, and it does nothing while the screen is locked.

## Plugin shortcuts
keys: plugin shortcut, print screen, print key, screenshot key, password shortcut, cast shortcut, super p, super a
These exist only while the plugin is installed and on. Screenshot: Print takes the screen, Shift+Print a region, Super+Print the window. Passwords: Super+P fills a login. Cast to TV: Super+Shift+C opens the TV list or hangs up. Companion: Super+A. Clipboard history has no key: it is the scissors in the bar.

## Make your own shortcut
keys: own shortcut, custom shortcut, add shortcut, new shortcut, change shortcut, rebind, bind a key, custom keybind
Your own launch shortcuts are recorded in ewe.conf as `desktop.keybinds`, so they sync with it. Today you add them from a terminal, for example `ewe-conf set desktop.keybinds '[{"combo": "SUPER + T", "exec": "kitty"}]'` (the list replaces the old one). Settings → Shortcuts shows shortcuts but cannot change them yet. Combos already taken by ewe stay ewe's.

Apps that register global shortcuts (1Password's Quick Access, Discord push-to-talk) get them bound automatically when the combo is free; `ewe-globalshortcuts list` shows what was bound and what was skipped.

# ewe browser

<!-- Topics about ewe browser beyond the app itself (apps.md has it).
     Checked 2026-10-18 against ewe browser 0.3.0-beta (origin/main
     a18366c): docs/CLI.md, packages/shared/src/shortcuts.ts (the default
     keys), BottomBar.tsx, pages/settings/main.tsx (the section and row
     labels), pages/history/main.tsx, and Safe 0.2.0-beta's README.
     Marked "(0.4)": wave 2 (ewe browser 0.4.0-beta, ewe 0.30.0-beta) as the
     wave plan's contracts define it, written before it landed: re-check the
     labels and keys against the browser's SHORTCUT_IDS and settings page
     when it does. Comments never reach a model. -->

## The bottom bar and workspaces in ewe browser
keys: browser workspace, browser workspaces, workspaces in the browser, bottom bar, workspace chips, new workspace, rename workspace, move tab to workspace, switch workspace in the browser
ewe browser keeps tabs in workspaces, shown as chips at the left of the bar along its bottom: click one to switch, double-click to rename, right-click for Rename, a colour, Move tabs here and Remove (which closes its tabs). "+" makes "Workspace N" and opens it on Home. Drag a tab onto a chip to move it, or right-click the tab → Move to workspace. The right of the bar has Bookmarks, Downloads, History, Music, Files and Settings.

Keys: Alt+Shift+1…9 jumps to a workspace (9 is the last), Ctrl+Alt+Right and Ctrl+Alt+Left (or Ctrl+Shift+Page Down and Page Up) go to the next and previous one. (0.4) Alt+Shift+Right and Alt+Shift+Left do too, Alt+Shift+N makes a new one, Alt+Shift+R renames it, and a four-finger swipe on the touchpad switches them while ewe browser is in front.

Ask me: "switch to my work workspace in the browser", "next browser workspace", "new workspace called travel", "show my browser workspaces", "open github.com in my work workspace".

## Search a site from ewe browser
keys: search a site, search on a site, site search, search zara, search amazon, shop on, search inside a site, search youtube for
(0.4) Say "search men's shoes on zara", "find a kettle on amazon" or "search youtube for sheepdogs": I ask ewe browser, which opens that site's own search results. In the browser's palette (Ctrl+K), type `@zara men's shoes`. It finds a site's search page from your own history first (so it keeps your country's version of the shop), then from a search box the site announces, then from a built-in list of about fifty sites; for a site it knows only by name, it searches the web with `site:`. When it does not know the site at all, I search the web for the whole phrase and say so.

## History in ewe browser
keys: browser history, browsing history, my history, visited sites, site i visited, page i visited, clear history, delete history, clear browsing data
Ctrl+H opens the History sidebar; Ctrl+Shift+H opens the full History page, with search (`site:example.com` works), "Delete everything from this site…" and "Keep history for". Settings → Browsing → Clear data… clears it all. Deleting syncs to your other computers. Ask me: "find tax in my history" or "that site about sheep", and a click opens the page.

## Tabs in ewe browser
keys: browser tabs, my tabs, open tabs, switch tab, go to tab, youtube tab, which tabs, mute tab, pin tab, close tab, reopen closed tab
Ctrl+T opens a tab, Ctrl+W closes it, Ctrl+Shift+T reopens the last one closed, Ctrl+Tab and Ctrl+Shift+Tab move between them and Ctrl+1…9 (or Alt+1…9) jump to one. The palette (Ctrl+K) finds tabs with `t:`, your other computers' tabs too. Right-click a tab for Reload, Duplicate, Pin, Mute, Copy url, Move to workspace and Close.

Ask me: "show my tabs", "go to the YouTube tab", "mute this tab", "pin the github tab", "reopen the closed tab", "go back a page", "reload the page". "Close this tab" I only offer, with Run: ewe browser asks first for that one.

## Bookmarks and downloads in ewe browser
keys: browser bookmarks, my bookmarks, bookmark this page, add bookmark, browser downloads, my downloads in the browser, what did i download, download folder browser
Ctrl+D bookmarks the page, Ctrl+Shift+B opens the Bookmarks sidebar and Ctrl+J the Downloads one; the palette finds bookmarks with `b:`. Ask me: "bookmark this page" (with Undo), "find recipes in my bookmarks", "what did I download" (each with Open and Show in folder), "open the browser's settings" (or its history, music, files or home).

## Shortcuts in ewe browser
keys: browser shortcuts, browser shortcut, browser keys, ewe browser shortcuts, browser keyboard shortcuts, shortcuts in the browser
Settings → Keyboard shortcuts changes any of them. Address bar Ctrl+L, palette Ctrl+K (`t:` tabs, `b:` bookmarks, `w:` the web), Home Alt+Home, back Alt+Left, forward Alt+Right, reload Ctrl+R or F5 (Ctrl+Shift+R skips the cache), bookmark Ctrl+D, Files Ctrl+Shift+E, fill a password Ctrl+Shift+L, Bookmarks, Downloads and History sidebars Ctrl+Shift+B, Ctrl+J and Ctrl+H, the History page Ctrl+Shift+H, hide the tab rail Ctrl+Shift+S, new window Ctrl+N, close it Ctrl+Shift+W, zoom Ctrl+= Ctrl+- Ctrl+0, full screen F11, developer tools F12, quit Ctrl+Shift+Q.

(0.4) Find in page Ctrl+F, Settings Ctrl+,, print Ctrl+P, page source Ctrl+U, mute the tab Ctrl+M, pin it Alt+Shift+P, duplicate it Alt+Shift+D, copy its address Alt+Shift+C, search your tabs Ctrl+Shift+A, Music Alt+Shift+M, Home also Alt+Shift+H.

## Find on a page in ewe browser
keys: find in page, find on page, search this page, ctrl+f in the browser, find text on a page
(0.4) Ctrl+F opens the find bar at the top right of the page, filled with what you selected, and counts the matches. Enter, F3 or Ctrl+G goes to the next one, Shift+Enter, Shift+F3 or Ctrl+Shift+G to the one before, and Esc closes it.

## Gestures and mouse buttons in ewe browser
keys: browser gestures, swipe back, two finger swipe, swipe to go back, mouse side buttons, back button on mouse, four finger swipe browser
(0.4) Swipe two fingers to the right on the touchpad to go back and to the left to go forward (with natural scrolling, ewe's default; the other way round without it): an arrow at the page's edge fills as you swipe, and letting go before it is full does nothing (Settings turns it off). The side buttons of a mouse go back and forward too. Four fingers left or right switch ewe browser's workspaces while the browser is in front; three fingers stay the desktop's.

## Right-click menus in ewe browser
keys: right click in the browser, right-click menu, context menu, open link in new tab, save image, copy image, inspect element, open link in new workspace
(0.4) Right-click on a page: a link opens in a new tab, a new workspace or a new window, or is copied or saved; an image opens, saves or copies; a video or song plays, pauses, mutes, loops or saves; selected text copies or is searched on the web; in a text field you get spelling suggestions, undo, cut, copy, paste and Paste as plain text, and in a login field Fill from Safe (Use a strong password in a new-password field). Anywhere: back, forward, reload, bookmark, Save page as, print, view source and Inspect.

## Privacy and security in ewe browser
keys: secure dns, dns over https, https only, https-only, site permissions, camera permission, microphone permission, notification permission, third party cookies, global privacy control, gpc, privacy settings
(0.4) Settings → Privacy and security. Secure DNS: Off (the system's, the default), Automatic, or a provider (Cloudflare, Quad9, Mullvad, AdGuard, NextDNS, or your own address). HTTPS-Only turns http addresses into https and, when a site has no https, asks before going on to the http one. Third-party cookies are blocked and Global Privacy Control is sent, both on by default.

A site that wants the camera, the microphone, your location or to send notifications asks with a card at the top: Allow or Block, remembered for that site. The lock in the address bar shows whether the connection is secure, the site's permissions with switches, and Clear this site's data; Settings → Privacy and security lists every site's permissions to change or forget.

## Sync in ewe browser
keys: browser sync, sync browser, sync bookmarks, sync history, tabs from other computers, other computer tabs, browser on another computer
Settings → Sync → Sync with Nextcloud keeps bookmarks, history (deletions too) and settings in step through your ewe account's Nextcloud, and the palette's `t:` lists your other computers' open tabs. ewe-sync, which owns all syncing, does not show the browser's sync state yet.

## Safe in ewe browser
keys: safe in browser, safe in the browser, browser passwords, fill in browser, autofill browser, browser login, password in browser, password in the browser, login in the browser, login suggestions, password manager extension, 1password extension, bitwarden extension
Safe is built in: click a login, email or password field and a list drops under it with your passkeys and this site's logins, then Open Safe, Use a strong password or Unlock Safe; type to filter, Enter fills, Esc closes. Nothing fills on its own when a page loads. After you sign in, ewe browser asks "Save to Safe?" or offers to update the password. Settings → ewe holds Fill from Safe, Show suggestions when a field gets focus, Passkeys and Offer to save logins. Or ask me "log me in to GitHub": I open the site and you fill it there.

(0.4) Only Safe fills logins in ewe browser: Ctrl+Shift+L and right-click → Fill from Safe open Safe's list for the field. A password-manager extension (1Password, Bitwarden, LastPass and the like) is refused when you add it, and one already there is turned off with a note: import your logins into Safe instead (Safe → Import, or `ewe-safe import 1pux`).

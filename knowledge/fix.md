# When something is wrong

<!-- What a user can do themselves, from project-map/11-How-We-Work/
     Troubleshooting Knowledge.md, ewe/docs/TROUBLESHOOTING.md and the
     website's troubleshooting page. Developer-only fixes stay out.
     Re-checked 2026-11-01 against ewe 0.39.0-beta, ewe Settings 0.22 and
     ewe Sync 0.16 (no "Push anyway": the settings decision is "Use
     <name>'s settings…" or "Keep this computer's settings"). -->

## Something from a plugin is missing
keys: missing dock, no dock, missing clipboard, no screenshot, missing tile, plugin missing, plugin not working, shortcut not working, icon missing
open: komble plugins
A fresh ewe has no plugins, so the dock, clipboard, screenshots and Cast tile are not there until you add them: Komble → Plugins → From ewe. If one you had stopped working, `ewe-plugin list` shows whether it is installed and on, and Show in the bar in its sheet (click it in Komble → Plugins) decides whether it shows in the bar. The System monitor left Quick settings in ewe 0.39: its CPU, memory and speed readout is in the top bar, and a click on it opens the card.

## Plugins disabled for this session
keys: plugins disabled, safe mode, shell crashed, shell keeps crashing
The shell crashed three times in a minute, so it started without plugins. Turn off the plugin it names (`ewe-plugin disable <id>`); a start that stays up for a minute clears safe mode.

## The desktop looks old after an update
keys: stale desktop, old version after update, update did not apply, looks broken after update, after update
Log out and back in first. If it still looks old, run `ewe-setup` and log in again. For the login screen or boot splash, run `bash /usr/share/ewe/install.sh --no-packages` as yourself, never with sudo.

## A setting does not stick
keys: setting not saved, setting resets, setting does not stick, settings lost, config reset, change reverted
Change it in ewe Settings (or with `ewe-conf set …`), never in ~/.config/hypr/generated/, which is rebuilt from ewe.conf. `ewe-conf get <key>` shows what is recorded and `ewe-conf apply` applies it again.

## ewe Settings says the shell is not running
keys: shell not running, shell isn't running, restart shell, bar gone, bar disappeared, desktop frozen
ewe Settings → About this computer → Restart the shell → Restart…, then Restart; your windows stay open. If the bar is just hidden, Super+Shift+B brings it back.

## Sync will not upload or says another computer is newer
keys: sync conflict, sync not working, push anyway, remote newer, another machine, another computer, sync refuses, sync error, two computers changed, needs you, needs a decision
open: sync
Open ewe Sync: its Overview shows the problem under "Needs you". When two computers changed your settings, choose in Settings & apps: "Use <name>'s settings…" takes the other computer's, "Keep this computer's settings" keeps yours and puts them in your account (that is what Push anyway used to do; the button is gone). A file changed in two places waits under its folder in Folders: "Keep this computer's" or "Keep the account's". Give each computer its own name (ewe Settings → Network → Name on the network). `ewe-conf sync-status` says why: remote-newer means the account's copy is newer, remote-exists means the account already has settings.

## The keyring keeps asking for a password
keys: keyring, keyring password, choose password for new keyring, keyring prompt, keyring timeout, secret service
When an app asks, it wants your login password. If a "Choose password for new keyring" window keeps coming, run `ewe-auth keyring-reset`, log out and back in, then sign in again from ewe Sync.

## The sign-in page did not open
keys: sign in browser did not open, login page not opening, cannot sign in, sign-in link
Use "Open the sign-in page" or "Copy the link" in Welcome or ewe Sync, and finish in any browser.

## A Bluetooth device will not pair
keys: bluetooth not pairing, bluetooth drops, cannot pair, will not pair, won't pair, disconnect, disconnects, keeps disconnecting, keeps dropping, bluetooth problem
Forget the device on both sides and pair again, holding its pairing button until it blinks. `systemctl restart bluetooth` brings back ewe's pairing helper.

## A black screen after plugging a monitor
keys: black screen, screen is black, screen went black, black after, after plugging, no display, monitor not working, screen off, external monitor black, display problem
Press Super+Shift+R: it turns every screen back on and reapplies your saved arrangement. ewe Settings → Displays → Reset displays does the same.

## Apps are too big or too small on a second screen
keys: steam too big, blurry app, app scale wrong, java app scale, xwayland scale, tiny app
Old X11 apps (Steam, Java) get one scale, chosen when you log in. Log out and back in after docking.

## Cast to TV fails
keys: cast not working, tv not found, cast frozen, cast no sound, casting problem
Run `sh ~/.config/ewe/plugins/ewe.cast/cast-check.sh`. Open Screen Mirroring on the TV before choosing it, keep the laptop near the TV and prefer 5 GHz Wi-Fi.

## The VPN will not start
keys: vpn failed, vpn service failed, vpn not connecting, l2tp failed, internet only with vpn
For "The VPN service failed to start" (L2TP), run `bash /usr/share/ewe/install.sh --no-packages` as yourself. If the internet works only with the VPN on, `/etc/resolv.conf` must point at systemd-resolved; updating ewe fixes it.

## Mail shows nothing
keys: mail empty, mail not working, mail auth failed, no mail badge
Install the Mail plugin and check `ewe-mail status`. "auth-failed" usually means the provider wants an app-specific password.

## Screen sharing or file pickers do nothing
keys: screen share not working, screen sharing, file picker not opening, portal, share screen
The session's portal service is missing; run `bash /usr/share/ewe/install.sh --no-packages` as yourself, then log in again.

## The laptop freezes
keys: freeze, freezes, hard freeze, hang, lunar lake, crash, frozen laptop, laptop is frozen, computer is frozen, everything is frozen, whole screen is frozen
Update everything (`sudo pacman -Syu`) for a newer kernel and firmware. On Intel Lunar Lake laptops the kernel parameter `xe.enable_psr=0` helps. Keep the output of `journalctl -b -1 -p err` for a report.

## An app is frozen
keys: app is frozen, an app freezes, frozen app, not responding, force quit, kill an app, kill a program, app won't close
Super+Q asks the window to close. When an app does not answer at all, ask me "firefox is frozen" or "force quit firefox": I show `pkill -KILL -x firefox` with Run, and it ends at once, losing what it did not save. "Kill firefox" shows `pkill -x firefox`, which asks it to end. Nothing runs until you press Run.

## The disk is full
keys: disk full, storage full, no space, low disk space, free space, clean up, cleanup, disk space
run: sudo paccache -rk1
Ask me "what's eating my disk" to see what takes space. Clear pacman's package cache with `sudo paccache -rk1` (keeps one old version), empty the trash (in Nemo, or ask me "empty the trash"), and look for big folders with `du -sh ~/* | sort -h`. I show a disk past 85 % in my panel.

## What to send with a bug report
keys: report bug, bug report, logs, diagnostics, status commands, help me debug
These print what ewe knows, as JSON: `ewe-cloud status`, `ewe-conf sync-status`, `ewe-plugin list`, `ewe-mail status` and `ewe-auth status`. Add `journalctl -b -1 -p err` for a crash.

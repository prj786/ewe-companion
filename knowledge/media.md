# Video, music and pictures

<!-- Playing things: links, codecs, Stremio, phone photos, subtitles, DVDs.
     Checked 2026-11-21 against ewe 0.45.0-beta (packages/common.list: the
     GStreamer sets, gst-libav, gst-plugin-va, ffmpeg since 0.44), ewe apps
     0.1.1 (AGENTS.md "Codecs": the 26-row matrix; the Player's Open URL…
     on Ctrl+L, Load subtitles…, sidecar subtitles; the Viewer's plain
     "cannot show HEIC"), and the 2026-11-21 wave plan (Phase A) with the
     work in progress for ewe 0.46: .wt/a-player (feat/player-links,
     uncommitted: player.ts decodeFailureMessage's titles "This is a web page,
     not a video file", "This video is locked"; links.rs yt-dlp routes;
     languages.ts native names and the preferred-language rule), .wt/a-text
     (feat/text-images, committed: "Subtitle encoding", the Viewer's encoding
     menu, Files' "Read as Cyrillic (Windows-1251).", HEIC/AVIF/JPEG XL),
     .wt/a-komble (feat/stremio, committed: curated.js Media → Stremio =
     stremio-linux-shell; needs.rs "Needs Stremio’s streaming server to
     play", "Get the streaming server") and .wt/l2-ewe (libdvdcss,
     webp-pixbuf-loader in common.list). Arch's mpv depends on libdvdnav.
     Comments never reach a model. -->

## Play a YouTube video or a web link in ewe Player
keys: play youtube, youtube video, play a link, video link, play a url, web link, paste a link, play in ewe player, yt-dlp, vimeo, twitch
From ewe 0.46, ewe Player plays the video on a web page, from YouTube, Vimeo, Twitch, news sites and about a thousand others: press Ctrl+L (Open URL…) and paste the link, drop the link on its window, run `ewe-player <link>`, or right-click a page or a link in ewe Browser → Play in ewe Player. It gets the video out of the page with yt-dlp, shows its title and seeks like a file. A direct link to a video file (…/film.mp4) or a live stream (.m3u8) plays in every version.

"This is a web page, not a video file" means the link is a page, not a video: before ewe 0.46 ewe Player could not read pages at all, and from 0.46 it says so when it finds no video on the page (or when yt-dlp is missing: Komble installs it). Opening it in ewe Browser plays it on the site. "This video is locked" means the site protects it (DRM, like Netflix), so it plays only on the site itself; when a site sends a video in a form ewe Player cannot stream, the page offers mpv instead.
<!-- from the wave plan 2026-11-21: check the labels at release (the error titles are in a-player's player.ts; the buttons' words are not written yet) -->

## Why won't this video play?
keys: video won't play, won't this play, won't this video play, won't play, cannot play, can't play, doesn't play, does not play, no sound in the video, missing codec, codec, gst-libav, unsupported format
ewe Player says why on the page it shows instead of the video: the codec in words ("It uses HEVC and AC-3") and what is missing. ewe has shipped the codecs for nearly everything since ewe 0.44, so "needs gst-libav (FFmpeg codecs for GStreamer), which is not installed" means that package was removed: install it again with Komble or `sudo pacman -S gst-libav`. When the decoder is there and still refuses the file, the file may be damaged: Open with… tries another app (VLC from Komble plays nearly anything). Ask me why it won't play, with the file or the link, and I look at it for you.

A link that says "This is a web page, not a video file" is a page, not a video: ewe 0.46 plays those (Play a YouTube video or a web link). Before ewe 0.46, ewe Player blamed gst-libav for every link that did not play, even with gst-libav installed; that sentence was wrong.
<!-- "ask me why": Shaun's own power in the wave plan 2026-11-21 (S4, A7): check at release -->

## Video and music formats ewe plays
keys: video formats, music formats, audio formats, what formats, mkv, mp4, hevc, h265, wmv, avi, flac, alac, hardware decoding, codecs out of the box
ewe plays nearly every format out of the box (since ewe 0.44): MP4, MKV, WebM, MOV, AVI, WMV, MPEG and TS video with H.264, HEVC (H.265), VP8, VP9, AV1, MPEG-2 or MPEG-4 (DivX, Xvid) inside, and MP3, AAC, FLAC, ALAC, Opus, Vorbis, WMA, APE, AC-3 and DTS sound. On Intel and AMD graphics, H.264 and HEVC decode on the graphics chip, which saves battery. ewe Player plays them all, and so does every other app that uses GStreamer or FFmpeg.

## Stremio
keys: stremio, stremio enhanced, stremio does not play, stremio streaming server, streaming server, stremio-linux-shell
open: komble search=stremio
Install Stremio from Komble: Discover → Popular → Media → Stremio (stremio-linux-shell, Stremio's own Linux app, from the AUR; from Komble 0.24 it is in Popular, before that search for it). Its streaming server and its own player come with it, so torrents and MKV, HEVC or AC-3 streams play.

Stremio Enhanced plays nothing until it has Stremio's streaming server: on Linux it looks for the server only as a Flatpak, which ewe does not use, and that is why its videos did not play. From ewe 0.46 (Komble 0.24), its row in Komble → Installed and its page show "Needs Stremio's streaming server to play": press Get the streaming server (about 7 MB from strem.io), then restart Stremio Enhanced. Or use the official Stremio above, which needs nothing more.
<!-- from the wave plan 2026-11-21 (a-komble feat/stremio, committed, unreleased): check the labels at release -->

## Photos from an iPhone (HEIC)
keys: heic, heif, iphone photos, iphone pictures, photos from my phone, avif, jpeg xl, jxl, webp
From ewe 0.46, ewe Viewer shows HEIC and HEIF photos from an iPhone (upright, with the camera details), and ewe Files shows them in its thumbnails, quick look and preview; AVIF and JPEG XL pictures get thumbnails too, and WebP pictures show in older GTK apps' file pickers. Up to ewe 0.45, ewe Viewer says plainly that it cannot show HEIC: open it in another app, or set the iPhone to take JPEG photos (iPhone Settings → Camera → Formats → Most Compatible).
<!-- from the wave plan 2026-11-21 (a-text feat/text-images, committed, unreleased): check at release -->

## Subtitles in any language
keys: subtitles, srt, subtitle file, load subtitles, subtitle encoding, garbled subtitles, subtitles show wrong characters, mojibake
ewe Player shows subtitles from the video file and from a .srt, .vtt, .ass or .ssa file next to it with the same name (Film.ru.srt or Film.en.srt); C turns them on and off, and Load subtitles… in the subtitles menu picks another file. From ewe 0.46, subtitles in any encoding read right (Russian, Arabic, Greek, Hebrew, Turkish, Chinese, Japanese, Korean …), and the subtitles menu's Subtitle encoding fixes a wrong guess (Automatic is the default).

Text files in old encodings: from ewe 0.46, ewe Viewer reads them right too and names the encoding in its toolbar: click it to pick another, and it remembers that per file. ewe Files' quick look says "Read as Cyrillic (Windows-1251)." when a file was not UTF-8.
<!-- from the wave plan 2026-11-21 (a-text feat/text-images, committed, unreleased): check the labels at release -->

## Audio and subtitle tracks in your language
keys: audio track, audio language, subtitle language, preferred language, dubbed, original language, other audio track
When a film has several audio or subtitle tracks, ewe Player's audio and subtitles menus (in the bar at the bottom of the video) list them, and a click switches. From ewe 0.46 they are named in their own language (English, Русский, ქართული) instead of codes like rus, and ewe Player's settings have a preferred audio and subtitle language (the system's language at first), picked by itself when a file has several tracks.
<!-- from the wave plan 2026-11-21 (A6; a-player languages.ts, uncommitted): check the label at release -->

## Watch a DVD
keys: dvd, play a dvd, film dvd, encrypted dvd, libdvdcss
A DVD shows in ewe Files' places when you put it in. ewe Player opens video files, not disc menus: play the disc with VLC from Komble, or `mpv dvd://` in a terminal (mpv comes with ewe). From ewe 0.46, libdvdcss is installed, which unlocks bought film DVDs; before, add it with Komble (search libdvdcss).
<!-- libdvdcss in common.list: from the wave plan 2026-11-21 (l2-ewe 0768852, unreleased) -->

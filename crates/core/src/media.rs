//! "Why won't this play?": a file's or a link's codecs in words, and the
//! GStreamer decoders ewe Player (WebKitGTK) needs for each. Pure: the
//! daemon runs `ffprobe` and `gst-inspect-1.0 --exists`, this reads what
//! they say.
//!
//! The decoders per codec follow ewe-apps' codec matrix (AGENTS.md
//! "Codecs", 2026-11-19): without gst-libav the box plays H.264 and HEVC
//! (VA-API), VP9, AAC, MP3, Opus, Vorbis, FLAC; WMV/WMA, ALAC, AC-3, DTS
//! and MPEG-4 Part 2 need gst-libav (`avdec_*`).

use serde_json::Value;

/// A codec: its name in words and the GStreamer elements that decode it,
/// any one of them enough.
pub struct Codec {
    pub id: &'static str,
    pub words: &'static str,
    pub decoders: &'static [&'static str],
}

pub const CODECS: &[Codec] = &[
    // video
    Codec {
        id: "h264",
        words: "H.264",
        decoders: &["vah264dec", "avdec_h264", "openh264dec", "nvh264dec"],
    },
    Codec {
        id: "hevc",
        words: "HEVC (H.265)",
        decoders: &["vah265dec", "avdec_h265", "nvh265dec"],
    },
    Codec {
        id: "vp8",
        words: "VP8",
        decoders: &["vp8dec", "avdec_vp8"],
    },
    Codec {
        id: "vp9",
        words: "VP9",
        decoders: &["vp9dec", "vavp9dec", "avdec_vp9"],
    },
    Codec {
        id: "av1",
        words: "AV1",
        decoders: &["dav1ddec", "av1dec", "vaav1dec", "avdec_av1"],
    },
    Codec {
        id: "mpeg4",
        words: "MPEG-4 Part 2 (DivX, Xvid)",
        decoders: &["avdec_mpeg4"],
    },
    Codec {
        id: "msmpeg4v3",
        words: "Microsoft MPEG-4 (DivX 3)",
        decoders: &["avdec_msmpeg4"],
    },
    Codec {
        id: "msmpeg4v2",
        words: "Microsoft MPEG-4 v2",
        decoders: &["avdec_msmpeg4v2"],
    },
    Codec {
        id: "wmv1",
        words: "Windows Media Video 7",
        decoders: &["avdec_wmv1"],
    },
    Codec {
        id: "wmv2",
        words: "Windows Media Video 8",
        decoders: &["avdec_wmv2"],
    },
    Codec {
        id: "wmv3",
        words: "Windows Media Video 9",
        decoders: &["avdec_wmv3"],
    },
    Codec {
        id: "vc1",
        words: "VC-1",
        decoders: &["avdec_vc1"],
    },
    Codec {
        id: "mpeg2video",
        words: "MPEG-2 (DVD)",
        decoders: &["mpeg2dec", "vampeg2dec", "avdec_mpeg2video"],
    },
    Codec {
        id: "mpeg1video",
        words: "MPEG-1",
        decoders: &["mpeg2dec", "avdec_mpeg1video"],
    },
    Codec {
        id: "theora",
        words: "Theora",
        decoders: &["theoradec"],
    },
    Codec {
        id: "prores",
        words: "Apple ProRes",
        decoders: &["avdec_prores"],
    },
    Codec {
        id: "flv1",
        words: "Flash video (Sorenson)",
        decoders: &["avdec_flv"],
    },
    Codec {
        id: "mjpeg",
        words: "Motion JPEG",
        decoders: &["jpegdec"],
    },
    Codec {
        id: "h263",
        words: "H.263",
        decoders: &["avdec_h263"],
    },
    Codec {
        id: "rv40",
        words: "RealVideo",
        decoders: &["avdec_rv40"],
    },
    // audio
    Codec {
        id: "aac",
        words: "AAC",
        decoders: &["faad", "avdec_aac", "fdkaacdec"],
    },
    Codec {
        id: "mp3",
        words: "MP3",
        decoders: &["mpg123audiodec", "avdec_mp3"],
    },
    Codec {
        id: "mp2",
        words: "MPEG audio layer 2",
        decoders: &["mpg123audiodec", "avdec_mp2float"],
    },
    Codec {
        id: "opus",
        words: "Opus",
        decoders: &["opusdec"],
    },
    Codec {
        id: "vorbis",
        words: "Vorbis",
        decoders: &["vorbisdec"],
    },
    Codec {
        id: "flac",
        words: "FLAC",
        decoders: &["flacdec"],
    },
    Codec {
        id: "alac",
        words: "Apple Lossless (ALAC)",
        decoders: &["avdec_alac"],
    },
    Codec {
        id: "ac3",
        words: "Dolby Digital (AC-3)",
        decoders: &["a52dec", "avdec_ac3"],
    },
    Codec {
        id: "eac3",
        words: "Dolby Digital Plus (E-AC-3)",
        decoders: &["avdec_eac3"],
    },
    Codec {
        id: "dts",
        words: "DTS",
        decoders: &["dtsdec", "avdec_dca"],
    },
    Codec {
        id: "truehd",
        words: "Dolby TrueHD",
        decoders: &["avdec_truehd"],
    },
    Codec {
        id: "wmav1",
        words: "Windows Media Audio",
        decoders: &["avdec_wmav1"],
    },
    Codec {
        id: "wmav2",
        words: "Windows Media Audio",
        decoders: &["avdec_wmav2"],
    },
    Codec {
        id: "wmapro",
        words: "Windows Media Audio Pro",
        decoders: &["avdec_wmapro"],
    },
    Codec {
        id: "amr_nb",
        words: "AMR (phone audio)",
        decoders: &["amrnbdec", "avdec_amrnb"],
    },
    Codec {
        id: "cook",
        words: "RealAudio",
        decoders: &["avdec_cook"],
    },
];

/// The containers worth naming, in words.
pub fn container_words(format_name: &str) -> &'static str {
    let f = format_name.split(',').next().unwrap_or("");
    match f {
        "matroska" => "an MKV file",
        "mov" => "an MP4 or MOV file",
        "avi" => "an AVI file",
        "asf" => "a Windows Media (WMV/WMA) file",
        "mpegts" => "an MPEG transport stream",
        "mpeg" => "an MPEG file",
        "ogg" => "an Ogg file",
        "flv" => "a Flash video file",
        "webm" => "a WebM file",
        "mp3" => "an MP3 file",
        "flac" => "a FLAC file",
        "wav" => "a WAV file",
        "rm" => "a RealMedia file",
        "hls" => "a streaming playlist (HLS)",
        _ => "a media file",
    }
}

/// One stream that needs a decoder.
#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    /// `video` or `audio`
    pub kind: String,
    /// ffprobe's codec_name
    pub codec: String,
    /// the codec in words
    pub words: String,
    /// decoders that would do (empty: none needed, e.g. plain PCM)
    pub decoders: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Probe {
    pub container: String,
    pub streams: Vec<Stream>,
    pub duration_s: Option<f64>,
}

/// `ffprobe -print_format json -show_streams -show_format` as a [`Probe`]:
/// the video and audio streams (cover pictures, subtitles and data
/// left out), each with the decoders that would play it.
pub fn parse_ffprobe(v: &Value) -> Probe {
    let mut p = Probe {
        container: v["format"]["format_name"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        duration_s: v["format"]["duration"]
            .as_str()
            .and_then(|d| d.parse().ok()),
        ..Default::default()
    };
    for s in v["streams"].as_array().into_iter().flatten() {
        let kind = s["codec_type"].as_str().unwrap_or("");
        if kind != "video" && kind != "audio" {
            continue;
        }
        // an album cover is a picture, not something to decode as video
        if s["disposition"]["attached_pic"].as_i64() == Some(1) {
            continue;
        }
        let codec = s["codec_name"].as_str().unwrap_or("unknown").to_string();
        let (words, decoders) = if codec.starts_with("pcm_") {
            ("uncompressed PCM".to_string(), vec![])
        } else {
            match CODECS.iter().find(|c| c.id == codec) {
                Some(c) => (c.words.to_string(), c.decoders.to_vec()),
                None => (codec.to_uppercase(), vec![]),
            }
        };
        p.streams.push(Stream {
            kind: kind.into(),
            codec,
            words,
            decoders,
        });
    }
    p
}

/// Whether a codec is one this table knows (an unknown one may still
/// play: GStreamer is asked about it by name).
pub fn known_codec(codec: &str) -> bool {
    codec.starts_with("pcm_") || CODECS.iter().any(|c| c.id == codec)
}

/// The streams of a probe in words: "HEVC (H.265) video and DTS audio".
pub fn describe(p: &Probe) -> String {
    let mut parts: Vec<String> = Vec::new();
    for kind in ["video", "audio"] {
        let mut names: Vec<&str> = p
            .streams
            .iter()
            .filter(|s| s.kind == kind)
            .map(|s| s.words.as_str())
            .collect();
        names.dedup();
        if !names.is_empty() {
            let all = names.join(" + ");
            // "Windows Media Audio", not "Windows Media Audio audio"
            if all.to_lowercase().contains(kind) {
                parts.push(all);
            } else {
                parts.push(format!("{all} {kind}"));
            }
        }
    }
    match parts.len() {
        0 => "no video or sound I can name".into(),
        _ => parts.join(" and "),
    }
}

/// Whether a link is a web page a site shows a video on (YouTube and the
/// like), rather than a media file: no media extension on its path.
pub fn looks_like_page(url: &str) -> bool {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim_end_matches('/');
    let last = path.rsplit('/').next().unwrap_or("");
    let ext = last.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    !matches!(
        ext.as_deref(),
        Some(
            "mp4"
                | "mkv"
                | "webm"
                | "mov"
                | "avi"
                | "m4v"
                | "mp3"
                | "m4a"
                | "flac"
                | "ogg"
                | "opus"
                | "wav"
                | "m3u8"
                | "mpd"
                | "ts"
                | "wmv"
                | "aac"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_probe_in_words() {
        let v = json!({
            "format": {"format_name": "matroska,webm", "duration": "5400.1"},
            "streams": [
                {"codec_type": "video", "codec_name": "hevc"},
                {"codec_type": "audio", "codec_name": "dts"},
                {"codec_type": "audio", "codec_name": "dts"},
                {"codec_type": "subtitle", "codec_name": "subrip"},
                {"codec_type": "video", "codec_name": "mjpeg", "disposition": {"attached_pic": 1}}
            ]
        });
        let p = parse_ffprobe(&v);
        assert_eq!(container_words(&p.container), "an MKV file");
        assert_eq!(p.streams.len(), 3);
        assert_eq!(describe(&p), "HEVC (H.265) video and DTS audio");
        let wmv = parse_ffprobe(&json!({"format": {"format_name": "asf"}, "streams": [
            {"codec_type": "video", "codec_name": "wmv3"}, {"codec_type": "audio", "codec_name": "wmav2"}]}));
        assert_eq!(
            describe(&wmv),
            "Windows Media Video 9 and Windows Media Audio"
        );
        assert!(p.streams[1].decoders.contains(&"avdec_dca"));
        assert_eq!(p.duration_s, Some(5400.1));
        let wav = parse_ffprobe(&json!({"format": {"format_name": "wav"},
            "streams": [{"codec_type": "audio", "codec_name": "pcm_s16le"}]}));
        assert!(wav.streams[0].decoders.is_empty());
        assert!(known_codec("pcm_s16le") && known_codec("wmv3") && !known_codec("zzz"));
    }

    #[test]
    fn pages_and_files() {
        assert!(looks_like_page("https://www.youtube.com/watch?v=x"));
        assert!(looks_like_page("https://vimeo.com/12345"));
        assert!(!looks_like_page("https://example.com/a/film.MKV"));
        assert!(!looks_like_page(
            "https://cdn.example.com/live/index.m3u8?token=1"
        ));
    }
}

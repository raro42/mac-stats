# Downloads organizer rules

<!--
How to edit (mac-stats ignores this block):
- Add one ## Rule section per rule. Rules are checked top to bottom; the first match wins.
- Under ## Settings you can set catch_all (relative folder under your Downloads root) for files that match no rule.
- Use match_extensions: comma-separated list (no dots), or match_glob: e.g. "*.dmg"
- Partial downloads (.crdownload, .part) and .DS_Store are always skipped.
-->

## Settings

- catch_all: _Unsorted

## Rule

- match_extensions: png, jpg, jpeg, gif, webp, heic
- destination: Images

## Rule

- match_extensions: pdf
- destination: Documents/PDFs

## Rule

- match_extensions: zip, tar, gz, tgz, bz2, xz, rar, 7z
- destination: Archives

## Rule

- match_glob: "*.dmg"
- destination: Installers

## Rule

- match_extensions: mp4, mov, mkv, webm, m4v
- destination: Video

## Rule

- match_extensions: mp3, wav, flac, aac, m4a
- destination: Audio

---

ruleset_version: 1

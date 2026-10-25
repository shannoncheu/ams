# Windows x64 Portable

Extract the complete ZIP to a writable directory and run `GPUI-Music-Player.exe`.
Do not remove `portable.flag` unless switching to normal per-user storage.
No administrator rights, installation, font installation or development paths
are required. Fonts, icons and compiled shaders are embedded in the EXE.
Keep any included DLLs beside the executable.

Portable layout, created on first run:

```text
GPUI-Music-Player.exe
portable.flag
data/
  config/config.toml
  config/library.toml
  config/logs/
  state/playback.json
  state/music-shell.json
  cache/artwork/
  cache/playlist-covers/
licenses/
  THIRD_PARTY_SOURCES.zip
THIRD_PARTY_NOTICES.txt
```

`--portable` also enables this behavior. Without the flag/marker, normal platform
config/state/cache directories are used. Existing legacy client config is read
only in normal mode. Portable mode starts with its own fresh configuration.
Do not copy another machine's config/cache into a public release.

Music stays in its original location; the library records paths rather than
copying music. Moving the player does not automatically relocate music files.
Re-import missing paths after moving a library to a different computer.
The `ROMUS_LOCAL_CONFIG` advanced override, if explicitly configured, still takes
precedence for the catalog. No such override is set by the portable package.

For no transparency:

```toml
[appearance]
backdrop = "opaque"
```

`auto` attempts Win10-compatible Acrylic and falls back to slate; `transparent`
explicitly requests desktop transparency. Restart after changing this option.
Windows 11 Mica is not used.

Preview builds are unsigned. Windows 10 physical-device compatibility and audio
stability are not yet accepted; see README and the release notes.

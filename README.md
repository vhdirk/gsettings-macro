# GSettings Macro

Macro for easy GSettings key access

The main purpose of this is to reduce the risk of mistyping a key and
reduce boilerplate rust code. Furthermore, the summary and the default
of the value is included in the documentation of the getter and setter
function. This would be helpful if you use `rust-analyzer` and would
encourage documenting GSchema keys.

Schema like the following

```xml
<schemalist>
    <schema path="/io/github/seadve/test/" id="io.github.seadve.test">
        <key name="is-maximized" type="b">
            <default>false</default>
            <summary>Window maximized behaviour</summary>
            <description></description>
        </key>
        <key name="theme" type="s">
            <default>"light"</default>
            <summary>Current theme</summary>
            <description></description>
        </key>
        <key name="invalid-words" type="as">
            <default>[]</default>
            <summary>Contains invalid words</summary>
            <description></description>
        </key>
        <key name="window-width" type="i">
            <default>600</default>
            <summary>Window width</summary>
            <description>Window width</description>
        </key>
        <key name="preferred-audio-source" type="s">
            <choices>
                <choice value="microphone"/>
                <choice value="desktop-audio"/>
            </choices>
            <default>"microphone"</default>
            <summary>Preferred audio source to use in recording audio</summary>
            <description></description>
        </key>
    </schema>
</schemalist>
```

could be accessed with

```rust
use gio::prelude::*;

#[gsettings_macro::gen_settings(
    file = "./examples/test.gschema.xml",
    id = "io.github.seadve.test"
)]
pub struct Settings;

let settings = Settings::new();

settings
    .set_is_maximized(true)
    .expect("key is not writable");
assert!(settings.is_maximized());

settings
    .set_theme("dark")
    .expect("key is not writable");
assert_eq!(settings.theme(), "dark");

settings
    .set_invalid_words(&["invalid", "words"])
    .expect("key is not writable");
assert_eq!(settings.invalid_words(), vec!["invalid", "words"]);

settings
    .set_window_width(30_000)
    .expect("key is not writable");
settings.window_width();
assert_eq!(settings.window_width(), 30_000);

settings
    .set_preferred_audio_source(PreferredAudioSource::DesktopAudio)
    .expect("key is not writable");
assert_eq!(
    settings.preferred_audio_source(),
    PreferredAudioSource::DesktopAudio
);
```

## Known issue

* Failing to compile on unknown variants
  * Maybe skip generating them
* Not updating when the gschema file is modified
  * Use hacks like `include_str!`

## Todo list

* Usage documentation
* Add enum and flags support
* Add other common types support (`a{ss}`, etc.)

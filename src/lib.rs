mod generators;
mod imp;
mod schema;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

/// Macro for typesafe [`gio::Settings`] key access.
///
/// The macro's main purpose is to reduce the risk of mistyping a key,
/// using the wrong method to access values, inputing incorrect values,
/// and reduce boilerplate Rust code. Additionally, the summary, the
/// description, and the default of the value are included in the
/// documentation of each generated methods. This would be beneficial
/// if you use tools like [`rust-analyzer`](https://rust-analyzer.github.io/).
///
/// **⚠️ IMPORTANT ⚠️**
///
/// `gio` needs to be in scope, so unless it's one of the direct crate
/// dependencies, you need to import it because `gen_settings` is using
/// it internally. For example:
///
/// ```rust,ignore
/// use gtk::gio;
/// ```
///
/// ### Example
///
/// ```rust,ignore
/// use gsettings_macro::gen_settings;
///
/// #[gen_settings(file = "./tests/io.github.seadve.test.gschema.xml")]
/// pub struct ApplicationSettings;
///
/// let settings = ApplicationSettings::new("io.github.seadve.test");
///
/// // `i` DBus type
/// settings.set_window_width(100);
/// assert_eq!(settings.window_width(), 100)
///
/// // enums
/// settings.set_alert_sound(AlertSound::Glass);
/// assert_eq!(settings.alert_sound(), AlertSound::Glass);
///
/// // bitflags
/// settings.set_space_style(SpaceStyle::BEFORE_COLON | SpaceStyle::BEFORE_COMMA);
/// assert_eq!(
///     settings.space_style(),
///     SpaceStyle::BEFORE_COLON | SpaceStyle::BEFORE_COMMA
/// );
/// ```
///
/// Note: The file path is relative to the project root or where the
/// `Cargo.toml` file is located.
///
/// ### Generated methods
///
/// The procedural macro generates code for each key for following
/// [`gio::Settings`] methods:
///
/// * `set` -> `set_#key`, which panics when writing in a readonly
/// key, and `try_set_#key`, which behaves the same as the original method.
/// * `get` -> `get_#key`
/// * `connect_changed` -> `connect_#key_changed`
/// * `bind` -> `bind_#key`
/// * `create_action` -> `create_#key_action`
///
/// ### Known DBus type codes
///
/// The setter and getter methods has the following argument and
/// return type, depending on the key's DBus type code.
///
/// | DBus type code | argument type  | return type    |
/// | -------------- | -------------- | -------------- |
/// | b              | `bool`         | `bool`         |
/// | i              | `i32`          | `i32`          |
/// | u              | `u32`          | `u32`          |
/// | x              | `i64`          | `i64`          |
/// | t              | `u64`          | `u64`          |
/// | d              | `f64`          | `f64`          |
/// | (ii)           | `(i32, i32`)   | `(i32, i32`)   |
/// | as             | `&[&str]`      | `Vec<String>`  |
/// | s *            | `&str`         | `String`       |
///
/// \* If the key of DBus type code `s` has no choice
/// specified in the GSchema, the argument and return types stated
/// in the table would be true. Otherwise, it will generate an
/// enum, like described in the next section, and use it as argument
/// and return type, instead of `&str` and `String` respectively.
///
/// The code would fail to compile if the DBus type code is not
/// included above. However, it is possible to skip generating a
/// specific key or DBus type code by using `#[gen_settings_skip]`
/// or define a custom argument and return types using `#[gen_settings_define]`.
/// Usage would be further explained in the following sections.
///
/// ### Enums and Flags
///
/// It will also automatically generate enums or flags. If it is
/// an enum, it would generated a normal Rust enum with each nick
/// specified in the GSchema converted to pascal case as an enum variant.
/// The enum would implement both [`ToVariant`](gio::glib::ToVariant)
/// and [`FromVariant`](gio::glib::FromVariant), [`Clone`],
/// [`Hash`], [`PartialEq`], [`Eq`], [`PartialOrd`], and [`Ord`]. On
/// the other hand, if it is a flag, it would generate bitflags
/// same as the bitflags generated by the [`bitflags`] macro with each
/// nick specified in the GSchema converted to screaming snake case as
/// a const flag.
///
/// The generated types, enum or bitflags, would have the same
/// visibility and scope with the generated struct.
///
/// ### Skipping generating code
///
/// This would be helpful if you want to have full control
/// with the key without the macro intervening. For example:
///
/// ```rust,ignore
/// use gsettings_macro::gen_settings;
///
/// #[gen_settings(
///     file = "./tests/io.github.seadve.test.gschema.xml",
///     id = "io.github.seadve.test"
/// )]
/// // Skip generating code for keys with DBus type `(ss)`
/// #[gen_settings_skip(signature = "(ss)")]
/// // Skip generating code for keys with name `some-key-name`
/// #[gen_settings_skip(key_name = "some-key-name")]
/// pub struct Settings;
///
/// impl Settings {
///     pub fn set_some_key_name(value: &std::path::Path) {
///         // some code here
///     }
/// }
/// ```
///
/// ### Defining custom types
///
/// ```rust,ignore
/// use gsettings_macro::gen_settings;
///
/// use std::path::{Path, PathBuf};
///
/// #[gen_settings(file = "./tests/io.github.seadve.test.gschema.xml")]
/// // Define custom argument and return types for keys with type `(ss)`
/// #[gen_settings_define(
///     signature = "(ss)",
///     arg_type = "(&str, &str)",
///     ret_type = "(String, String)"
/// )]
/// // Define custom argument and return types for key with name `cache-dir`
/// #[gen_settings_define(key_name = "cache-dir", arg_type = "&Path", ret_type = "PathBuf")]
/// pub struct SomeAppSettings;
///
/// let settings = SomeAppSettings::new("io.github.seadve.test");
///
/// settings.set_cache_dir(Path::new("/some_dir"));
/// assert_eq!(settings.cache_dir(), PathBuf::from("/some_dir"));
///
/// settings.set_string_tuple(("hi", "hi2"));
/// assert_eq!(settings.string_tuple(), ("hi".into(), "hi2".into()));
/// ```
///
/// The type specified in `arg_type` and `ret_type` has to be on scope or
/// you can specify the full path.
///
/// If you somehow don't want an enum argument and return types for `s` DBus
/// type code with choices. You can also use this to override that behavior.
///
/// Note: The type has to implement both [`ToVariant`](gio::glib::ToVariant)
/// and [`FromVariant`](gio::glib::FromVariant) or it would fail to compile.
///
/// ### Default trait
///
/// The schema id can be specified as an attribute, making it implement
/// [`Default`] and create a `new` constructor without arguments.
/// Otherwise, it won't implement [`Default`] and would require the
/// schema id as an argument in the constructor.
///
/// The following is an example of defining the `id` attribute in the macro:
///
/// ```rust,ignore
/// use gsettings_macro::gen_settings;
///
/// #[gen_settings(
///     file = "./tests/io.github.seadve.test.gschema.xml",
///     id = "io.github.seadve.test"
/// )]
/// pub struct ApplicationSettings;
///
/// // The id is specified above so it is not needed
/// // to specify it in the constructor.
/// let settings = ApplicationSettings::new();
/// let another_instance = ApplicationSettings::default();
/// ```
///
/// [`gio::Settings`]: https://docs.rs/gio/0.15/gio/struct.Settings.html
/// [`gio::glib::ToVariant`]: https://docs.rs/glib/0.15/glib/variant/trait.ToVariant.html
/// [`gio::glib::FromVariant`]: https://docs.rs/glib/0.15/glib/variant/trait.FromVariant.html
/// [`bitflags`]: https://docs.rs/bitflags/1.0/bitflags/macro.bitflags.html
#[proc_macro_attribute]
#[proc_macro_error]
pub fn gen_settings(attr: TokenStream, item: TokenStream) -> TokenStream {
    imp::gen_settings(attr, item)
}

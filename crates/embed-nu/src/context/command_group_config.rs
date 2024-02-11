macro_rules! command_group_config {

    ($(#[doc=$doc:literal] $group:ident),*) => {

        /// Enables or disables certain command groups
        #[derive(Clone, Debug, Default)]
        pub struct CommandGroupConfig {
            $(pub(crate) $group: bool,)*
        }

        impl CommandGroupConfig {
            /// Enables all commands
            pub fn all_groups(mut self, enabled: bool) -> Self {
                $(
                    self.$group = enabled;
                )*

                self
            }

            $(
            paste::item! {
                #[doc=$doc]
                #[inline]
                pub fn [< $group _group>](mut self, enabled: bool) -> Self {
                    self.$group = enabled;

                    self
                }
            }
            )*
        }
    }
}

command_group_config!(
    /// Enables core commands
    core,
    /// Enables debug commands
    debug,
    /// Enables filter commands
    filter,
    /// Enables chart commands
    chart,
    /// Enables misc commands
    misc,
    /// Enables commands that allow path manipulation
    path,
    /// Enables system commands
    system,
    /// Enables commands to manipulate strings
    string,
    /// Enables commands to manipulate bits
    bit,
    /// Enables commands to manipulate bytes
    byte,
    /// Enables commands that allow file system operations
    file_system,
    /// Enables commands that allow using shell features like ansi colors
    platform,
    /// Enables commands that allow datetime manipulation
    date,
    /// Enables commands that allow creating and switching between nu shell instances
    shell,
    /// Enables commands that allow parsing from one data format to another
    format,
    /// Enables commands that allow displaying data in certain viewers like a table or grid
    viewer,
    /// Enables commands that allow converting from one data format to another
    conversion,
    /// Enables commands that allow manipulating environment variables
    environment,
    /// Enables math related commands
    math,
    /// Enables commands that allow networking
    network,
    /// Enables commands that generate random values
    random,
    /// Enables commands that generate values for a given input
    generator,
    /// Enables commands that work with hash sums
    hash,
    /// Enables commands that are still experimental like `is-admin` and `view-source`
    experimental
);

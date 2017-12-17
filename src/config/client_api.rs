

use super::DisplayConfig;

macro_rules! config_client_api {
    ( $name: ident ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            display_config: DisplayConfig,
        }

        impl $name {
            pub(super) fn new(display_config: DisplayConfig) -> $name {
                $name {
                    display_config
                }
            }

            pub fn display_config(&self) -> &DisplayConfig {
                &self.display_config
            }

            pub(crate) fn into_display_config(self) -> DisplayConfig {
                self.display_config
            }
        }
    };
}

config_client_api!(ConfigOpenGL);
config_client_api!(ConfigOpenGLES);
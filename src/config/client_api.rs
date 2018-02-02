use super::DisplayConfig;

use platform::Platform;

use EGLHandle;

macro_rules! config_type {
    ( $name: ident ) => {
        #[derive(Debug, Clone)]
        pub struct $name<P: Platform> {
            display_config: DisplayConfig<P>,
        }

        impl<P: Platform> $name<P> {
            pub(crate) fn new(display_config: DisplayConfig<P>) -> Self {
                $name {
                    display_config
                }
            }

            pub fn display_config(&self) -> &DisplayConfig<P> {
                &self.display_config
            }

            pub fn egl_handle(&self) -> &EGLHandle {
                self.display_config.egl_handle()
            }
        }
    };
}

config_type!(ConfigOpenGL);
config_type!(ConfigOpenGLES);

config_type!(ConfigWindow);

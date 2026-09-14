#[cfg(test)]
mod tests {
    #[test]
    fn provider_dispatch_is_available() {
        let provider = crate::platform::create_provider();
        assert!(provider
            .check_permission(crate::core::model::Scope::User)
            .unwrap());
    }
}

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use crate::core::traits::EnvProvider;

#[cfg(unix)]
use unix::UnixProvider;
#[cfg(windows)]
use windows::WindowsProvider;

pub fn create_provider() -> Box<dyn EnvProvider + Send + Sync> {
    #[cfg(windows)]
    {
        Box::new(WindowsProvider::new())
    }
    #[cfg(unix)]
    {
        Box::new(UnixProvider::new())
    }
}

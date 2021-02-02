fn main() {
    windows::build!(
      windows::media::playback::MediaPlayer
      windows::media::core::MediaSource
      windows::foundation::{Uri, Size}
      windows::foundation::numerics::Vector2
      windows::ui::composition::{Compositor, SpriteVisual}
      windows::ui::composition::desktop::DesktopWindowTarget
      windows::system::DispatcherQueueController
      windows::win32::system_services::CreateDispatcherQueueController
      windows::win32::winrt::ICompositorDesktopInterop
    );
}

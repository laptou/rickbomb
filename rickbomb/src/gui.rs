use log::*;
use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};
use winapi::winrt::roapi::{RoInitialize, RoUninitialize, RO_INIT_SINGLETHREADED};
use winit::{
    event::{self, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use crate::interop::create_dispatcher_queue_controller_for_current_thread;
use crate::window_target::CompositionDesktopWindowTargetSource;

use bindings::windows::foundation::numerics::Vector2;
use bindings::windows::foundation::{Size, TypedEventHandler, Uri};
use bindings::windows::media::core::MediaSource;
use bindings::windows::media::playback::MediaPlayer;
use bindings::windows::ui::composition::Compositor;

pub fn rickroll(video_path: &PathBuf) -> bindings::windows::Result<()> {
    info!("displaying rick");

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rick")
        .with_maximized(true)
        .with_always_on_top(true)
        .with_decorations(false)
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    trace!("window and event loop created");

    match unsafe { RoInitialize(RO_INIT_SINGLETHREADED) } {
        winapi::shared::winerror::S_OK | winapi::shared::winerror::S_FALSE => {}
        hr => panic!("RoInitialize failed: 0x{:08x}", hr),
    };

    trace!("RoInitialize succeeded");

    let size = window.inner_size().cast();

    // dispatcher will stay alive as long the event loop stays alive
    let dispatcher = create_dispatcher_queue_controller_for_current_thread()?;

    // we need a dispatcher in order to construct a compositor
    // this ctor will fail if the dispatcher is cleaned up before it is called
    // which means that it will fail if event_loop is dropped
    let compositor = Compositor::new()?;

    let target = window.create_window_target(&compositor, true)?;

    trace!("compositor created");

    let player = MediaPlayer::new()?;
    let uri = Uri::create_uri(video_path.to_str().unwrap())?;
    let source = MediaSource::create_from_uri(uri)?;
    player.set_source(source)?;
    player.set_surface_size(Size {
        width: size.width,
        height: size.height,
    })?;

    let surface = player.get_surface(&compositor)?;

    let visual = compositor.create_sprite_visual()?;
    let brush = compositor.create_surface_brush_with_surface(surface.composition_surface()?)?;
    visual.set_size(Vector2 {
        x: size.width,
        y: size.height,
    })?;
    visual.set_brush(brush)?;
    target.set_root(visual)?;

    trace!("media player initialized");

    player.play()?;

    let is_playing = Arc::new(AtomicBool::new(true));

    player.media_ended(TypedEventHandler::new({
        let is_playing = is_playing.clone();
        move |_, _| {
            is_playing.store(false, std::sync::atomic::Ordering::Relaxed);
            info!("media ended");
            Ok(())
        }
    }))?;

    trace!("event loop running");

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if !is_playing.load(std::sync::atomic::Ordering::Relaxed) {
            debug!("event loop exiting");
            *control_flow = ControlFlow::Exit;
            return;
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                // ignore
            }
            _ => {}
        }
    });

    debug!("event loop exited");

    unsafe {
        RoUninitialize();
    }

    Ok(())
}

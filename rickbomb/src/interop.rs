use bindings::windows;
use bindings::windows::system::DispatcherQueueController;
use bindings::windows::win32::system_services::{
    CreateDispatcherQueueController, DispatcherQueueOptions, DISPATCHERQUEUE_THREAD_APARTMENTTYPE,
    DISPATCHERQUEUE_THREAD_TYPE,
};

pub fn create_dispatcher_queue_controller(
    thread_type: DISPATCHERQUEUE_THREAD_TYPE,
    apartment_type: DISPATCHERQUEUE_THREAD_APARTMENTTYPE,
) -> windows::Result<DispatcherQueueController> {
    let options = DispatcherQueueOptions {
        dw_size: std::mem::size_of::<DispatcherQueueOptions>() as u32,
        thread_type,
        apartment_type,
    };
    unsafe {
        let mut result = None;
        CreateDispatcherQueueController(options, &mut result).and_some(result)
    }
}

pub fn create_dispatcher_queue_controller_for_current_thread(
) -> windows::Result<DispatcherQueueController> {
    create_dispatcher_queue_controller(
        DISPATCHERQUEUE_THREAD_TYPE::DQTYPE_THREAD_CURRENT,
        DISPATCHERQUEUE_THREAD_APARTMENTTYPE::DQTAT_COM_NONE,
    )
}

#[derive(Debug, Clone)]
pub struct WindowsError(bindings::windows::Error);

impl std::fmt::Display for WindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "windows error: {:?}", self)
    }
}

impl std::error::Error for WindowsError {}

impl From<bindings::windows::Error> for WindowsError {
    fn from(win_err: bindings::windows::Error) -> Self {
        WindowsError(win_err)
    }
}

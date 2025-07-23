use std::{
    any::Any,
    panic::{self, PanicHookInfo},
};

use super::ThreadGroup;

pub fn setup_terminate_group_panic_hook(thread_group: &ThreadGroup) {
    let original_hook = panic::take_hook();
    let terminate_group_channel_tx = thread_group.get_terminate_group_channel_tx();
    let panic_handler = move |hook_info: &PanicHookInfo| {
        let _ = terminate_group_channel_tx.send(());
        original_hook(hook_info);
    };
    panic::set_hook(Box::new(panic_handler));
}

pub fn any_to_string(any: &Box<dyn Any + Send>) -> String {
    if let Some(message) = any.downcast_ref::<&'static str>() {
        message.to_string()
    } else if let Some(message) = any.downcast_ref::<String>() {
        message.clone()
    } else {
        format!("{any:?}")
    }
}

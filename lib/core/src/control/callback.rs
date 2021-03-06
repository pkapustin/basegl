//! Definitions of callback handling utilities.

use crate::prelude::*;



// ================
// === Callback ===
// ================

/// Immutable callback type.
pub trait CallbackFn = Fn() + 'static;

/// Immutable callback object.
pub type Callback = Box<dyn CallbackFn>;

/// Callback object smart constructor.
#[allow(non_snake_case)]
pub fn Callback<F:CallbackFn>(f:F) -> Callback {
    Box::new(f)
}

/// Mutable callback type.
pub trait CallbackMutFn = FnMut() + 'static;

/// Mutable callback object.
pub type CallbackMut = Box<dyn CallbackMutFn>;

/// Mutable callback type with one parameter.
pub trait CallbackMut1Fn<T> = FnMut(T) + 'static;

/// Mutable callback object with one parameter.
pub type CallbackMut1<T> = Box<dyn CallbackMut1Fn<T>>;



// ======================
// === CallbackHandle ===
// ======================

/// Handle to a callback. When the handle is dropped, the callback is removed.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackHandle {
    rc: Rc<()>
}

impl CallbackHandle {

    /// Create a new handle.
    pub fn new() -> Self {
        default()
    }

    /// Create guard for this handle.
    pub fn guard(&self) -> Guard {
        let weak = Rc::downgrade(&self.rc);
        Guard {weak}
    }

    /// Forget the handle. Warning! You would not be able to stop the callback after performing this
    /// operation.
    pub fn forget(self) {
        std::mem::forget(self)
    }
}

/// CallbackHandle's guard. Used to check if the handle is still valid.
pub struct Guard {
    weak: Weak<()>
}

impl Guard {
    /// Checks if the handle is still valid.
    pub fn exists(&self) -> bool {
        self.weak.upgrade().is_some()
    }
}



// ========================
// === CallbackRegistry ===
// ========================

/// Registry gathering callbacks. Each registered callback is assigned with a handle. Callback and
/// handle lifetimes are strictly connected. As soon a handle is dropped, the callback is removed
/// as well.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackRegistry {
    #[derivative(Debug="ignore")]
    callback_list: Vec<(Guard, CallbackMut)>
}

impl CallbackRegistry {

    /// Adds new callback and returns a new handle for it.
    pub fn add<F:CallbackMutFn>(&mut self, callback:F) -> CallbackHandle {
        let callback = Box::new(callback);
        let handle   = CallbackHandle::new();
        let guard    = handle.guard();
        self.callback_list.push((guard, callback));
        handle
    }

    /// Fires all registered callbacks.
    pub fn run_all(&mut self) {
        self.clear_unused_callbacks();
        self.callback_list.iter_mut().for_each(|(_,callback)| callback());
    }

    /// Checks all registered callbacks and removes the ones which got dropped.
    fn clear_unused_callbacks(&mut self) {
        self.callback_list.retain(|(guard,_)| guard.exists());
    }
}

/// Registry gathering callbacks. Each registered callback is assigned with a handle. Callback and
/// handle lifetimes are strictly connected. As soon a handle is dropped, the callback is removed
/// as well.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackRegistry1<T:Copy> {
    #[derivative(Debug="ignore")]
    callback_list: Vec<(Guard, CallbackMut1<T>)>
}

impl<T:Copy> CallbackRegistry1<T> {
    /// Adds new callback and returns a new handle for it.
    pub fn add<F:CallbackMut1Fn<T>>(&mut self, callback:F) -> CallbackHandle {
        let callback = Box::new(callback);
        let handle   = CallbackHandle::new();
        let guard    = handle.guard();
        self.callback_list.push((guard, callback));
        handle
    }

    /// Fires all registered callbacks.
    pub fn run_all(&mut self, t:T) {
        self.clear_unused_callbacks();
        self.callback_list.iter_mut().for_each(move |(_,callback)| callback(t));
    }

    /// Checks all registered callbacks and removes the ones which got dropped.
    fn clear_unused_callbacks(&mut self) {
        self.callback_list.retain(|(guard,_)| guard.exists());
    }
}

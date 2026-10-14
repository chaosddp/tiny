use std::{cell::RefCell, collections::HashMap};

use log::debug;
use mlua::prelude::*;

pub struct EventHub {
    inner: LuaTable,
    rs_callbacks: RefCell<HashMap<String, Vec<Box<dyn Fn(&LuaValue) -> bool>>>>,
    rs_once_callbacks: RefCell<HashMap<String, Vec<Box<dyn FnOnce(&LuaValue) -> bool>>>>,
}

impl EventHub {
    pub fn new(inner: LuaTable) -> Self {
        Self {
            inner,
            rs_callbacks: RefCell::new(HashMap::new()),
            rs_once_callbacks: RefCell::new(HashMap::new()),
        }
    }

    pub fn register<F>(&self, event: &str, callback: F)
    where
        F: Fn(&LuaValue) -> bool + 'static,
    {
        let mut rs_callbacks = self.rs_callbacks.borrow_mut();

        if rs_callbacks.contains_key(event) {
            rs_callbacks
                .get_mut(event)
                .unwrap()
                .push(Box::new(callback));
        } else {
            rs_callbacks.insert(event.to_string(), vec![Box::new(callback)]);
        }
    }

    pub fn register_once<F>(&self, event: &str, callback: F)
    where
        F: FnOnce(&LuaValue) -> bool + 'static,
    {
        let mut rs_once_callbacks = self.rs_once_callbacks.borrow_mut();

        if rs_once_callbacks.contains_key(event) {
            rs_once_callbacks
                .get_mut(event)
                .unwrap()
                .push(Box::new(callback));
        } else {
            rs_once_callbacks.insert(event.to_string(), vec![Box::new(callback)]);
        }
    }

    pub fn send(&self, event: &str, payload: LuaValue) {
        let mut should_sub_to_lua = true;

        let rs_callbacks = self.rs_callbacks.borrow();

        if rs_callbacks.contains_key(event) {
            let callbacks = rs_callbacks.get(event).unwrap();

            for callback in callbacks {
                if callback(&payload) {
                    should_sub_to_lua = false;

                    break;
                }
            }
        }

        let mut rs_once_callbacks = self.rs_once_callbacks.borrow_mut();

        // no matter if there is any once callback being called, we will clean it
        if rs_once_callbacks.contains_key(event) {
            let callbacks = rs_once_callbacks.get_mut(event).unwrap();

            let mut stop_sub = false;

            while callbacks.len() > 0 {
                let callback = callbacks.remove(0);

                if !stop_sub && callback(&payload) {
                    should_sub_to_lua = false;
                    stop_sub = true;
                }
            }
        }

        if should_sub_to_lua {
            if let Err(error) = self.inner.call_method::<()>("sub", payload) {
                debug!(
                    "Fail to sub event {} to lua side: {}",
                    event,
                    error.to_string()
                );
            }
        }
    }
}

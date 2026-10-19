use std::{
    collections::HashMap,
    sync::{
        Arc, Condvar, Mutex,
        atomic::AtomicUsize,
        mpsc::{self, Receiver, Sender},
    },
};

use log::{debug, error};
use mlua::prelude::*;

use crate::lua::thread::value::LuaThreadValue;

#[derive(Debug)]
pub struct ChannelInner {
    sender: Sender<LuaThreadValue>,
    receiver: Mutex<Receiver<LuaThreadValue>>,
    count: AtomicUsize,
    consume_lock: Mutex<()>,
    consume_cv: Condvar,
}

impl ChannelInner {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            sender,
            receiver: Mutex::new(receiver),
            count: AtomicUsize::new(0),
            consume_lock: Mutex::new(()),
            consume_cv: Condvar::new(),
        }
    }

    pub fn send(&self, value: LuaThreadValue) -> LuaResult<()> {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if self.sender.send(value).is_err() {
            self.count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

            error!("fail to send");

            return Err(LuaError::RuntimeError("channel closed".into()));
        }

        // we block here until the channel is empty
        let mut guard = self.consume_lock.lock().unwrap();

        while self.count.load(std::sync::atomic::Ordering::SeqCst) > 0 {
            debug!("locking, waiting for channel empty");
            guard = self.consume_cv.wait(guard).unwrap();
        }

        debug!("send completed");

        Ok(())
    }

    pub fn recv(&self) -> LuaResult<Option<LuaThreadValue>> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

        match receiver.try_recv() {
            Ok(value) => {
                let _ = self.count.try_update(
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                    |count| Some(count.saturating_sub(1)),
                );

                let _guard = self.consume_lock.lock().unwrap();

                self.consume_cv.notify_all();

                Ok(Some(value))
            }
            Err(_) => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        let receiver = self.receiver.lock().unwrap();

        while receiver.try_recv().is_ok() {
            // clear the items in channel
        }
    }
}

impl Default for ChannelInner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct LuaChannelRegistry {
    channels: Mutex<HashMap<String, Arc<ChannelInner>>>,
}

impl LuaChannelRegistry {
    pub fn get_or_create(&self, name: &str) -> Arc<ChannelInner> {
        let mut channels = self.channels.lock().unwrap();

        channels
            .entry(name.to_owned())
            .or_insert_with(|| Arc::new(ChannelInner::default()))
            .clone()
    }
}

#[derive(Clone)]
pub struct LuaChannel(pub Arc<ChannelInner>);

#[allow(dead_code)]
impl LuaChannel {
    pub fn new() -> Self {
        Self(Arc::new(ChannelInner::default()))
    }
}

impl LuaUserData for LuaChannel {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("send", |_, channel, value: LuaThreadValue| {
            channel.0.send(value)?;

            Ok(())
        });

        methods.add_method("recv", |_, channel, ()| {
            let value = channel.0.recv()?;

            Ok(value)
        });
    }
}

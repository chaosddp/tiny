use std::{
    io::{self},
    sync::mpsc::{Receiver, RecvError, RecvTimeoutError, Sender, channel},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyModifiers, MouseEvent};
use mlua::prelude::*;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, List},
};

use ratatui_textarea::TextArea;
use serde::{Deserialize, Serialize};
use tiny::{core::TinyResult, lua::agent::TinyAgent};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub index: u32,
    pub arguments: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chunk {
    pub id: Option<String>,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub tool_call: Option<ToolCall>,
    pub finish_reason: Option<String>,
    pub tool_result: Option<String>,
    pub user_message: Option<String>,
    pub end: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Chunk(Chunk),
    Message(Message),
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug)]
pub struct AppEventHandler {
    #[allow(dead_code)]
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl AppEventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = channel();

        let handler = {
            let sender = sender.clone();

            thread::spawn(move || {
                let mut last_tick = Instant::now();

                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("unable to pool for event") {
                        let ret = match event::read().expect("unable to read event") {
                            CrosstermEvent::Key(e) => sender.send(AppEvent::Key(e)),
                            CrosstermEvent::Mouse(e) => sender.send(AppEvent::Mouse(e)),
                            CrosstermEvent::Resize(w, h) => sender.send(AppEvent::Resize(w, h)),
                            _ => Ok(()),
                        };

                        if ret.is_err() {
                            break;
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender
                            .send(AppEvent::Tick)
                            .expect("fail to send tick event");

                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn next(&self) -> Result<AppEvent, RecvError> {
        Ok(self.receiver.recv()?)
    }

    pub fn try_next(&self, timeout: Duration) -> Result<AppEvent, RecvTimeoutError> {
        Ok(self.receiver.recv_timeout(timeout)?)
    }
}

struct Wrapper {
    user_input_receiver: Receiver<String>,
    event_sender: Sender<AppEvent>,
}

impl Wrapper {
    pub fn new(user_input_receiver: Receiver<String>, event_sender: Sender<AppEvent>) -> Self {
        Self {
            user_input_receiver,
            event_sender,
        }
    }
}

impl LuaUserData for Wrapper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("input", |_, wrapper, ()| {
            let user_input = wrapper
                .user_input_receiver
                .recv()
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

            Ok(user_input)
        });

        methods.add_method_mut("chunk", |lua, app, chunk_table: LuaTable| {
            let chunk: Chunk = lua.from_value(LuaValue::Table(chunk_table))?;

            app.event_sender.send(AppEvent::Chunk(chunk)).unwrap();

            Ok(())
        });

        methods.add_method_mut("message", |_, _, _: LuaTable| {
            // println!("{}", message.get::<String>("content")?);

            Ok(())
        });

        methods.add_method_mut("show_reasoning", |_, _, _: bool| Ok(()));
    }
}

fn run_lua_env(
    user_input_receiver: Receiver<String>,
    event_sender: Sender<AppEvent>,
) -> TinyResult<()> {
    let agent = TinyAgent::new();

    agent.init()?;

    let lua = agent.weak().upgrade();

    lua.globals().set(
        "DefaultWrapper",
        Wrapper::new(user_input_receiver, event_sender),
    )?;

    agent.run()?;

    Ok(())
}

#[derive(Debug)]
struct App<'a> {
    event_handler: AppEventHandler,
    textarea: TextArea<'a>,
    messages: Vec<Message>,
    user_input_sender: Sender<String>,
    event_receiver: Receiver<AppEvent>,
    input_enabled: bool,
    current_chunk_id: String,
}

impl<'a> App<'a> {
    fn new(
        tick_rate: u64,
        user_input_sender: Sender<String>,
        event_receiver: Receiver<AppEvent>,
    ) -> Self {
        Self {
            event_handler: AppEventHandler::new(tick_rate),
            textarea: Default::default(),
            messages: Default::default(),
            user_input_sender: user_input_sender,
            event_receiver: event_receiver,
            input_enabled: true,
            current_chunk_id: String::new(),
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Ok(event) = self.event_handler.try_next(Duration::from_millis(1)) {
                match event {
                    AppEvent::Key(e) => {
                        if e.modifiers.contains(KeyModifiers::SHIFT) && e.code == KeyCode::Enter {
                            self.textarea.input(e);
                            continue;
                        }

                        if e.code == KeyCode::Enter && e.is_release() {
                            if self.input_enabled && self.textarea.lines().len() > 0 {
                                let text = self.textarea.lines().join("\n");
                                let trim_text = text.trim();

                                if trim_text.len() > 0 {
                                    self.messages.push(Message {
                                        role: "user".to_string(),
                                        content: trim_text.to_string(),
                                    });

                                    self.user_input_sender.send(trim_text.to_string()).unwrap();

                                    self.input_enabled = false;

                                    self.textarea.clear();
                                }
                            }

                            continue;
                        }

                        if e.code == KeyCode::Esc {
                            break;
                        }

                        // `TextArea::input` can directly handle key events from backends and update the editor state
                        self.textarea.input(e);
                    }
                    _ => {}
                }
            }

            if let Ok(evt) = self.event_receiver.recv_timeout(Duration::from_millis(1)) {
                match evt {
                    AppEvent::Chunk(chunk) => {
                        // we ignore tool result here
                        if let Some(content) = chunk.content
                            && content.len() > 0
                        {
                            if let Some(chunk_id) = chunk.id {
                                if self.current_chunk_id != chunk_id {
                                    self.current_chunk_id = chunk_id;

                                    self.messages.push(Message {
                                        role: "assistant".to_string(),
                                        content: content,
                                    });
                                } else if let Some(msg) = self.messages.last_mut() {
                                    msg.content.push_str(&content);
                                }
                            }
                        }

                        if let Some(is_chunk_end) = chunk.end {
                            if is_chunk_end {
                                self.input_enabled = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let [messages_area, input_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(5)]).areas(frame.area());

        self.render_input(frame, input_area);
        self.render_messages(frame, messages_area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title_top("User input");

        frame.render_widget(block, area);
        frame.render_widget(
            &self.textarea,
            Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2),
        );
    }

    fn render_messages(&self, frame: &mut Frame, area: Rect) {
        let messages = self
            .messages
            .iter()
            .map(|message| format!("{}: {}", message.role, message.content));

        let messages = List::new(messages).block(Block::bordered().title("Messages"));

        frame.render_widget(messages, area);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (user_input_sender, user_input_receiver) = channel();
    let (event_sender, event_receiver) = channel();

    let env_joint = thread::spawn(move || run_lua_env(user_input_receiver, event_sender.clone()));

    let app = App::new(100, user_input_sender, event_receiver);

    let mut terminal = ratatui::init();
    app.run(&mut terminal)?;
    ratatui::restore();

    let _ = env_joint.join();

    Ok(())
}

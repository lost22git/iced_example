#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use iced::keyboard::{KeyCode, Modifiers};

use iced::widget::{button, container, radio, row, text};

use iced::window::{Icon, Mode};
use iced::{
    executor, keyboard, subscription, theme, window, Application, Command, Element, Event, Length,
    Settings,
};
use image::GenericImageView;

pub fn main() -> iced::Result {
    App::run(app_settings())
}

fn app_settings() -> Settings<()> {
    Settings {
        exit_on_close_request: false,
        window: window::Settings {
            transparent: true,
            size: (600, 370),
            icon: Some(load_logo_icon()),
            ..Default::default()
        },
        default_font: Some(include_bytes!("../微软雅黑.ttc")),
        ..Default::default()
    }
}

fn load_logo_icon() -> Icon {
    let logo_raw_bytes = include_bytes!("../logo.png");
    let logo_rgba =
        image::load_from_memory_with_format(logo_raw_bytes, image::ImageFormat::Png).unwrap();
    let (logo_w, logo_h) = logo_rgba.dimensions();
    let logo_icon = Icon::from_rgba(logo_rgba.as_bytes().to_vec(), logo_w, logo_h).unwrap();
    logo_icon
}

#[derive(Debug)]

struct AppState {
    dark_mode: Option<bool>,
    scale_factor: f64,
    show_exit_dialog: bool,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            dark_mode: Some(true),
            scale_factor: Self::ZOOM_DEFAULT,
            show_exit_dialog: false,
        }
    }
}
impl AppState {
    // zoom 参数
    const ZOOM_DELTA: f64 = 0.1;
    const ZOOM_DEFAULT: f64 = 1.;
    fn zoom_in(&mut self) {
        self.scale_factor += Self::ZOOM_DELTA;
    }

    fn zoom_out(&mut self) {
        self.scale_factor = f64::max(self.scale_factor - Self::ZOOM_DELTA, Self::ZOOM_DELTA);
    }

    fn zoom_reset(&mut self) {
        self.scale_factor = Self::ZOOM_DEFAULT;
    }
}

#[derive(Debug, Default)]
struct App {
    state: AppState,
}

impl App {
    fn handle_key_event(&mut self, event: keyboard::Event) -> Command<Message> {
        match event {
            // Alt+Enter
            keyboard::Event::KeyPressed {
                key_code: KeyCode::Enter,
                modifiers: Modifiers::ALT,
            } => window::fetch_mode(Message::ToggleFullScreen),
            // Ctrl+-
            keyboard::Event::KeyPressed {
                key_code: KeyCode::Minus,
                modifiers: Modifiers::CTRL,
            } => {
                self.state.zoom_out();
                Command::none()
            }
            // Ctrl+=
            keyboard::Event::KeyPressed {
                key_code: KeyCode::Equals,
                modifiers: Modifiers::CTRL,
            } => {
                self.state.zoom_in();
                Command::none()
            }
            // Ctrl+0
            keyboard::Event::KeyPressed {
                key_code: KeyCode::Key0,
                modifiers: Modifiers::CTRL,
            } => {
                self.state.zoom_reset();
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn handle_window_event(&mut self, event: window::Event) -> Command<Message> {
        match event {
            window::Event::CloseRequested => {
                self.state.show_exit_dialog = true;
                Command::none()
            }
            _ => Command::none(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    AppEvent(Event),
    ToggleDarkMode(bool),
    ToggleFullScreen(Mode),
    ConfirmExit(bool),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from(env!("CARGO_PKG_NAME"))
    }

    fn theme(&self) -> Self::Theme {
        match self.state.dark_mode {
            Some(true) => iced::Theme::Dark,
            Some(false) => iced::Theme::Light,
            None => {
                unreachable!("App field: `dark_mode` should not be None")
            }
        }
    }

    fn scale_factor(&self) -> f64 {
        self.state.scale_factor
    }

    // 处理 Message, 更新状态，生成 Command 投递给 Executor 异步运行
    // Message 来源：
    // 1. view 用户交互
    // 2. subscription events
    // 3. Command 异步执行结果
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::AppEvent(event) => match event {
                Event::Keyboard(e) => self.handle_key_event(e),
                Event::Window(e) => self.handle_window_event(e),
                _ => Command::none(),
            },
            Message::ToggleDarkMode(enable) => {
                self.state.dark_mode = Some(enable);
                Command::none()
            }
            Message::ToggleFullScreen(cur_mode) => {
                if cur_mode == Mode::Fullscreen {
                    window::change_mode(Mode::Windowed)
                } else {
                    window::change_mode(Mode::Fullscreen)
                }
            }
            Message::ConfirmExit(confirm_exit) => {
                if confirm_exit {
                    window::close()
                } else {
                    self.state.show_exit_dialog = false;
                    Command::none()
                }
            }
        }
    }

    // 监控 app 所有事件，并包装成 Message 投递给 update(...)
    fn subscription(&self) -> iced::Subscription<Self::Message> {
        subscription::events().map(Message::AppEvent)
    }

    // 视图渲染，与用户交互产生 Message 投递给 update(...)
    fn view(&self) -> Element<Message> {
        let content: Element<'_, Message> = if self.state.show_exit_dialog {
            iced::widget::column![
                text("您确认退出 App 么？"),
                row![
                    button("确认").on_press(Message::ConfirmExit(true)),
                    button("取消").on_press(Message::ConfirmExit(false))
                ]
                .align_items(iced::Alignment::Center)
                .spacing(20),
            ]
            .align_items(iced::Alignment::Center)
            .spacing(20)
            .into()
        } else {
            row![
                text("模式："),
                radio("日间", false, self.state.dark_mode, Message::ToggleDarkMode),
                radio("夜间", true, self.state.dark_mode, Message::ToggleDarkMode),
            ]
            .align_items(iced::Alignment::Center)
            .spacing(20)
            .into()
        };

        container(content)
            .style(theme::Container::Transparent)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

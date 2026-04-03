// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::{cosmic_theme, theme};
use cosmic::widget::{self, about::About, menu, icon};
use cosmic::{iced_futures, prelude::*};
use futures_util::SinkExt;
use core::fmt;
use std::collections::HashMap;
use std::time::Duration;
use std::fmt::{Display, Formatter};


const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Active page tracker
    page_active: Page,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, Message>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Inspected node
    inspect: Option<Node>,
    /// Time active
    time: u32,
    /// Toggle the watch subscription
    watch_is_active: bool,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Select(Page),
    Inspect(Option<Node>),
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    ToggleWatch,
    UpdateConfig(Config),
    WatchTick(u32),
}

/// The page to display in the application.
#[derive(Debug, Clone, Default)]
pub enum Page {
    Page1,
    Page2,
    #[default]
    Page3,
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Page::Page1 => write!(f, "Device - Software"),
            Page::Page2 => write!(f, "Software - Device"),
            Page::Page3 => write!(f, "Device - Device"),
        }
    }
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.SammyMcFly.switchbox";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            page_active: Page::default(),
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            inspect: None,
            time: 0,
            watch_is_active: false,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    fn header_center(&self) -> Vec<Element<'_, Self::Message>> {
        vec![
            widget::tooltip(
                widget::button::text(Page::Page1.to_string())
                    .on_press(Message::Select(Page::Page1)),
                widget::text(fl!("settings")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
            widget::tooltip(
                widget::button::text(Page::Page2.to_string())
                    .on_press(Message::Select(Page::Page2)),
                widget::text(fl!("settings")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
            widget::tooltip(
                widget::button::text(Page::Page3.to_string())
                    .on_press(Message::Select(Page::Page3)),
                widget::text(fl!("settings")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        ]
    }

    fn header_end(&self) -> Vec<Element<'_, Message>> {
        vec![
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("application-menu-symbolic"))
                    .on_press(Message::ToggleContextPage(ContextPage::Settings)),
                widget::text(fl!("settings")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        ]
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::Settings => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::Settings),
            ),
        })
    }

    fn footer(&self) -> Option<Element<'_, Message>> {
        if self.inspect.is_none() {
            return None;
        }

        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            space_s,
            ..
        } = theme::active().cosmic().spacing;

        let container = widget::layer_container(widget::column::with_children(vec![
            widget::row::with_children([
                widget::column::with_children(vec![
                    widget::text::body("footer").into(),
                ])
                .align_x(Alignment::Start)
                .into(),
                widget::space::horizontal().into(),
                widget::column::with_children(vec![
                    widget::tooltip(
                        widget::button::icon(icon::from_name("window-close-symbolic"))
                            .on_press(Message::Inspect(None))
                            .padding(8),
                        widget::text::body(fl!("close")),
                        widget::tooltip::Position::Top,
                    )
                    .into()
                ])
                .align_x(Alignment::End)
                .into(),
            ])
            .padding([space_xxs, 0])
            .align_y(Alignment::Center)
            .into(),
            widget::space::vertical().height(space_xs).into(),
            widget::text::body("footer").into(),
            widget::space::vertical().height(space_s).into(),
            widget::row::with_children(vec![
                widget::button::link(fl!("footer-info"))
                    .on_press(Message::ToggleContextPage(ContextPage::Settings))
                    .padding(0)
                    .trailing_icon(true)
                    .into(),
                widget::space::horizontal().into(),
                widget::button::standard(fl!("footer-info"))
                    .on_press(Message::ToggleContextPage(ContextPage::Settings))
                    .into(),
            ])
            .align_y(Alignment::Center)
            .into(),
        ]))
        .padding([space_xxs, space_xs])
        .layer(cosmic_theme::Layer::Primary);

        Some(container.into())
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let content: Element<_> = match self.page_active {
            Page::Page1 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 1)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                let counter_label = ["Watch: ", self.time.to_string().as_str()].concat();
                let section = cosmic::widget::settings::section().add(
                    cosmic::widget::settings::item::builder(counter_label).control(
                        widget::button::text(if self.watch_is_active {
                            "Stop"
                        } else {
                            "Start"
                        })
                        .on_press(Message::ToggleWatch),
                    ),
                );

                widget::column::with_capacity(2)
                    .push(header)
                    .push(section)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Page2 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 2)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                widget::column::with_capacity(1)
                    .push(header)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Page3 => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("welcome")))
                    .push(widget::text::title3(fl!("page-id", num = 3)))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                let counter_label = ["Watch: ", self.time.to_string().as_str()].concat();
                let section = cosmic::widget::settings::section().add(
                    cosmic::widget::settings::item::builder(counter_label).control(
                        widget::button::text("Footer")
                        .on_press(Message::Inspect(Some(Node { id: 1 }))),
                    ),
                );


                widget::column::with_capacity(1)
                    .push(header)
                    .push(section)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }
        };

        widget::container(content)
            .width(600)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let mut subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Conditionally enables a timer that emits a message every second.
        if self.watch_is_active {
            subscriptions.push(Subscription::run(|| {
                iced_futures::stream::channel(1, |mut emitter: iced_futures::futures::channel::mpsc::Sender<Self::Message>| async move {
                    let mut time = 1;
                    let mut interval = tokio::time::interval(Duration::from_secs(1));

                    loop {
                        interval.tick().await;
                        _ = emitter.send(Message::WatchTick(time)).await;
                        time += 1;
                    }
                })
            }));
        }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Select(page) => {
                self.page_active = page;
            }

            Message::Inspect(page) => {
                self.inspect = page
            }

            Message::WatchTick(time) => {
                self.time = time;
            }

            Message::ToggleWatch => {
                self.watch_is_active = !self.watch_is_active;
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
        }
        Task::none()
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        window_title.push_str(" — ");
        window_title.push_str(&self.page_active.to_string());

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    id: u32,
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    Settings,
}


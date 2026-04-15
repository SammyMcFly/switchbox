// SPDX-License-Identifier: MPL-2.0

mod settings;
mod graph;
mod inspector;

use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::theme;
use cosmic::widget::{self, about::About, menu};
use cosmic::{iced_futures, prelude::*};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::time::Duration;
use std::process;


const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../../resources/icons/hicolor/scalable/apps/icon.svg");


/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// The settings page for this app.
    settings: Settings,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// UI state
    view_settings: settings::Visibility,
    /// Pipewire graph state
    graph: graph::Graph,
    /// Inspector: node info and internal node settings
    inspector: inspector::Inspector,
    /// Hovered object
    hovered_items: HoveredItems,
    /// Inspected node
    inspected_object: Option<graph::PwId>,
}

#[derive(Debug, Clone, Default)]
struct Settings{
    buffer_size: u16,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    ObjectHover(graph::PwId),
    StopObjectHover,
    NodeHover(graph::PwId),
    StopNodeHover,
    PortHover(graph::PwId),
    StopPortHover,
    Inspect(Option<graph::PwId>),
    InspectorTabSelected(widget::segmented_button::Entity),
    ToggleShowVideo,
    ToggleShowAudio,
    ToggleShowMidi,
    ToggleShowDevices,
    ToggleShowClients,
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    Quit,
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
            settings: Settings::default(),
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
            view_settings: settings::Visibility::default(),
            // graph: graph::Graph::default(),
            graph: graph::Graph::test(),
            inspector: inspector::Inspector::new(),
            hovered_items: HoveredItems::default(),
            inspected_object: None,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let file = menu::Tree::with_children(
            menu::root(fl!("file")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("quit"), None, MenuAction::Quit)],
            ),
        );

        let view = menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        );

        let menu_bar = menu::bar(vec![
            file,
            view,
        ]);

        vec![menu_bar.into()]
    }

    // fn header_center(&self) -> Vec<Element<'_, Self::Message>> {
    //     vec![
    //         widget::tooltip(
    //             widget::button::text(Page::Page1.to_string())
    //                 .on_press(Message::Select(Page::Page1)),
    //             widget::text(fl!("settings")),
    //             widget::tooltip::Position::Bottom,
    //         )
    //         .into(),
    //         widget::tooltip(
    //             widget::button::text(Page::Page2.to_string())
    //                 .on_press(Message::Select(Page::Page2)),
    //             widget::text(fl!("settings")),
    //             widget::tooltip::Position::Bottom,
    //         )
    //         .into(),
    //         widget::tooltip(
    //             widget::button::text(Page::Page3.to_string())
    //                 .on_press(Message::Select(Page::Page3)),
    //             widget::text(fl!("settings")),
    //             widget::tooltip::Position::Bottom,
    //         )
    //         .into(),
    //     ]
    // }

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
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
        })
    }

    fn footer(&self) -> Option<Element<'_, Message>> {
        self.inspector.view(self.inspected_object)
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut content = widget::column::with_capacity(2);

        // top bar
        let top_bar_content = widget::container(
            widget::row::with_children(vec![
                widget::container(widget::column().width(Length::Fill)).width(Length::Fill).into(),
                widget::checkbox(self.view_settings.show_video)
                    .label(fl!("video"))
                    .on_toggle(|_| {Message::ToggleShowVideo})
                    .into(),
                widget::checkbox(self.view_settings.show_audio)
                    .label(fl!("audio"))
                    .on_toggle(|_| {Message::ToggleShowAudio})
                    .into(),
                widget::checkbox(self.view_settings.show_midi)
                    .label(fl!("midi"))
                    .on_toggle(|_| {Message::ToggleShowMidi})
                    .into(),
                // widget::divider::horizontal::default(),
                widget::checkbox(self.view_settings.show_devices)
                    .label(fl!("devices"))
                    .on_toggle(|_| {Message::ToggleShowDevices})
                    .into(),
                widget::checkbox(self.view_settings.show_clients)
                    .label(fl!("clients"))
                    .on_toggle(|_| {Message::ToggleShowClients})
                    .into(),
            ])
            .align_y(Alignment::Center)
            .padding(spacing.space_xxs)
            .spacing(spacing.space_xxs),
        )
        .style(|theme| {
            let cosmic = theme.cosmic();
            widget::container::Style {
                background: Some(cosmic.primary.base.into()),
                border: cosmic::iced::Border {
                    color: cosmic::iced::Color::from_rgb8(0, 0, 0),
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..widget::container::Style::default()
            }
        })
        .padding(spacing.space_xxs);

        content = content.push(top_bar_content);

        let section_content = self.graph.view(self.hovered_items.hovered(), self.inspected_object);

        // main content
        let section = widget::mouse_area(
            widget::container(section_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                // .center_y(Length::Fill)
        )
        .on_press(Message::Inspect(None));

        let scrollable_section = widget::scrollable(section)
        .height(Length::Fill);

        content = content.push(scrollable_section);

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .apply(widget::container)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(spacing.space_none)
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

        // // Conditionally enables a timer that emits a message every second.
        // if self.watch_is_active {
        //     subscriptions.push(Subscription::run(|| {
        //         iced_futures::stream::channel(1, |mut emitter: iced_futures::futures::channel::mpsc::Sender<Self::Message>| async move {
        //             let mut time = 1;
        //             let mut interval = tokio::time::interval(Duration::from_secs(1));

        //             loop {
        //                 interval.tick().await;
        //                 _ = emitter.send(Message::WatchTick(time)).await;
        //                 time += 1;
        //             }
        //         })
        //     }));
        // }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ObjectHover(pwid) => {
                println!("hover obj: {pwid}");
                self.hovered_items.enter_object(pwid);
            }

            Message::StopObjectHover => {
                println!("exit obj");
                self.hovered_items.exit_object();
            }

            Message::NodeHover(pwid) => {
                println!("hover node: {pwid}");
                self.hovered_items.enter_node(pwid);
            }

            Message::StopNodeHover => {
                println!("exit node");
                self.hovered_items.exit_node();
            }

            Message::PortHover(pwid) => {
                println!("hover port: {pwid}");
                self.hovered_items.enter_port(pwid);
            }

            Message::StopPortHover => {
                println!("exit port");
                self.hovered_items.exit_port();
            }

            Message::Inspect(pwid) => {
                self.inspected_object = pwid;
            }

            Message::InspectorTabSelected(tab) => {
                self.inspector.activate(tab);
            }

            Message::ToggleShowVideo => {
                self.view_settings.show_video = !self.view_settings.show_video;
            }

            Message::ToggleShowAudio => {
                self.view_settings.show_audio = !self.view_settings.show_audio;
            }

            Message::ToggleShowMidi => {
                self.view_settings.show_midi = !self.view_settings.show_midi;
            }

            Message::ToggleShowDevices => {
                self.view_settings.show_devices = !self.view_settings.show_devices;
            }

            Message::ToggleShowClients => {
                self.view_settings.show_clients = !self.view_settings.show_clients;
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
            }

            Message::Quit => {
                process::exit(0);
            }
        }
        Task::none()
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let window_title = fl!("app-title");

        // window_title.push_str(" — ");
        // window_title.push_str(&self.page_active.to_string());

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    fn settings(&self) -> Element<'_, Message> {
        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("repository"))
                .add(
                    widget::settings::item::builder(fl!("footer-info"))
                        .toggler(self.view_settings.show_audio, |_| {Message::ToggleShowAudio}),
                )
                .into(),
        ])
        .into()
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    Quit,
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::Quit => Message::Quit,
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct HoveredItems {
    hovered_object: Option<graph::PwId>,
    hovered_node: Option<graph::PwId>,
    hovered_port: Option<graph::PwId>,
}

impl HoveredItems {
    fn enter_object(&mut self, value: graph::PwId) {
        self.hovered_object = Some(value);
    }

    fn exit_object(&mut self) {
        self.hovered_object = None;
    }

    fn enter_node(&mut self, value: graph::PwId) {
        self.hovered_node = Some(value);
    }

    fn exit_node(&mut self) {
        self.hovered_node = None;
    }

    fn enter_port(&mut self, value: graph::PwId) {
        self.hovered_port = Some(value);
    }

    fn exit_port(&mut self) {
        self.hovered_port = None;
    }

    fn hovered(&self) -> Option<graph::PwId> {
        if self.hovered_port.is_some() {
            self.hovered_port
        } else if self.hovered_node.is_some() {
            self.hovered_node
        } else {
            self.hovered_object
        }
    }
}

// pub fn checkbox_style(media_type: MediaType, theme: &cosmic::Theme, status: widget::checkbox::Status) -> widget::checkbox::Style {
//     let color = match media_type {
//         MediaType::Video => VIDEO_PORT_COLOR,

//         MediaType::Audio => AUDIO_PORT_COLOR,

//         MediaType::Midi => MIDI_PORT_COLOR,

//         MediaType::Parameter => PARAMETER_PORT_COLOR,
//     };

//     let theme = theme.cosmic();

//     widget::checkbox::Style {
//         background: cosmic::iced::Background::Color(theme.accent.base.into()),
//         icon_color: color,
//         border: cosmic::iced::Border {
//             color: cosmic::iced::Color::TRANSPARENT,
//             width: 0.0,
//             radius: 5.0.into(),
//         },
//         text_color: Some(theme.accent.on.into()),
//     }
// }


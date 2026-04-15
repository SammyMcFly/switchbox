//

use crate::app::graph;
use crate::app::Message;


use crate::fl;
use cosmic::iced::Alignment;
use cosmic::{cosmic_theme, theme};
use cosmic::widget::{self, icon};
use cosmic::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InspectorTab {
    InternalRouting,
    #[default]
    Info,
}

pub struct Inspector {
    tabs: widget::segmented_button::Model<widget::segmented_button::SingleSelect>,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            tabs: widget::segmented_button::Model::builder()
                .insert(|b| b.text(fl!("internal-routing")).data(InspectorTab::InternalRouting))
                .insert(|b| b.text(fl!("info")).data(InspectorTab::Info).activate())
                .build(),
        }
    }
    pub fn activate(&mut self, entity: widget::segmented_button::Entity) {
        self.tabs.activate(entity);
    }

    pub fn view(&self, inspected_object: Option<graph::PwId>) -> Option<Element<'_, super::Message>> {
        inspected_object?;

        let spacing = theme::active().cosmic().spacing;

        let active_tab = self.tabs.active_data::<InspectorTab>().unwrap();
        let tabs = widget::segmented_button::horizontal(&self.tabs)
            .padding(spacing.space_none)
            .button_alignment(cosmic::iced::Alignment::Center)
            .on_activate(|id| Message::InspectorTabSelected(id) );

        let active_tab: cosmic::iced::Element<'_, Message, Theme, Renderer> = match active_tab {
            InspectorTab::InternalRouting => widget::row::with_children(vec![
                widget::button::link(fl!("footer-info"))
                    .on_press(Message::ToggleContextPage(super::ContextPage::Settings))
                    .padding(0)
                    .trailing_icon(true)
                    .into(),
                widget::space::horizontal().into(),
                widget::button::standard(fl!("footer-info"))
                    .on_press(Message::ToggleContextPage(super::ContextPage::Settings))
                    .into(),
            ])
            .align_y(Alignment::Center).into(),

            InspectorTab::Info => widget::text::body("footer").into(),
        };

        let mut content = widget::column::with_children(vec![]);

        content = content.push(
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
            .padding([spacing.space_xxs, 0])
            .align_y(Alignment::Center)
        );

        content = content.push(widget::space::vertical().height(spacing.space_none));

        content = content.push(
            widget::row::with_children([
                widget::column()
                    .push(tabs)
                    .push(active_tab)
                    .spacing(spacing.space_xxs)
                    .into(),
            ])
        );

        let container = widget::layer_container(content)
            .padding([spacing.space_xxs, spacing.space_xs])
            .layer(cosmic_theme::Layer::Primary);

        Some(container.into())
    }
}

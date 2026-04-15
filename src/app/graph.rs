//
//
//

use super::Message;
use std::collections::HashMap;
use cosmic::prelude::*;
use cosmic::{widget, theme};
use cosmic::iced::Alignment;


pub type PwId = u32;
// type UniqueId = String;

// type ObjectOrderKey = u32;
// type NodeOrderKey = u32;
// type PortOrderKey = u32;

const VIDEO_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(66, 133, 244);
const AUDIO_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(232, 157, 0);
const MIDI_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(0, 171, 122);
// const PARAMETER_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(181, 9, 40);



#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    Video,
    Audio,
    Midi,
    // Parameter,
    All,
}

#[derive(Debug, Clone)]
pub enum DeviceClass {
    Device,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortType {
    Source,
    Sink,
}



#[derive(Debug, Clone, Default)]
pub struct Graph {
    objects: Vec<PwId>,
    entities: HashMap<PwId, PipewireEntity>,
    connections: HashMap<(PwId, PwId), Connection>,
}

impl Graph {
    pub fn test() -> Self {
        let mut entities = HashMap::new();
        entities.insert(3, PipewireEntity::Port { id: 3, name: "test_port_1".to_string(), port_type: PortType::Source });
        entities.insert(4, PipewireEntity::Port { id: 4, name: "test_port_2".to_string(), port_type: PortType::Source });
        entities.insert(5, PipewireEntity::Port { id: 5, name: "test_port_1".to_string(), port_type: PortType::Source });
        entities.insert(6, PipewireEntity::Port { id: 6, name: "test_port_2".to_string(), port_type: PortType::Source });
        entities.insert(2, PipewireEntity::Node { id: 2, name: "test_node_1".to_string(), media_type: MediaType::Audio, ports: vec![3, 4, 5, 6] });
        entities.insert(1, PipewireEntity::MediaObject { id: 1, name: "test_device_1".to_string(), device_type: DeviceClass::Device, nodes: vec![2] });
        entities.insert(9, PipewireEntity::Port { id: 9, name: "test_port_1".to_string(), port_type: PortType::Sink });
        entities.insert(10, PipewireEntity::Port { id: 10, name: "test_port_2".to_string(), port_type: PortType::Sink });
        entities.insert(8, PipewireEntity::Node { id: 8, name: "test_node_1".to_string(), media_type: MediaType::Midi, ports: vec![9, 10] });
        entities.insert(11, PipewireEntity::Port { id: 11, name: "test_port_1".to_string(), port_type: PortType::Sink });
        entities.insert(12, PipewireEntity::Port { id: 12, name: "test_port_2".to_string(), port_type: PortType::Sink });
        entities.insert(13, PipewireEntity::Node { id: 13, name: "test_node_1".to_string(), media_type: MediaType::Audio, ports: vec![11, 12] });
        entities.insert(7, PipewireEntity::MediaObject { id: 7, name: "test_device_2".to_string(), device_type: DeviceClass::Device, nodes: vec![8, 13] });
        entities.insert(14, PipewireEntity::Port { id: 14, name: "test_port_1".to_string(), port_type: PortType::Source });
        entities.insert(15, PipewireEntity::Port { id: 15, name: "test_port_2".to_string(), port_type: PortType::Source });
        entities.insert(16, PipewireEntity::Node { id: 16, name: "test_node_1".to_string(), media_type: MediaType::Video, ports: vec![14, 15] });
        entities.insert(17, PipewireEntity::MediaObject { id: 17, name: "test_device_3".to_string(), device_type: DeviceClass::Device, nodes: vec![16] });


        let connections = HashMap::new();

        Self {
            objects: vec![1, 7, 17],
            entities,
            connections,
        }
    }

    pub fn view(&self, hovered_object: Option<PwId>, inspected_object: Option<PwId>) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut sources: widget::Column<'_, Message> = widget::column();
        for pwid in &self.objects {
            if let Some(device) = self.entities.get(pwid).expect("Failed to get value for key").view(
                PortType::Source,
                MediaType::All,
                hovered_object,
                inspected_object,
                &self.entities,
            ) {
                sources = sources.push(device);
            }
        }

        let mut sinks: widget::Column<'_, Message> = widget::column();
        for pwid in &self.objects {
            if let Some(device) = self.entities.get(pwid).expect("Failed to get value for key").view(
                PortType::Sink,
                MediaType::All,
                hovered_object,
                inspected_object,
                &self.entities,
            ) {
                sinks = sinks.push(device);
            }
        }

        widget::row::with_children([
            sources
            .spacing(spacing.space_xxs)
            .align_x(Alignment::Start)
            .into(),
            widget::space::horizontal().into(),
            sinks
            .spacing(spacing.space_xxs)
            .align_x(Alignment::End)
            .into(),
        ])
        .padding(spacing.space_xxxs)
        .align_y(Alignment::Start)
        .into()
    }
}


// struct KnownObject {
//     unique_id: UniqueId,
//     is_present: bool,
//     known_nodes: HashMap<UniqueId, KnownObject>,
//     known_connections: HashMap<(UniqueId, UniqueId), KnownObject>,
// }

// pub struct CompleteKnownGraph {
//     pwid_dict: HashMap<UniqueId, PwId>,
//     known_objects: HashMap<UniqueId, KnownObject>,
// }


#[derive(Debug, Clone)]
pub enum PipewireEntity {
    MediaObject {
        // unique_id: UniqueId,
        id: PwId,
        name: String,
        device_type: DeviceClass,
        nodes: Vec<PwId>,
        // node_order: HashMap<NodeOrderKey, PwId>,
    },
    Node {
        // unique_id: UniqueId,
        id: PwId,
        name: String,
        media_type: MediaType,
        ports: Vec<PwId>,
        // port_order: HashMap<PortOrderKey, PwId>,
    },
    Port {
        // unique_id: UniqueId,
        id: PwId,
        name: String,
        port_type: PortType,
    }
}

impl PipewireEntity {
    fn view<'a>(&self, side: PortType, media_type: MediaType, hovered_object: Option<PwId>, inspected_object: Option<PwId>, objects: &'a HashMap<PwId, Self>) -> Option<Element<'a, Message>> {
        match self {
            Self::MediaObject { id, name, device_type, nodes } => {
                let spacing = theme::active().cosmic().spacing;

                let is_hovered = if let Some(hover_pwid) = hovered_object {
                    hover_pwid == *id
                } else {
                    false
                };

                let is_inspected = if let Some(inspect_pwid) = inspected_object {
                    inspect_pwid == *id
                } else {
                    false
                };

                let mut device: widget::Column<'_, Message> = widget::column();
                let label: Element<'_, Message> = widget::row::with_children([widget::text(name.clone()).into()]).padding(spacing.space_xxs).into();
                device = device.push(label);

                let mut nodes_content: widget::Column<'_, Message> = widget::column();
                let mut has_nodes_on_side = false;
                for pwid in nodes {
                    if let Some(node) = objects.get(pwid).expect("Failed to get value for key").view(side, media_type, hovered_object, inspected_object, objects) {
                        nodes_content = nodes_content.push(node);
                        has_nodes_on_side = true;
                    }
                }

                if !has_nodes_on_side {
                    return None
                }

                device = device.push(widget::row::with_children([nodes_content.spacing(spacing.space_xxs).into()]));

                let padding = match side {
                    PortType::Source => [0, 0, spacing.space_xxs, spacing.space_xxs],
                    PortType::Sink => [0, spacing.space_xxs, spacing.space_xxs, 0],
                };

                let alignment = match side {
                    PortType::Source => Alignment::Start,
                    PortType::Sink => Alignment::End,
                };

                Some(widget::mouse_area(
                    widget::container(
                        device
                            .align_x(alignment)
                            .spacing(spacing.space_none)
                            .padding(padding)
                    )
                    .width(200)
                    .style(move |theme: &cosmic::Theme| {
                        let cosmic_data = theme.cosmic();
                        if is_inspected {
                            let border = cosmic::iced::Border {
                                color: cosmic_data.accent.base.into(),
                                width: 2.0,
                                radius: 8.0.into(),
                            };
                            if is_hovered {
                                widget::container::Style {
                                    background: Some(cosmic_data.primary.divider.into()),
                                    border,
                                    ..widget::container::Style::default()
                                }
                            } else {
                                widget::container::Style {
                                    background: Some(cosmic_data.primary.base.into()),
                                    border,
                                    ..widget::container::Style::default()
                                }
                            }
                        } else if is_hovered {
                            widget::container::Style {
                                background: Some(cosmic_data.primary.divider.into()),
                                border: cosmic::iced::Border {
                                    color: cosmic_data.background.small_widget.into(),
                                    width: 1.0,
                                    radius: 8.0.into(),
                                },
                                ..widget::container::Style::default()
                            }
                        } else {
                            widget::container::Style {
                                background: Some(cosmic_data.primary.base.into()),
                                border: cosmic::iced::Border {
                                    color: cosmic_data.background.small_widget.into(),
                                    width: 1.0,
                                    radius: 8.0.into(),
                                },
                                // shadow: cosmic::iced::Shadow {
                                //     color: cosmic::iced::Color::from_rgb8(0, 0, 0),
                                //     blur_radius: 4.0,
                                //     ..Default::default()
                                // },
                                ..widget::container::Style::default()
                            }
                        }
                    })
                )
                .on_enter(Message::ObjectHover(*id))
                .on_exit(Message::StopObjectHover)
                .on_press(Message::Inspect(Some(*id)))
                .into())
            }

            Self::Node { id, name, media_type, ports } => {
                let spacing = theme::active().cosmic().spacing;

                let is_hovered = if let Some(hover_pwid) = hovered_object {
                    hover_pwid == *id
                } else {
                    false
                };

                let is_inspected = if let Some(inspect_pwid) = inspected_object {
                    inspect_pwid == *id
                } else {
                    false
                };

                let mut node: widget::Column<'_, Message> = widget::column();
                let label: Element<'_, Message> = widget::row::with_children([widget::text(name.clone()).into()]).into();
                node = node.push(label);
                let mut has_ports_on_side = false;
                for key in ports {
                    if let Some(port) = objects.get(key).expect("Failed to get value for key").view(side, *media_type, hovered_object, inspected_object, objects) {
                        node = node.push(port);
                        has_ports_on_side = true;
                    }
                }

                if !has_ports_on_side {
                    return None
                }

                let padding = match side {
                    PortType::Source => [spacing.space_xxs, 0, spacing.space_xxs, spacing.space_xxs],
                    PortType::Sink => [spacing.space_xxs, spacing.space_xxs, spacing.space_xxs, 0],
                };

                Some(widget::mouse_area(
                    widget::container(
                        node
                            .align_x(Alignment::Center)
                            .spacing(spacing.space_none)
                            .padding(padding)
                    )
                    .width(200)
                    .style(move |theme: &cosmic::Theme| {
                        let cosmic_data = theme.cosmic();
                        if is_inspected {
                            let border = cosmic::iced::Border {
                                color: cosmic_data.accent.base.into(),
                                width: 2.0,
                                radius: 8.0.into(),
                            };
                            if is_hovered {
                                widget::container::Style {
                                    background: Some(cosmic_data.secondary.divider.into()),
                                    border,
                                    ..widget::container::Style::default()
                                }
                            } else {
                                widget::container::Style {
                                    background: Some(cosmic_data.secondary.base.into()),
                                    border,
                                    // shadow: cosmic::iced::Shadow {
                                    //     color: cosmic::iced::Color::from_rgb8(0, 0, 0),
                                    //     blur_radius: 4.0,
                                    //     ..Default::default()
                                    // },
                                    ..widget::container::Style::default()
                                }
                            }
                        } else if is_hovered {
                            widget::container::Style {
                                background: Some(cosmic_data.secondary.divider.into()),
                                border: cosmic::iced::Border {
                                    color: cosmic_data.primary.small_widget.into(),
                                    width: 1.0,
                                    radius: 8.0.into(),
                                },
                                ..widget::container::Style::default()
                            }
                        } else {
                            widget::container::Style {
                                background: Some(cosmic_data.secondary.base.into()),
                                border: cosmic::iced::Border {
                                    color: cosmic_data.primary.small_widget.into(),
                                    width: 1.0,
                                    radius: 8.0.into(),
                                },
                                ..widget::container::Style::default()
                            }
                        }
                    })
                )
                .on_enter(Message::NodeHover(*id))
                .on_exit(Message::StopNodeHover)
                .on_press(Message::Inspect(Some(*id)))
                .into())
            }

            Self::Port { id, name, port_type } => {
                let is_hovered = if let Some(hover_pwid) = hovered_object {
                    hover_pwid == *id
                } else {
                    false
                };

                let is_inspected = if let Some(inspect_pwid) = inspected_object {
                    inspect_pwid == *id
                } else {
                    false
                };

                fn port_widget<'a, Message: 'a>(media_type: MediaType, is_hovered: bool, is_inspected: bool) -> Element<'a, Message> {
                    let size = if is_inspected {
                        16
                    } else if is_hovered {
                        14
                    } else {
                        12
                    };
                    widget::container(widget::space::horizontal())
                        .width(size)
                        .height(size)
                        .style(move |theme| port_style(media_type, theme, is_hovered, is_inspected))
                        .into()
                }

                fn port_style(media_type: MediaType, theme: &cosmic::Theme, is_hovered: bool, is_inspected: bool) -> widget::container::Style {
                    let color = match media_type {
                        MediaType::Video => VIDEO_PORT_COLOR,

                        MediaType::Audio => AUDIO_PORT_COLOR,

                        MediaType::Midi => MIDI_PORT_COLOR,

                        // MediaType::Parameter => PARAMETER_PORT_COLOR,
                        MediaType::All => panic!("No media type found"),
                    };
                    let cosmic_data = theme.cosmic();
                    let border_radius = if is_hovered && is_inspected {6} else if is_inspected {8} else {6};
                    let border_width = if is_inspected {2.} else {0.};
                    let border = cosmic::iced::Border {
                        color: cosmic_data.accent.base.into(),
                        width: border_width,
                        radius: border_radius.into(),
                    };

                    widget::container::Style {
                        background: Some(cosmic::iced::Background::Color(color)),
                        border,
                        ..Default::default()
                    }
                }
                if side != *port_type {
                    return None
                }

                let spacing = theme::active().cosmic().spacing;
                let padd = if is_inspected {
                        0
                    } else if is_hovered {
                        1
                    } else {
                        2
                    };

                match *port_type {
                    PortType::Source => {
                        Some(widget::row::with_children([
                            widget::space::horizontal().into(),
                            widget::text(name.clone()).size(12).into(),
                            widget::space::horizontal().width(spacing.space_xxs).into(),
                            widget::container(
                                widget::mouse_area(port_widget(media_type, is_hovered, is_inspected))
                                    .on_enter(Message::PortHover(*id))
                                    .on_exit(Message::StopPortHover)
                                    .on_press(Message::Inspect(Some(*id)))
                            )
                            .padding([0, padd, 0, padd])
                            .into(),
                        ])
                        .align_y(Alignment::Center)
                        .padding(spacing.space_xxs)
                        .into())
                    }

                    PortType::Sink => {
                        Some(widget::row::with_children([
                            widget::container(
                                widget::mouse_area(port_widget(media_type, is_hovered, is_inspected))
                                    .on_enter(Message::PortHover(*id))
                                    .on_exit(Message::StopPortHover)
                                    .on_press(Message::Inspect(Some(*id)))
                            )
                            .padding([0, padd, 0, padd])
                            .into(),
                            widget::space::horizontal().width(spacing.space_xxs).into(),
                            widget::text(name.clone()).size(12).into(),
                            widget::space::horizontal().into(),
                        ])
                        .align_y(Alignment::Center)
                        .padding(spacing.space_xxs)
                        .into())
                    }
                }
            }
        }

    }
}


#[derive(Debug, Clone)]
struct Connection {
    from_port_id: PwId,
    to_port_id: PwId,
}




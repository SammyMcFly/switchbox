// Objects needed to model the pipewire flow graph
//
// These struct represent the current state of the pipewire graph.
// Their "view" method is used to render the graph to the UI.

use super::Message;
use std::collections::{HashMap, HashSet};
use cosmic::prelude::*;
use cosmic::{widget, theme};
use cosmic::iced::{Alignment, Length};
use cosmic::iced::widget::stack;
use cosmic::iced::widget::canvas;


pub type PipewireId = u32;
// type UniqueId = String;

// type ObjectOrderKey = u32;
// type NodeOrderKey = u32;
// type PortOrderKey = u32;

const VIDEO_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(66, 133, 244);
const AUDIO_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(232, 157, 0);
const MIDI_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(0, 171, 122);
// const PARAMETER_PORT_COLOR: cosmic::iced::Color = cosmic::iced::Color::from_rgb8(181, 9, 40);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    objects: Vec<PipewireId>,
    connections: HashSet<PipewireId>,
    entities: HashMap<PipewireId, PipewireEntity>,
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
        entities.insert(18, PipewireEntity::Port { id: 18, name: "test_port_3".to_string(), port_type: PortType::Source });
        entities.insert(19, PipewireEntity::Port { id: 19, name: "test_port_4".to_string(), port_type: PortType::Source });
        entities.insert(8, PipewireEntity::Node { id: 8, name: "test_node_1".to_string(), media_type: MediaType::Midi, ports: vec![9, 10, 18, 19] });
        entities.insert(11, PipewireEntity::Port { id: 11, name: "test_port_1".to_string(), port_type: PortType::Sink });
        entities.insert(12, PipewireEntity::Port { id: 12, name: "test_port_2".to_string(), port_type: PortType::Sink });
        entities.insert(20, PipewireEntity::Port { id: 20, name: "test_port_3".to_string(), port_type: PortType::Source });
        entities.insert(21, PipewireEntity::Port { id: 21, name: "test_port_4".to_string(), port_type: PortType::Source });
        entities.insert(13, PipewireEntity::Node { id: 13, name: "test_node_1".to_string(), media_type: MediaType::Audio, ports: vec![11, 12, 20, 21] });
        entities.insert(7, PipewireEntity::MediaObject { id: 7, name: "test_device_2".to_string(), device_type: DeviceClass::Device, nodes: vec![8, 13] });
        entities.insert(14, PipewireEntity::Port { id: 14, name: "test_port_1".to_string(), port_type: PortType::Source });
        entities.insert(15, PipewireEntity::Port { id: 15, name: "test_port_2".to_string(), port_type: PortType::Source });
        entities.insert(16, PipewireEntity::Node { id: 16, name: "test_node_1".to_string(), media_type: MediaType::Video, ports: vec![14, 15] });
        entities.insert(17, PipewireEntity::MediaObject { id: 17, name: "test_device_3".to_string(), device_type: DeviceClass::Device, nodes: vec![16] });


        let mut connections = HashSet::new();
        connections.insert(100);
        connections.insert(101);
        entities.insert(100, PipewireEntity::Connection { id: 100, from_port_id: 5, to_port_id: 11, media_type: MediaType::Audio, });
        entities.insert(101, PipewireEntity::Connection { id: 101, from_port_id: 6, to_port_id: 12, media_type: MediaType::Audio, });

        Self {
            objects: vec![1, 7, 17],
            entities,
            connections,
        }
    }

    fn canvas_padding() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxxs as f32
    }

    fn column_vertical_padding() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_none as f32
    }

    const fn column_horizontal_padding() -> f32 {
        10.
    }

    fn column_spacing() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxs as f32
    }

    const fn node_area_width() -> f32 {
        200. + 2.*Self::column_horizontal_padding()
    }

    pub fn view(
        &self,
        hovered_object: Option<PipewireId>,
        inspected_object: Option<PipewireId>,
        width: f32,
        source_scroll_offset_y: f32,
        sink_scroll_offset_y: f32,
    ) -> Element<'_, Message> {
        // store positions of all ports on the UI
        let mut port_positions: HashMap<PipewireId, (f32, f32)> = HashMap::new();

        // Widget layer
        // Sources
        // references to track the position on the canvas
        let x_total_padding_sources = Self::canvas_padding()+Self::node_area_width()-Self::column_horizontal_padding()-PipewireEntity::node_padding()-PipewireEntity::inspected_port_size()/2.;
        let y_total_padding_sources = Self::canvas_padding()+Self::column_vertical_padding();
        let mut reference = (x_total_padding_sources, y_total_padding_sources);

        let mut sources: widget::Column<'_, Message> = widget::column();
        for pwid in &self.objects {
            if let Some(object) = self.entities.get(pwid).expect("Failed to get value for key").view(
                PortType::Source,
                MediaType::All,
                hovered_object,
                inspected_object,
                &self.entities,
                &mut reference,
                &mut port_positions,
            ) {
                sources = sources.push(object);
                reference.1 += Self::column_spacing();
            }
        }

        // Sinks
        // references to track the position on the canvas
        let x_total_padding_sinks = width - x_total_padding_sources;
        let y_total_padding_sinks = Self::canvas_padding()+Self::column_vertical_padding();
        let mut reference = (x_total_padding_sinks, y_total_padding_sinks);

        let mut sinks: widget::Column<'_, Message> = widget::column();
        for pwid in &self.objects {
            if let Some(device) = self.entities.get(pwid).expect("Failed to get value for key").view(
                PortType::Sink,
                MediaType::All,
                hovered_object,
                inspected_object,
                &self.entities,
                &mut reference,
                &mut port_positions,
            ) {
                sinks = sinks.push(device);
                reference.1 += Self::column_spacing();
            }
        }

        let widget_layer = widget::row::with_children([
            widget::scrollable(
                sources
                    .spacing(Self::column_spacing())
                    .padding([Self::column_vertical_padding(), 10.])
                    .width(Self::node_area_width())
                    .align_x(Alignment::Start)
            )
            .on_scroll(|viewport| {
                let source_scroll_offset = viewport.absolute_offset();
                Message::SourcesScrolled(source_scroll_offset.y) // Schickt die vertikale Pixel-Position an update()
            })
            .height(Length::Fill).into(),
            widget::space::horizontal().into(),
            widget::scrollable(
                sinks
                    .spacing(Self::column_spacing())
                    .padding([Self::column_vertical_padding(), 10.])
                    .width(Self::node_area_width())
                    .align_x(Alignment::End)
            )
            .on_scroll(|viewport| {
                let sink_scroll_offset = viewport.absolute_offset();
                Message::SinksScrolled(sink_scroll_offset.y) // Schickt die vertikale Pixel-Position an update()
            })
            .height(Length::Fill).into(),
        ])
        .padding(Self::canvas_padding())
        .align_y(Alignment::Start);

        // Connections
        let ccset = CanvasConnectionSet::from(
            &self.connections,
            &self.entities,
            &port_positions,
            source_scroll_offset_y,
            sink_scroll_offset_y,
        );
        let connection_layer = cosmic::iced::widget::canvas::Canvas::<CanvasConnectionSet, Message, cosmic::Theme, cosmic::Renderer>::new(
            ccset
        )
            .width(Length::Fill)
            .height(Length::Fill);

        stack![widget_layer, connection_layer].into()
    }
}


// struct KnownObject {
//     unique_id: UniqueId,
//     is_present: bool,
//     known_nodes: HashMap<UniqueId, KnownObject>,
//     known_connections: HashMap<(UniqueId, UniqueId), KnownObject>,
// }

// pub struct CompleteKnownGraph {
//     pwid_dict: HashMap<UniqueId, PipewireId>,
//     known_objects: HashMap<UniqueId, KnownObject>,
// }


#[derive(Debug, Clone)]
pub enum PipewireEntity {
    MediaObject {
        // unique_id: UniqueId,
        id: PipewireId,
        name: String,
        device_type: DeviceClass,
        nodes: Vec<PipewireId>,
        // node_order: HashMap<NodeOrderKey, PipewireId>,
    },
    Node {
        // unique_id: UniqueId,
        id: PipewireId,
        name: String,
        media_type: MediaType,
        ports: Vec<PipewireId>,
        // port_order: HashMap<PortOrderKey, PipewireId>,
    },
    Port {
        // unique_id: UniqueId,
        id: PipewireId,
        name: String,
        port_type: PortType,
    },
    Connection {
        id: PipewireId,
        from_port_id: PipewireId,
        to_port_id: PipewireId,
        media_type: MediaType,
    }
}

impl PipewireEntity {
    const fn inspection_border_width() -> f32 {
        2.
    }

    fn object_spacing() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxs as f32
    }

    fn object_padding() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxs as f32
    }

    fn object_label_height() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xl as f32
    }

    fn object_label_padding() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxs as f32
    }

    fn node_spacing() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_none as f32
    }

    fn node_padding() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxs as f32
    }

    fn node_label_height() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_l as f32
    }

    fn port_spacing() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_none as f32
    }

    fn port_padding() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxs as f32
    }

    fn port_height() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_l as f32
    }

    fn port_spacing_to_label() -> f32 {
        let spacing = theme::active().cosmic().spacing;
        spacing.space_xxs as f32
    }

    const fn inspected_port_size() -> f32 {
        16.
    }

    const fn port_font_size() -> f32 {
        12.
    }

    fn view<'a>(
        &self,
        side: PortType,
        media_type: MediaType,
        hovered_object: Option<PipewireId>,
        inspected_object: Option<PipewireId>,
        objects: &'a HashMap<PipewireId, Self>,
        reference: &mut (f32, f32),
        port_positions: &mut HashMap<PipewireId, (f32, f32)>,
    ) -> Option<Element<'a, Message>> {
        match self {
            Self::MediaObject { id, name, device_type, nodes } => {
                // add height to reference position
                let height_of_header = Self::object_padding()*0.+Self::object_label_height();
                let height_of_footer = Self::object_padding();
                reference.1 += height_of_header;

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
                let label: Element<'_, Message> = widget::row::with_children([widget::text(name.clone()).into()])
                    .height(Self::object_label_height())
                    .align_y(Alignment::Center)
                    .padding(Self::object_label_padding())
                    .into();
                device = device.push(label);

                let mut nodes_content: widget::Column<'_, Message> = widget::column();
                let mut has_nodes_on_side = false;
                for pwid in nodes {
                    if let Some(node) = objects
                        .get(pwid).expect("Failed to get value for key")
                        .view(side, media_type, hovered_object, inspected_object, objects, reference, port_positions) {
                        nodes_content = nodes_content.push(node);
                        reference.1 += Self::object_spacing();
                        has_nodes_on_side = true;
                    }
                }

                if !has_nodes_on_side {
                    // remove height to reference position
                    reference.1 -= height_of_header;
                    return None
                }

                // add height to reference position
                reference.1 -= Self::object_spacing();
                reference.1 += height_of_footer;

                // create UI element
                let padding = match side {
                    PortType::Source => [0., 0., Self::object_padding(), Self::object_padding()],
                    PortType::Sink => [0., Self::object_padding(), Self::object_padding(), 0.],
                };

                device = device.push(widget::row::with_children([
                    nodes_content
                        .spacing(Self::object_spacing())
                        .padding(padding)
                        .into()
                ]));

                let alignment = match side {
                    PortType::Source => Alignment::Start,
                    PortType::Sink => Alignment::End,
                };

                Some(widget::mouse_area(
                    widget::container(
                        device
                            .align_x(alignment)
                            // .padding(padding)
                    )
                    .width(Length::Fill)
                    .style(move |theme: &cosmic::Theme| {
                        let cosmic_data = theme.cosmic();
                        if is_inspected {
                            let border = cosmic::iced::Border {
                                color: cosmic_data.accent.base.into(),
                                width: Self::inspection_border_width(),
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
                // add height to reference position
                let height_of_header = Self::node_padding()+Self::node_label_height()+Self::node_spacing();
                let height_of_footer = Self::node_padding();
                reference.1 += height_of_header;

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
                let label: Element<'_, Message> = widget::row::with_children([widget::text(name.clone()).into()])
                    .height(Self::node_label_height())
                    .align_y(Alignment::Center)
                    .into();
                node = node.push(label);
                let mut has_ports_on_side = false;
                for key in ports {
                    if let Some(port) = objects
                        .get(key).expect("Failed to get value for key")
                        .view(side, *media_type, hovered_object, inspected_object, objects, reference, port_positions) {
                        node = node.push(port);
                        reference.1 += Self::port_spacing();
                        has_ports_on_side = true;
                    }
                }

                if !has_ports_on_side {
                    // remove height to reference position
                    reference.1 -= height_of_header;
                    return None
                }

                // add height to reference position
                reference.1 -= Self::port_spacing();
                reference.1 += height_of_footer;

                // create UI element
                let padding = match side {
                    PortType::Source => [Self::node_padding(), 0., Self::node_padding(), Self::node_padding()],
                    PortType::Sink => [Self::node_padding(), Self::node_padding(), Self::node_padding(), 0.],
                };

                Some(widget::mouse_area(
                    widget::container(
                        node
                            .align_x(Alignment::Center)
                            .spacing(Self::node_spacing())
                            .padding(padding)
                    )
                    .width(Length::Fill)
                    .style(move |theme: &cosmic::Theme| {
                        let cosmic_data = theme.cosmic();
                        if is_inspected {
                            let border = cosmic::iced::Border {
                                color: cosmic_data.accent.base.into(),
                                width: Self::inspection_border_width(),
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
                        PipewireEntity::inspected_port_size()
                    } else if is_hovered {
                        PipewireEntity::inspected_port_size()-PipewireEntity::inspection_border_width()
                    } else {
                        PipewireEntity::inspected_port_size()-2.*PipewireEntity::inspection_border_width()
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
                    let border_width = if is_inspected {PipewireEntity::inspection_border_width()} else {0.};
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

                // add port position to hashset
                let height_of_element = Self::port_height();
                port_positions.insert(*id, (reference.0, reference.1+height_of_element/2.));
                reference.1 += height_of_element;

                // create UI element
                let padd = if is_inspected {0.} else if is_hovered {Self::inspection_border_width()/2.} else {Self::inspection_border_width()};

                match *port_type {
                    PortType::Source => {
                        Some(widget::row::with_children([
                            widget::space::horizontal().into(),
                            widget::text(name.clone()).size(Self::port_font_size()).into(),
                            widget::space::horizontal().width(Self::port_spacing_to_label()).into(),
                            widget::container(
                                widget::mouse_area(port_widget(media_type, is_hovered, is_inspected))
                                    .on_enter(Message::PortHover(*id))
                                    .on_exit(Message::StopPortHover)
                                    .on_press(Message::Inspect(Some(*id)))
                            )
                            .padding([0., padd, 0., padd])
                            .into(),
                        ])
                        .height(Self::port_height())
                        .align_y(Alignment::Center)
                        .spacing(Self::port_spacing())
                        .padding(Self::port_padding())
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
                            .padding([0., padd, 0., padd])
                            .into(),
                            widget::space::horizontal().width(Self::port_spacing_to_label()).into(),
                            widget::text(name.clone()).size(Self::port_font_size()).into(),
                            widget::space::horizontal().into(),
                        ])
                        .height(Self::port_height())
                        .align_y(Alignment::Center)
                        .spacing(Self::port_spacing())
                        .padding(Self::port_padding())
                        .into())
                    }
                }
            }
            PipewireEntity::Connection { .. } => {
                panic!(".view() is not supposed to be called on PipewireEntity::Connection directly.")
            },
        }
    }
}



#[derive(Debug, Clone, Default)]
pub struct CanvasConnectionSet {
    pub connections: Vec<(cosmic::iced::Point, cosmic::iced::Point, MediaType,)>,
}

impl CanvasConnectionSet {
    fn from(
        set: &HashSet<PipewireId>,
        entities: &HashMap<PipewireId, PipewireEntity>,
        port_positions: &HashMap<PipewireId, (f32, f32)>,
        source_scroll_offset_y: f32,
        sink_scroll_offset_y: f32,
    ) -> Self {
        let mut connections = Vec::new();
        for link_id in set {
            let link = entities.get(link_id).expect("Failed to get pipewire connection");
            if let PipewireEntity::Connection { from_port_id, to_port_id, media_type, .. } = link {
                let from = port_positions.get(from_port_id).expect("Failed to get position of port with id 'from_port_id'");
                let to = port_positions.get(to_port_id).expect("Failed to get position of port with id 'to_port_id'");
                connections.push((
                    cosmic::iced::Point::new(from.0, from.1-source_scroll_offset_y),
                    cosmic::iced::Point::new(to.0, to.1-sink_scroll_offset_y),
                    *media_type,
                ));
            }
        }
        Self { connections }
    }
}

impl<Message> canvas::Program<Message, cosmic::Theme, cosmic::Renderer> for CanvasConnectionSet {
 type State = ();

    fn draw(
    &self,
    _state: &Self::State,
    renderer: &cosmic::Renderer,
    _theme: &cosmic::Theme,
    bounds: cosmic::iced::Rectangle,
    _cursor: cosmic::iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        for (start, end, media_type) in &self.connections {
            let control_offset = (end.x - start.x).abs() / 2.0;

            let cp1 = cosmic::iced::Point::new(start.x + control_offset, start.y);
            let cp2 = cosmic::iced::Point::new(end.x - control_offset, end.y);

            let curve = canvas::Path::new(|builder| {
                builder.move_to(*start);
                builder.bezier_curve_to(cp1, cp2, *end);
            });

            let color = match media_type {
                MediaType::Video => VIDEO_PORT_COLOR,

                MediaType::Audio => AUDIO_PORT_COLOR,

                MediaType::Midi => MIDI_PORT_COLOR,

                // MediaType::Parameter => PARAMETER_PORT_COLOR,

                MediaType::All => panic!("No media type found"),
            };
            frame.stroke(
                &curve,
                canvas::Stroke::default()
                .with_width(2.5)
                .with_color(color) // Dezentere Farbe
                .with_line_cap(canvas::LineCap::Round),
            );
        }
        vec![frame.into_geometry()]
    }
}

//! Build and show dropdown `ListMenus`.
use crate::selection_list::StyleSheet;

use iced_native::{
    alignment::{Horizontal, Vertical},
    event,
    layout::{Limits, Node},
    mouse, renderer, touch, Clipboard, Color, Event, Layout, Length, Point, Rectangle, Shell, Size,
};
use std::borrow::Cow;
use std::marker::PhantomData;

use iced_native::widget::tree::{self, Tree};
use iced_native::{Element, Widget};

/// The Private [`List`] Handles the Actual list rendering.
#[allow(missing_debug_implementations)]
pub struct List<'a, T: 'a, Message, Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: iced_native::Renderer + iced_native::text::Renderer<Font = iced_native::Font>,
    Renderer::Theme: StyleSheet,
{
    /// Options pointer to hold all rendered strings
    pub options: Cow<'a, [T]>,
    /// Hovered Item Pointer
    /// Label Font
    pub font: Renderer::Font,
    /// Style for Font colors and Box hover colors.
    pub style: <Renderer::Theme as StyleSheet>::Style,
    /// Function Pointer On Select to call on Mouse button press.
    pub on_selected: Box<dyn Fn(usize) -> Message>,
    /// The padding Width
    pub padding: f32,
    /// The Text Size
    pub text_size: f32,
    /// Shadow Type holder for Renderer.
    pub phantomdata: PhantomData<Renderer>,
    /// selected index
    pub selected_index: Option<usize>,
    /// True if selected index is overwrited to last_selected_index
    pub selected_index_applied: bool,
}

/// The Private [`ListState`] Handles the State of the inner list.
#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// Statehood of hovered_option
    pub hovered_option: Option<usize>,
    /// The index in the list of options of the last chosen Item Clicked for Processing
    pub last_selected_index: Option<usize>,
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer> for List<'a, T, Message, Renderer>
where
    T: Clone + ToString,
    Renderer: iced_native::Renderer + iced_native::text::Renderer<Font = iced_native::Font>,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ListState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ListState::default())
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, _renderer: &Renderer, limits: &Limits) -> Node {
        use std::f32;
        let limits = limits.height(Length::Fill).width(Length::Fill);

        #[allow(clippy::cast_precision_loss)]
        let intrinsic = Size::new(
            limits.fill().width,
            (self.text_size + self.padding * 2.0) * self.options.len() as f32,
        );

        Node::new(intrinsic)
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let mut status = event::Status::Ignored;
        let list_state = state.state.downcast_mut::<ListState>();

        let mut changed_selected_index = false;

        if !self.selected_index_applied {
            list_state.last_selected_index = self.selected_index.clone();
            changed_selected_index = true;
            self.selected_index_applied = true;
        }

        if bounds.contains(cursor_position) {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    list_state.hovered_option = Some(
                        ((cursor_position.y - bounds.y) / (self.text_size + (self.padding * 2.0)))
                            as usize,
                    );
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    list_state.hovered_option = Some(
                        ((cursor_position.y - bounds.y) / (self.text_size + (self.padding * 2.0)))
                            as usize,
                    );

                    if let Some(index) = list_state.hovered_option {
                        list_state.last_selected_index = Some(index);
                    }

                    status =
                        list_state
                            .last_selected_index
                            .map_or(event::Status::Ignored, |last| {
                                if let Some(_option) = self.options.get(last) {
                                    shell.publish((self.on_selected)(last.clone()));
                                    event::Status::Captured
                                } else {
                                    event::Status::Ignored
                                }
                            });
                }
                _ => {
                    if changed_selected_index {
                        status =
                        list_state
                            .last_selected_index
                            .map_or(event::Status::Ignored, |last| {
                                if let Some(_option) = self.options.get(last) {
                                    shell.publish((self.on_selected)(last.clone()));
                                    event::Status::Captured
                                } else {
                                    event::Status::Ignored
                                }
                            });
                    }
                }
            }
        }

        status
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        _cursor_position: iced_graphics::Point,
        viewport: &iced_graphics::Rectangle,
    ) {
        use std::f32;
        let bounds = layout.bounds();
        let option_height = self.text_size + (self.padding * 2.0);
        let offset = viewport.y - bounds.y;
        let start = (offset / option_height) as usize;
        let end = ((offset + viewport.height) / option_height).ceil() as usize;

        let visible_options = &self.options[start..end.min(self.options.len())];
        let list_state = state.state.downcast_ref::<ListState>();

        let selected_index = if !self.selected_index_applied {
            self.selected_index.clone()
        } else {
            list_state.last_selected_index.clone()
        };

        for (i, option) in visible_options.iter().enumerate() {
            let i = start + i;
            let is_selected = selected_index == Some(i);
            let is_hovered = list_state.hovered_option == Some(i);

            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + option_height * i as f32,
                width: bounds.width,
                height: self.text_size + (self.padding * 2.0),
            };

            if is_selected || is_hovered {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: (0.0).into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    if is_selected {
                        theme.style(self.style).selected_background
                    } else {
                        theme.style(self.style).hovered_background
                    },
                );
            }

            let text_color = if is_selected {
                theme.style(self.style).selected_text_color
            } else if is_hovered {
                theme.style(self.style).hovered_text_color
            } else {
                theme.style(self.style).text_color
            };

            renderer.fill_text(iced_native::text::Text {
                content: &option.to_string(),
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.center_y(),
                    width: f32::INFINITY,
                    ..bounds
                },
                size: self.text_size,
                color: text_color,
                font: self.font,
                horizontal_alignment: Horizontal::Left,
                vertical_alignment: Vertical::Center,
            });
        }
    }
}

impl<'a, T, Message, Renderer> From<List<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: ToString + Clone,
    Message: 'a,
    Renderer: 'a + iced_native::Renderer + iced_native::text::Renderer<Font = iced_native::Font>,
    Renderer::Theme: StyleSheet,
{
    fn from(list: List<'a, T, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(list)
    }
}
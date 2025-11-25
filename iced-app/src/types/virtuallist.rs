use iced::{
    Length,
    widget::{Column, Scrollable, Space, column},
};

#[derive(Debug, Clone)]
pub enum VirtualListMessage {
    Scrolled(f32),
}

pub struct VirtualList<'a, T, Gen>
where
    Gen: Fn(usize, &'a T) -> iced::Element<'a, VirtualListMessage>,
{
    items: &'a [T],
    row_height: f32,
    scroll_y: f32,
    viewport_h: f32,
    generator: Gen,
}

impl<'a, T, Gen> VirtualList<'a, T, Gen>
where
    Gen: Fn(usize, &'a T) -> iced::Element<'a, VirtualListMessage>,
{
    pub fn new(
        items: &'a [T],
        row_height: f32,
        scroll_y: f32,
        viewport_h: f32,
        generator: Gen,
    ) -> Self {
        Self {
            items,
            row_height,
            scroll_y,
            viewport_h,
            generator,
        }
    }

    pub fn view(self) -> iced::Element<'a, VirtualListMessage> {
        let total = self.items.len();

        let first = (self.scroll_y / self.row_height).floor() as usize;
        let visible = (self.viewport_h / self.row_height).ceil() as usize + 2;
        let last = (first + visible).min(total);

        // Padding above the visible items
        let top_pad_h = first as f32 * self.row_height;
        let top_pad = Space::with_height(Length::Fixed(top_pad_h));

        // Padding below
        let bot_pad_h = (total - last) as f32 * self.row_height;
        let bot_pad = Space::with_height(Length::Fixed(bot_pad_h));

        let mut col: Column<'_, VirtualListMessage> = column![top_pad];

        for i in first..last {
            col = col.push((self.generator)(i, &self.items[i]));
        }

        col = col.push(bot_pad);

        Scrollable::new(col)
            .on_scroll(|vp| VirtualListMessage::Scrolled(vp.absolute_offset().y))
            .into()
    }
}

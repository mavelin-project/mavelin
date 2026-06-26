use std::time::Duration;

use crate::render::context::{RowStrategy, UiSubcontext};

pub mod loading_overlay;
pub mod main_screen;
pub mod pause_menu;
pub mod settings_screen;

pub trait Screen {
    type Message;

    fn update(&mut self, delta: Duration);
    fn draw(&self, context: &mut UiSubcontext<'_, RowStrategy, RowStrategy>, message: &mut Option<Self::Message>);
    fn render(&self, context: &mut UiSubcontext<'_, RowStrategy, RowStrategy>) -> Option<Self::Message> {
        let mut message = None;

        self.draw(context, &mut message);

        message
    }
}

// #[allow(dead_code)]
// pub struct ScreenManager {
//     screens: Vec<Box<dyn Screen>>,
// }

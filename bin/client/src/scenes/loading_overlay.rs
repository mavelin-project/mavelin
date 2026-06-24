use std::time::Duration;

use meralus_shared::{Color, Size2D, Vector2D};
use meralus_tween::{Animation, Tween};

use crate::{
    render::context::{Arrangement, RowStrategy, UiSubcontext},
    scenes::Screen,
};

pub struct LoadingOverlay {
    pub progress: Tween<f32>,
}

impl Screen for LoadingOverlay {
    fn update(&mut self, delta: Duration) {
        self.progress.advance(delta);
    }

    fn draw(&self, scope: &mut UiSubcontext<'_, RowStrategy, RowStrategy>) {
        scope.center(|scope| {
            scope.abs_pos(0.0, 24.0);
            scope.part_of_parent_width(1.0);

            scope.text("MERALUS", 72.0, "default", Color::from_hsl(110.0, 0.4, 0.7));
        });

        scope.column(|scope| {
            scope.abs_pos(0.0, 0.0);
            scope.fill_max_size();
            scope.set_arrangement(Arrangement::End);

            scope.row(|scope| {
                scope.part_of_parent_width(0.4);
                scope.set_height(32.0);
                scope.set_background_color(Color::BLACK);

                let size = scope.parent_size();

                scope.rect(Size2D::new(size.x * 0.4 * *self.progress.get(), 32.0), Color::RED);
            })
        });
    }
}

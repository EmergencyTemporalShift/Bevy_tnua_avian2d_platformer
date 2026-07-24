pub mod animating;
pub(crate) mod controls_other;
pub(crate) mod units;
pub mod particles;
pub mod game_states;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailedKeyState {
    JustPressed,
    Held,          // Pressed, but not just_pressed
    JustReleased,  // True only on the release frame
    Idle,          // Optional: Key is not being interacted with
}

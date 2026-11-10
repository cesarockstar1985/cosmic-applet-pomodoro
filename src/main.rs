//! Applet de panel COSMIC: temporizador Pomodoro.
//! Panel: muestra fase + tiempo restante. Click: popup con controles.

use std::time::{Duration, Instant};

use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Rectangle, Subscription};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{button, text};
use cosmic::{Application, Element};

const WORK_MIN: u64 = 25;
const SHORT_BREAK_MIN: u64 = 5;
const LONG_BREAK_MIN: u64 = 15;
const POMODOROS_PER_LONG_BREAK: u32 = 4;

const SOUND: &str = "/usr/share/sounds/freedesktop/stereo/complete.oga";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn duration(self) -> Duration {
        let min = match self {
            Phase::Work => WORK_MIN,
            Phase::ShortBreak => SHORT_BREAK_MIN,
            Phase::LongBreak => LONG_BREAK_MIN,
        };
        Duration::from_secs(min * 60)
    }

    fn label(self) -> &'static str {
        match self {
            Phase::Work => "Trabajo",
            Phase::ShortBreak => "Descanso corto",
            Phase::LongBreak => "Descanso largo",
        }
    }

    fn emoji(self) -> &'static str {
        match self {
            Phase::Work => "🍅",
            Phase::ShortBreak | Phase::LongBreak => "☕",
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    StartPause,
    Reset,
    Skip,
    SetPhase(Phase),
    ResetCount,
    PopupClosed(Id),
    Surface(cosmic::surface::Action<Message>),
}

struct Pomodoro {
    core: Core,
    popup: Option<Id>,
    phase: Phase,
    /// Tiempo restante cuando está en pausa.
    remaining: Duration,
    /// Momento de fin cuando está corriendo.
    deadline: Option<Instant>,
    completed: u32,
}

impl Pomodoro {
    fn running(&self) -> bool {
        self.deadline.is_some()
    }

    fn left(&self) -> Duration {
        match self.deadline {
            Some(d) => d.saturating_duration_since(Instant::now()),
            None => self.remaining,
        }
    }

    fn time_str(&self) -> String {
        // redondeo hacia arriba: muestra 25:00 al iniciar, 00:01 en el último segundo
        let secs = self.left().as_millis().div_ceil(1000) as u64;
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
        self.remaining = phase.duration();
        self.deadline = None;
    }

    fn next_phase(&self) -> Phase {
        match self.phase {
            Phase::Work if self.completed % POMODOROS_PER_LONG_BREAK == 0 => Phase::LongBreak,
            Phase::Work => Phase::ShortBreak,
            _ => Phase::Work,
        }
    }

    /// Fin de fase (natural o saltada): avanza a la siguiente, en pausa.
    fn advance(&mut self, natural: bool) {
        if self.phase == Phase::Work && natural {
            self.completed += 1;
        }
        let next = self.next_phase();
        if natural {
            let body = match next {
                Phase::Work => "Descanso terminado. ¡A trabajar!".to_string(),
                _ => format!(
                    "Pomodoro #{} completo. {} de {} min.",
                    self.completed,
                    next.label(),
                    next.duration().as_secs() / 60
                ),
            };
            notify(&body);
        }
        self.set_phase(next);
    }
}

fn notify(body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "Pomodoro", "-i", "alarm-symbolic", "-u", "critical", "Pomodoro", body])
        .spawn();
    let _ = std::process::Command::new("paplay").arg(SOUND).spawn();
}

impl Application for Pomodoro {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.cperalta.CosmicAppletPomodoro";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        (
            Self {
                core,
                popup: None,
                phase: Phase::Work,
                remaining: Phase::Work.duration(),
                deadline: None,
                completed: 0,
            },
            Task::none(),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.running() {
            cosmic::iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if self.running() && self.left().is_zero() {
                    self.advance(true);
                }
            }
            Message::StartPause => match self.deadline.take() {
                Some(d) => self.remaining = d.saturating_duration_since(Instant::now()),
                None => self.deadline = Some(Instant::now() + self.remaining),
            },
            Message::Reset => self.set_phase(self.phase),
            Message::Skip => self.advance(false),
            Message::SetPhase(p) => self.set_phase(p),
            Message::ResetCount => self.completed = 0,
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Surface(a) => return cosmic::task::message(cosmic::Action::Surface(a)),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let applet = &self.core.applet;
        let label = if self.running() {
            format!("{} {}", self.phase.emoji(), self.time_str())
        } else {
            format!("{} {} ⏸", self.phase.emoji(), self.time_str())
        };
        let (pad_major, pad_minor) = applet.suggested_padding(true);
        let suggested = applet.suggested_size(true);
        let (hpad, vpad) = if applet.is_horizontal() {
            (pad_major, pad_minor)
        } else {
            (pad_minor, pad_major)
        };

        let have_popup = self.popup;
        let btn = button::custom(
            cosmic::widget::container(
                applet
                    .text(label)
                    .wrapping(cosmic::iced::widget::text::Wrapping::None)
                    .width(Length::Shrink),
            )
            .width(Length::Shrink)
                .center_y(Length::Fixed(f32::from(suggested.1 + 2 * vpad))),
        )
        .padding([0, hpad])
        .class(cosmic::theme::Button::AppletIcon)
        .on_press_with_rectangle(move |offset, bounds| {
            if let Some(id) = have_popup {
                Message::Surface(destroy_popup(id))
            } else {
                Message::Surface(app_popup::<Pomodoro>(
                    |_| Default::default(),
                    move |state: &mut Pomodoro| {
                        let new_id = Id::unique();
                        state.popup = Some(new_id);
                        let mut s = state.core.applet.get_popup_settings(
                            state.core.main_window_id().unwrap(),
                            new_id,
                            None,
                            None,
                            None,
                        );
                        s.positioner.anchor_rect = Rectangle {
                            x: (bounds.x - offset.x) as i32,
                            y: (bounds.y - offset.y) as i32,
                            width: bounds.width as i32,
                            height: bounds.height as i32,
                        };
                        s
                    },
                    Some(Box::new(|state: &Pomodoro| {
                        Element::from(state.core.applet.popup_container(state.popup_view()))
                            .map(cosmic::Action::App)
                    })),
                ))
            }
        });

        self.core.applet.autosize_window(btn).into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        "".into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl Pomodoro {
    fn popup_view(&self) -> Element<'_, Message> {
        let phase_btn = |p: Phase, name: &'static str| {
            let b = button::text(name).on_press(Message::SetPhase(p));
            if self.phase == p {
                b.class(cosmic::theme::Button::Suggested)
            } else {
                b.class(cosmic::theme::Button::Standard)
            }
        };

        let start_label = if self.running() { "Pausar" } else { "Iniciar" };

        cosmic::widget::Column::new()
            .spacing(12)
            .padding(16)
            .align_x(Alignment::Center)
            .push(
                cosmic::widget::Row::new()
                    .spacing(6)
                    .push(phase_btn(Phase::Work, "Trabajo"))
                    .push(phase_btn(Phase::ShortBreak, "Corto"))
                    .push(phase_btn(Phase::LongBreak, "Largo")),
            )
            .push(text::body(self.phase.label()))
            .push(text::title1(self.time_str()))
            .push(
                cosmic::widget::Row::new()
                    .spacing(8)
                    .push(button::suggested(start_label).on_press(Message::StartPause))
                    .push(button::standard("Reiniciar").on_press(Message::Reset))
                    .push(button::standard("Saltar").on_press(Message::Skip)),
            )
            .push(
                cosmic::widget::Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(text::body(format!("Pomodoros: {}", self.completed)))
                    .push(button::link("poner a 0").on_press(Message::ResetCount)),
            )
            .into()
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<Pomodoro>(())
}

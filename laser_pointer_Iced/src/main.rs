use iced::{Point, Element, Subscription, Event, Rectangle, window, Color, Length,
    Renderer, Theme};
use iced::widget::canvas::{self, Frame, Geometry, Path};
use iced::mouse::Cursor;
use fastrand;

#[derive(Default, Clone, Copy)]
struct LaserPointer {
    x: f32,
    y: f32,
    speed: Speed,
    imaginary_target: Point,
}

#[derive(Clone, Copy, Default)]
enum Speed {
    #[default]
    Still,
    Slow,
    Fast,
    CrazyFast,
}

#[derive(Debug, Clone)]
enum Message {
    MovePointer(Point),
    UpdateFrame(Rectangle),
    TimerTick, // New message to trigger periodic updates
    Nothing,// This message ensures we handle properly other events so that ignore them in the update function
}
#[derive(Default, Clone)]
struct LaserPointerApp {
    pointer: LaserPointer,
    rectangle: Rectangle,
}
//Implementing From here isn’t essential, but it helps the following code be a bit cleaner.
impl From<LaserPointer> for Point {
        fn from(pointer: LaserPointer) -> Self {
            Point {
                x: pointer.x,
                y: pointer.y,
            }
        }
}
//We define this function because iced does not have an equivalent method
fn get_bottom_right(rect: Rectangle) -> Point {
    Point {
        x: rect.x + rect.width,
        y: rect.y + rect.height,
    }
}

impl LaserPointer {
    fn new() -> Self {
        Self {
        x: 50.0,
        y: 50.0,
        speed: Speed::default(),
        imaginary_target: Point { x: 50.0, y: 50.0 },
        }
    }

    //This method now handles the random laser pointer movement when the mouse arrow gets too close.
    fn random_movement(&mut self, amount: f32) {
        if fastrand::bool() {
            self.x += fastrand::f32() * amount;
            } else {
            self.x -= fastrand::f32() * amount;
            }
            if fastrand::bool() {
                self.y += fastrand::f32() * amount;
            } else {
                self.y -= fastrand::f32() * amount;
            }
    }
    //We don’t want the speed to change too frequently (cats get bored when a laser pointer moves too quickly), so
    //we’ll use a random f32 from 0.0 to 1.0 and only change when the number is greater than 0.98. In practice, this
    //will mean a speed change every few seconds. The following try_change_target() changes the invisible target
    //for the pointer in the same way.
    fn try_change_speed(&mut self) {
            use Speed::*;
            if fastrand::f32() > 0.98 {
                self.speed = match fastrand::u8(0..3) {
                    0 => Still,
                    1 => Slow,
                    2 => Fast,
                    _ => CrazyFast,
            }
        }
    }
    fn try_change_target(&mut self, rect: Rectangle) {
        let bottom_right = get_bottom_right(rect);
        if fastrand::f32() > 0.1 {
            self.imaginary_target = Point {
            x: fastrand::f32() * bottom_right.x,
            y: fastrand::f32() * bottom_right.y,
            };
        }
    }
    fn change_speed(&self) -> f32 {
        match self.speed {
            Speed::Still => 0.0,
            Speed::Slow => 0.05,
            Speed::Fast => 0.1,
            Speed::CrazyFast => 0.3,
        }
    }
    //Finally, we have this method to move the laser pointer once every loop. One of the speeds is 0.0,
    //though, so it will stay absolutely still in that case.
    fn move_self(&mut self) {
        let x_from_target = self.imaginary_target.x - self.x;
        let y_from_target = self.imaginary_target.y - self.y;
        self.x += fastrand::f32() * x_from_target * self.change_speed();
        self.y += fastrand::f32() * y_from_target * self.change_speed();
    }
}

impl LaserPointerApp{
    //This will be the default position of the laser pointer when starting the application
    fn new() -> Self {
        LaserPointerApp {
            pointer: LaserPointer::new(),
            rectangle: Rectangle {
                x: 10.0,
                y: 10.0,
                width: 800.0,
                height: 600.0,
            },
        }
    }
}
// The subscription function creates a Subscription listening to external events, which are mouse movements
// window resize events and other events
fn subscription() -> Subscription<Message> {
    Subscription::batch(vec![
    iced::event::listen().map(move |event| match event {
        Event::Mouse(iced::mouse::Event::CursorMoved{position})=>Message::MovePointer(position),
        Event::Window(window::Event::Resized (resize_info)) => {
            
            Message::UpdateFrame(Rectangle {
                x: 0.0,
                y: 0.0,
                width: resize_info.width as f32,
                height: resize_info.height as f32,
            })
        }
        _ => Message::Nothing ,
}),
    // Create a periodic timer to trigger updates every 100 ms
    iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::TimerTick),
    ])
}


// The canvas::Program trait allows us to implement the drawing logic for our application
impl canvas::Program<Message> for LaserPointerApp {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: Cursor) 
    -> Vec<Geometry> {
        let mut frame = Frame::new(renderer,bounds.size());

        // Draw the red laser pointer
        let path = Path::circle(Point::new(self.pointer.x, self.pointer.y), 20.0);
        frame.fill(&path, Color::from_rgb(1.0, 0.0, 0.0));

        vec![frame.into_geometry()]
    }
}


//The update function is responsible for applying the state changes based on the incoming messages.
//It is responsible for handling messages triggered by the events listened by the subscription function above.
fn update(laserpointer: &mut LaserPointerApp, message: Message) {
    let bounds=Rectangle {
        x: 0.0,
        y: 0.0,
        width: 800.0, // Default width
        height: 600.0, // Default height
    };
    // Always update the target and move the pointer regardless of the message
    laserpointer.rectangle = bounds;
    laserpointer.pointer.try_change_speed();
    laserpointer.pointer.try_change_target(laserpointer.rectangle);
    laserpointer.pointer.move_self();
    match message {
        Message::TimerTick => {
            // On each timer tick, update the target and move the pointer
            laserpointer.pointer.try_change_speed();
            laserpointer.pointer.try_change_target(laserpointer.rectangle);
            laserpointer.pointer.move_self();
        }
        Message::MovePointer(pos) => {
        if (laserpointer.pointer.x - pos.x).abs() < 20.0 && (laserpointer.pointer.y - pos.y).abs() < 20.0 {
            laserpointer.pointer.random_movement(50.0);
        }
    },
        Message::UpdateFrame(new_bounds) => {
            *laserpointer = LaserPointerApp::new();
            laserpointer.rectangle = new_bounds;
            laserpointer.pointer.try_change_speed();
            laserpointer.pointer.try_change_target(laserpointer.rectangle);
            laserpointer.pointer.move_self();
        },
        Message::Nothing => {}
}
}
//The view function is responsible for applying the drawing logic defined above to the screen. 
fn view(laserpointer: &LaserPointerApp) -> Element<Message> {
    iced::widget::Canvas::new(laserpointer)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn main() -> iced::Result {
    iced::application("Awesome laser pointer",update,view)
    .subscription(|_| subscription())
    .run()
}
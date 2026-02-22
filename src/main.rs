use std::{fs::DirEntry, path::Path};

use iced::{
    Length::Fill,
    Task, task,
    widget::{button, column, mouse_area, row, scrollable, text, text_input},
};

type Element<'a> = iced::Element<'a, Message>;

struct App {
    path: String,
    entries: Vec<DirEntry>,
}

#[derive(Clone)]
enum Message {
    GoTo(String),
    Open(String),
    GoBack,
    PathChanged(String),
}

// TODO: Handle file does not exist
// TODO: handle errors with a nice error toast/shitter

impl App {
    fn new(path: String) -> App {
        // TODO: Remove unwrap logic
        let entries = std::fs::read_dir(&path).unwrap().flatten().collect();
        Self { path, entries }
    }

    fn goto(&mut self, path: String) {
        self.entries = std::fs::read_dir(&path).unwrap().flatten().collect();
        self.path = path;
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GoTo(path) => self.goto(path),
            Message::Open(path) => {
                let x = open::that_detached(path);
                println!("{:?}", x);
            }
            Message::PathChanged(path) => self.path = path,
            Message::GoBack => {
                let p = Path::new(&self.path);
                if let Some(parent) = p.parent() {
                    let path = parent.to_string_lossy().into_owned();
                    self.goto(path);
                }
            }
        }

        task::Task::none()
    }

    fn view(&self) -> Element<'_> {
        let files = self.entries.iter().map(|e| -> Element<'_> {
            // TODO: I hate this
            let file_type = match e.file_type() {
                Ok(ft) => {
                    if ft.is_dir() {
                        "dir"
                    } else if ft.is_file() {
                        "file"
                    } else {
                        "uknown"
                    }
                }
                Err(_) => "unknown",
            };

            let name = e.file_name().to_string_lossy().into_owned();
            let path = e.path().to_string_lossy().into_owned();

            let label: Element = text(format!("{} {}", file_type, name)).into();

            if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                mouse_area(label)
                    .on_double_click(Message::GoTo(path))
                    .into()
            } else {
                mouse_area(label)
                    .on_double_click(Message::Open(path))
                    .into()
            }
        });

        column![
            row![
                button("Back").on_press(Message::GoBack),
                text_input("Path...", &self.path)
                    .on_input(Message::PathChanged)
                    .on_submit(Message::GoTo(self.path.clone())),
            ],
            scrollable(column(files).width(Fill).padding(20).spacing(10))
        ]
        .into()
    }
}

fn boot() -> App {
    let mut default_dir = "/".to_owned();
    if let Ok(dir) = std::env::var("HOME") {
        default_dir = dir
    }

    App::new(default_dir)
}

fn main() {
    iced::application(boot, App::update, App::view)
        .run()
        .unwrap();
}

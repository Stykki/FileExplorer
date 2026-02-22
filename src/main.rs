use std::{fs::DirEntry, path::Path};

use iced::{
    Length::Fill,
    Task, task,
    widget::{button, column, mouse_area, row, scrollable, text, text_input},
};

enum FileType {
    File,
    Dir,
    Unknown,
}

fn get_ft(entry: &DirEntry) -> FileType {
    let Ok(ft) = entry.file_type() else {
        return FileType::Unknown;
    };
    if ft.is_dir() {
        FileType::Dir
    } else if ft.is_file() {
        FileType::File
    } else {
        FileType::Unknown
    }
}

type Element<'a> = iced::Element<'a, Message>;

struct App {
    path: String,
    entries: Vec<DirEntry>,

    // TODO: index?
    selected_entry: Option<usize>,
}

#[derive(Clone)]
enum Message {
    GoTo(String),
    Open(String),
    Select(usize),
    GoBack,
    PathChanged(String),
}

// TODO: Handle file does not exist
// TODO: handle errors with a nice error toast/shitter

impl App {
    fn new(path: String) -> App {
        // TODO: Remove unwrap logic
        let entries = std::fs::read_dir(&path).unwrap().flatten().collect();
        Self {
            path,
            entries,
            selected_entry: None,
        }
    }

    fn goto(&mut self, path: String) {
        self.entries = std::fs::read_dir(&path).unwrap().flatten().collect();
        self.path = path;
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GoTo(path) => self.goto(path),
            Message::Open(path) => {
                //TODO: delete this dep?
                let _ = open::that_detached(path);
            }
            Message::PathChanged(path) => self.path = path,
            Message::GoBack => {
                let p = Path::new(&self.path);
                if let Some(parent) = p.parent() {
                    let path = parent.to_string_lossy().into_owned();
                    self.goto(path);
                }
            }
            Message::Select(index) => self.selected_entry = Some(index),
        }

        task::Task::none()
    }

    fn view(&self) -> Element<'_> {
        let files = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, e)| -> Element<'_> {
                let ft = get_ft(e);
                let ft_label = match ft {
                    FileType::Dir => "\u{ea83}",
                    FileType::File => "\u{f15b}",
                    FileType::Unknown => "\u{f128}",
                };
                let name = e.file_name().to_string_lossy().into_owned();
                let path = e.path().to_string_lossy().into_owned();

                let row = row![text(ft_label).width(24), text(name),];

                match ft {
                    FileType::Dir => mouse_area(row)
                        .on_press(Message::Select(index))
                        .on_double_click(Message::GoTo(path))
                        .into(),
                    _ => mouse_area(row).on_double_click(Message::Open(path)).into(),
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

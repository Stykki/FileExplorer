use std::{ffi::OsString, fs::DirEntry, path::Path};

use iced::{
    Event,
    Length::Fill,
    Subscription, Task, event,
    keyboard::Modifiers,
    task,
    widget::{button, column, container, mouse_area, row, scrollable, text, text_input},
};

enum FileType {
    File,
    Dir,
    Unknown,
}

// TODO: map when reading directory
struct File {
    file_type: FileType,
    name: OsString,
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
    modifiers: Modifiers,

    // TODO: Vec of selected entries
    selected_entries: Vec<usize>,
    // TODO: Make this fixed when doing selections
    anchor: Option<usize>,
}

#[derive(Clone)]
enum Message {
    GoTo(String),
    Open(String),
    Select(usize),
    GoBack,
    PathChanged(String),
    Event(Event),
}

// TODO: Handle file does not exist
// TODO: handle errors with a nice error toast/shitter

impl App {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    fn new(path: String) -> App {
        // TODO: Remove unwrap logic
        let entries = std::fs::read_dir(&path).unwrap().flatten().collect();
        Self {
            path,
            entries,
            selected_entries: Vec::with_capacity(255),
            anchor: None,
            modifiers: Modifiers::empty(),
        }
    }

    fn goto(&mut self, path: String) {
        let now = std::time::Instant::now();
        self.entries = std::fs::read_dir(&path).unwrap().flatten().collect();
        println!("read_dir took: {:?}", now.elapsed());
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
            Message::Select(index) => {
                if self.modifiers.shift() {
                    if let Some(anchor) = self.anchor {
                        if anchor > index + 1 {
                            self.selected_entries = (index..anchor + 1).collect()
                        } else {
                            self.selected_entries = (anchor..index + 1).collect()
                        }
                    }
                } else if self.modifiers.control() {
                    if self.selected_entries.contains(&index) {
                        self.selected_entries.retain(|idx| index != *idx);
                    } else {
                        self.selected_entries.push(index);
                    }
                } else {
                    self.anchor = Some(index);
                    self.selected_entries.clear();
                    self.selected_entries.push(index);
                }
            }

            Message::Event(event) => match event {
                Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                    self.modifiers = modifiers
                }
                _ => {}
            },
        }

        task::Task::none()
    }

    fn view(&self) -> Element<'_> {
        let now = std::time::Instant::now();
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

                let is_selected = self.selected_entries.contains(&index);

                let row = container(row![text(ft_label).width(24), text(name)])
                    .width(Fill)
                    .padding([2, 4])
                    .style(move |theme: &iced::Theme| {
                        if is_selected {
                            iced::widget::container::Style {
                                background: Some(
                                    theme.extended_palette().primary.weak.color.into(),
                                ),
                                ..Default::default()
                            }
                        } else {
                            iced::widget::container::Style::default()
                        }
                    })
                    .padding(10);

                match ft {
                    FileType::Dir => mouse_area(row)
                        .on_press(Message::Select(index))
                        .on_double_click(Message::GoTo(path))
                        .into(),
                    _ => mouse_area(row)
                        .on_press(Message::Select(index))
                        .on_double_click(Message::Open(path))
                        .into(),
                }
            });

        println!("Parsing files took: {:?}", now.elapsed());
        column![
            row![
                button("Back").on_press(Message::GoBack),
                text_input("Path...", &self.path)
                    .on_input(Message::PathChanged)
                    .on_submit(Message::GoTo(self.path.clone())),
            ],
            scrollable(column(files).width(Fill).padding(20))
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
        .subscription(App::subscription)
        .run()
        .unwrap();
}

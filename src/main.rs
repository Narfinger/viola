extern crate id3;
extern crate gtk;
extern crate walkdir;
use gtk::prelude::*;
use gtk::{Button, ListBox, Layout, Label, Grid, Orientation, ScrolledWindow, Window, WindowType};
use walkdir::WalkDir;

fn parse_folder(w: &Window) -> ScrolledWindow {
    let b = ListBox::new();

    let folder = "/mnt/ssd-media/Musik/1rest";
    for i in  WalkDir::new(folder) {
        if let Ok(f) = i {
            if f.file_type().is_file() {
                let st:String = format!("{}", f.path().display());
                let stt:&str = &st;
                let label = Label::new(Some(stt));
                b.add(&label);
            }
        }
    }
    let l = ScrolledWindow::new(None, None);
    l.add(&b);
    l
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("First GTK+ Program");
    window.set_default_size(350, 70);
   
    println!("Building list");
    let b = parse_folder(&window);
    println!("Done building list");

    let button = Button::new_with_label("Click me!");
    b.add(&button);
    window.add(&b);
    window.show_all();

    window.set_default_size(500,500);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    
/* 
    button.connect_clicked(|_| {
        println!("Clicked!");
    }); */
    
    gtk::main();
}
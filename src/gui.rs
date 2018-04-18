use std::rc::Rc;
use gtk;


/// TODO try to get all this as references and not as Rc with explicit lifetimes
pub struct Gui {
    notebook: Rc<gtk::Notebook>,
    title_label: Rc<gtk::Label>,
    artist_label: Rc<gtk::Label>,
    album_label: Rc<gtk::Label>,
    cover: Rc<gtk::Image>, 
}
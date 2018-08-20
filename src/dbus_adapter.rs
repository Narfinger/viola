use dbus::{BusType, Connection, NameFlag, Path};
use dbus::tree::{DataType, Factory, Interface, MTFn, MethodErr};
use dbus::arg;
use dbus::tree;
use std::thread;
use std;

use dbus_mpris_player::OrgMprisMediaPlayer2Player;
use gstreamer_wrapper::{GStreamerAction, GStreamerExt};
use types::*;

struct DBusAdapter {
    gstreamer: GStreamerPtr,
}

impl OrgMprisMediaPlayer2Player for DBusAdapter {
    type Err = MethodErr;
    fn next(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Next);
        Ok(())
    }

    fn previous(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Previous);
        Ok(())
    }

    fn pause(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Pausing);
        Ok(())
    }

    fn play_pause(&self) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Next);
        Ok(())
    }

    fn stop(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Next);
        Ok(())
    }

    fn play(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Playing);
        Ok(())
    }

    fn seek(&self, offset: i64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn set_position(&self, track_id: Path, position: i64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn open_uri(&self, uri: &str) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_playback_status(&self) -> Result<String, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(String::from("NOTHING"))
    }

    fn get_loop_status(&self) -> Result<String, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(String::from("NOTHING"))
    }

    fn set_loop_status(&self, value: String) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_rate(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn set_rate(&self, value: f64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_shuffle(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(false)
    }

    fn set_shuffle(&self, value: bool) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_metadata(&self) -> Result<std::collections::HashMap<String, arg::Variant<Box<arg::RefArg>>>, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        let hm = std::collections::HashMap::new();
        Ok(hm)
    }

    fn get_volume(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn set_volume(&self, value: f64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_position(&self) -> Result<i64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0)
    }

    fn get_minimum_rate(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn get_maximum_rate(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn get_can_go_next(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_go_previous(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_play(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_pause(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_seek(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

    fn get_can_control(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

}

pub fn setup(gui: &MainGuiPtr) -> () {
    //thread::spawn(move || {
    //    build(gui)
    //});
}




/* window.rs
 *
 * Copyright 2022 reticulis
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use gtk::prelude::*;
use adw::subclass::prelude::*;
use ashpd::desktop::account::UserInfo;
use ashpd::WindowIdentifier;
use gtk::{gio, glib, CompositeTemplate};
use crate::gio::glib::{clone, MainContext};
use crate::DATABASE;

mod imp {
    use gettextrs::gettext;
    use super::*;
    use anyhow::Result;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/reticulis/tors/window.ui")]
    pub struct TorsGtkWindow {
        #[template_child]
        pub welcome_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub avatar_profile: TemplateChild<adw::Avatar>,
        #[template_child]
        pub username_profile: TemplateChild<gtk::Label>,
        #[template_child]
        pub level_profile: TemplateChild<gtk::Label>,
        #[template_child]
        pub experience_profile: TemplateChild<gtk::Label>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TorsGtkWindow {
        const NAME: &'static str = "TorsGtkWindow";
        type Type = super::TorsGtkWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl TorsGtkWindow {
        async fn user_info() -> Result<UserInfo> {
            Ok(ashpd::desktop::account::user_information(
                &WindowIdentifier::default(),
                "App would like to access user information",
            ).await?)
        }

        fn build_avatar(&self, path: &str) -> Result<()> {
            let picture = gio::File::for_uri(path);
            let texture = gtk::gdk::Texture::from_file(&picture)?;

            self.avatar_profile.set_custom_image(Some(&texture));

            Ok(())
        }

        fn build_welcome_label(&self, username: &str) {
            let message = gettext("Hello, {0}!").replace("{0}", username);

            self.welcome_label.set_label(&message)
        }

        fn build_profile_stats(&self) -> Result<()> {
            let db = &mut *DATABASE.lock().unwrap();

            let account = &db.account;

            let level = gettext("Level: {0}").replace("{0}", &account.lvl.to_string());
            let experience = gettext("Experience: {0}").replace("{0}", &account.exp.to_string());

            self.level_profile.set_label(&level);
            self.experience_profile.set_label(&experience);

            Ok(())
        }
    }

    impl ObjectImpl for TorsGtkWindow {
        fn constructed(&self, obj: &Self::Type) {
            let main_context = MainContext::default();

            main_context.block_on(clone!(@strong self as ui => async move {
                let user_info = TorsGtkWindow::user_info().await?;

                ui.build_avatar(user_info.image())?;
                ui.build_welcome_label(user_info.name());
                ui.build_profile_stats()?;

                ui.username_profile.set_label(&user_info.name());

                anyhow::Ok(())
            })).unwrap();

            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for TorsGtkWindow {}
    impl WindowImpl for TorsGtkWindow {}
    impl ApplicationWindowImpl for TorsGtkWindow {}
    impl AdwApplicationWindowImpl for TorsGtkWindow {}

}

glib::wrapper! {
    pub struct TorsGtkWindow(ObjectSubclass<imp::TorsGtkWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl TorsGtkWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
            .expect("Failed to create TorsGtkWindow")
    }
}

use adw::{prelude::*, subclass::prelude::*};
use anyhow::Context;
use gtk::glib;
use process_data::Containerization;

use crate::config::PROFILE;
use crate::i18n::i18n;
use crate::utils::boot_time;
use crate::utils::process::ProcessItem;
use crate::utils::units::{convert_speed, convert_storage, format_time};

mod imp {

    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/net/nokyan/Resources/ui/dialogs/process_dialog.ui")]
    pub struct ResProcessDialog {
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub cpu_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub memory_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub drive_read_speed: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub drive_read_total: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub drive_write_speed: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub drive_write_total: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub gpu_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub vram_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub encoder_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub decoder_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub total_cpu_time: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub user_cpu_time: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub system_cpu_time: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub pid: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub running_since: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub commandline: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub user: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub cgroup: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub containerized: TemplateChild<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResProcessDialog {
        const NAME: &'static str = "ResProcessDialog";
        type Type = super::ResProcessDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResProcessDialog {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }
    }

    impl WidgetImpl for ResProcessDialog {}
    impl WindowImpl for ResProcessDialog {}
    impl AdwDialogImpl for ResProcessDialog {}
}

glib::wrapper! {
    pub struct ResProcessDialog(ObjectSubclass<imp::ResProcessDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl ResProcessDialog {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn init<S: AsRef<str>>(&self, process: &ProcessItem, user: S) {
        self.setup_widgets(process, user.as_ref());
    }

    pub fn setup_widgets(&self, process: &ProcessItem, user: &str) {
        let imp = self.imp();

        imp.name.set_label(&process.display_name);

        imp.user.set_subtitle(user);

        imp.pid.set_subtitle(&process.pid.to_string());

        imp.running_since.set_subtitle(
            &boot_time()
                .and_then(|boot_time| {
                    boot_time
                        .add_seconds(process.starttime)
                        .context("unable to add seconds to boot time")
                })
                .and_then(|time| time.format("%c").context("unable to format running_since"))
                .map(|gstr| gstr.to_string())
                .unwrap_or(i18n("N/A")),
        );

        imp.commandline.set_subtitle(&process.commandline);
        imp.commandline.set_tooltip_text(Some(&process.commandline));

        imp.cgroup
            .set_subtitle(&process.cgroup.clone().unwrap_or_else(|| i18n("N/A")));
        imp.cgroup
            .set_tooltip_text(Some(&process.cgroup.clone().unwrap_or_else(|| i18n("N/A"))));

        let containerized = match process.containerization {
            Containerization::None => i18n("No"),
            Containerization::Flatpak => i18n("Yes (Flatpak)"),
            Containerization::Snap => i18n("Yes (Snap)"),
        };
        imp.containerized.set_subtitle(&containerized);

        self.update(process);
    }

    pub fn update(&self, process: &ProcessItem) {
        let imp = self.imp();

        imp.cpu_usage
            .set_subtitle(&format!("{:.1} %", process.cpu_time_ratio * 100.0));

        imp.memory_usage
            .set_subtitle(&convert_storage(process.memory_usage as f64, false));

        if let Some(read_speed) = process.read_speed {
            imp.drive_read_speed
                .set_subtitle(&convert_speed(read_speed, false));
        } else {
            imp.drive_read_speed.set_subtitle(&i18n("N/A"));
        }

        if let Some(read_total) = process.read_total {
            imp.drive_read_total
                .set_subtitle(&convert_storage(read_total as f64, false));
        } else {
            imp.drive_read_total.set_subtitle(&i18n("N/A"));
        }

        if let Some(write_speed) = process.write_speed {
            imp.drive_write_speed
                .set_subtitle(&convert_speed(write_speed, false));
        } else {
            imp.drive_write_speed.set_subtitle(&i18n("N/A"));
        }

        if let Some(write_total) = process.write_total {
            imp.drive_write_total
                .set_subtitle(&convert_storage(write_total as f64, false));
        } else {
            imp.drive_write_total.set_subtitle(&i18n("N/A"));
        }

        imp.gpu_usage
            .set_subtitle(&format!("{:.1} %", process.gpu_usage * 100.0));

        imp.vram_usage
            .set_subtitle(&convert_storage(process.gpu_mem_usage as f64, false));

        imp.encoder_usage
            .set_subtitle(&format!("{:.1} %", process.enc_usage * 100.0));

        imp.decoder_usage
            .set_subtitle(&format!("{:.1} %", process.dec_usage * 100.0));

        imp.total_cpu_time.set_subtitle(&format_time(
            process.user_cpu_time + process.system_cpu_time,
        ));

        imp.user_cpu_time
            .set_subtitle(&format_time(process.user_cpu_time));

        imp.system_cpu_time
            .set_subtitle(&format_time(process.system_cpu_time));
    }
}

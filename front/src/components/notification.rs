use crate::icons::{CircleCheck, CircleX, Info, TriangleAlert, X};
use yew::prelude::*;

use crate::hooks::use_trans::use_trans;

#[derive(Clone, PartialEq, Debug)]
#[allow(dead_code)]
pub enum NotificationType {
    Success,
    Warning,
    Error,
    Info,
}

impl NotificationType {
    pub fn to_title(&self) -> &'static str {
        match self {
            NotificationType::Success => "成功",
            NotificationType::Warning => "警告",
            NotificationType::Error => "错误",
            NotificationType::Info => "信息",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct NotificationProps {
    pub notification_type: NotificationType,
    #[prop_or_default]
    pub title: Option<String>,
    pub message: String,
    pub show: bool,
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,
    #[prop_or_default]
    pub auto_close: Option<bool>,
    #[prop_or_default]
    pub duration: Option<u32>, // 自动关闭时间（毫秒）
}

#[function_component(Notification)]
pub fn notification(props: &NotificationProps) -> Html {
    let show = use_state(|| props.show);
    let t = use_trans();

    // 自动关闭逻辑
    {
        let show = show.clone();
        let auto_close = props.auto_close.unwrap_or(true);
        let duration = props.duration.unwrap_or(5000);
        let on_close = props.on_close.clone();

        use_effect_with(props.show, move |&is_show| {
            if is_show && auto_close {
                let show_clone = show.clone();
                let on_close_clone = on_close.clone();

                gloo::timers::callback::Timeout::new(duration, move || {
                    show_clone.set(false);
                    if let Some(callback) = on_close_clone {
                        callback.emit(());
                    }
                })
                .forget();
            }
            || ()
        });
    }

    let on_close_click = {
        let show = show.clone();
        let on_close = props.on_close.clone();

        Callback::from(move |_: MouseEvent| {
            show.set(false);
            if let Some(callback) = &on_close {
                callback.emit(());
            }
        })
    };

    if !*show || !props.show {
        return html! {};
    }

    let default_title = match props.notification_type {
        NotificationType::Success => t.t("notification.success"),
        NotificationType::Warning => t.t("notification.warning"),
        NotificationType::Error => t.t("notification.error"),
        NotificationType::Info => t.t("notification.info"),
    };
    let title = props.title.clone().unwrap_or(default_title);

    let (icon, color_class, border_class) = match props.notification_type {
        NotificationType::Success => (
            html! {<CircleCheck class="h-5 w-5" />},
            "text-green-400 bg-green-500/20",
            "border-green-500/40",
        ),
        NotificationType::Warning => (
            html! {<TriangleAlert class="h-5 w-5" />},
            "text-amber-400 bg-amber-500/20",
            "border-amber-500/40",
        ),
        NotificationType::Error => (
            html! {<CircleX class="h-5 w-5" />},
            "text-red-400 bg-red-500/20",
            "border-red-500/40",
        ),
        NotificationType::Info => (
            html! {<Info class="h-5 w-5" />},
            "text-blue-400 bg-blue-500/20",
            "border-blue-500/40",
        ),
    };

    html! {
        <div class={classes!("relative", "w-full", "rounded-lg", "border", "p-4", "mb-4", color_class, border_class)} role="alert">
            <div class="flex gap-4">
                <div class="mt-0.5">{icon}</div>
                <div class="flex-1">
                    <h5 class="mb-1 font-medium leading-none tracking-tight">{title}</h5>
                    <div class="text-sm opacity-90">{&props.message}</div>
                </div>
                if props.on_close.is_some() {
                    <button
                        class="absolute right-4 top-4 rounded-md opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                        onclick={on_close_click}
                    >
                        <X class="h-4 w-4" />
                        <span class="sr-only">{t.t("common.close")}</span>
                    </button>
                }
            </div>
        </div>
    }
}

/// 简化的通知组件，用于快速显示消息
#[derive(Properties, PartialEq)]
pub struct SimpleNotificationProps {
    pub message: String,
    pub notification_type: NotificationType,
}

#[function_component(SimpleNotification)]
pub fn simple_notification(props: &SimpleNotificationProps) -> Html {
    let t = use_trans();
    let title = match props.notification_type {
        NotificationType::Success => t.t("notification.success"),
        NotificationType::Warning => t.t("notification.warning"),
        NotificationType::Error => t.t("notification.error"),
        NotificationType::Info => t.t("notification.info"),
    };

    let (icon, color_class, border_class) = match props.notification_type {
        NotificationType::Success => (
            html! {<CircleCheck class="h-5 w-5" />},
            "text-green-400 bg-green-500/20",
            "border-green-500/40",
        ),
        NotificationType::Warning => (
            html! {<TriangleAlert class="h-5 w-5" />},
            "text-amber-400 bg-amber-500/20",
            "border-amber-500/40",
        ),
        NotificationType::Error => (
            html! {<CircleX class="h-5 w-5" />},
            "text-red-400 bg-red-500/20",
            "border-red-500/40",
        ),
        NotificationType::Info => (
            html! {<Info class="h-5 w-5" />},
            "text-blue-400 bg-blue-500/20",
            "border-blue-500/40",
        ),
    };

    html! {
        <div class={classes!("relative", "w-full", "rounded-lg", "border", "p-4", "mb-4", color_class, border_class)} role="alert">
            <div class="flex gap-4">
                <div class="mt-0.5">{icon}</div>
                <div class="flex-1">
                    <h5 class="mb-1 font-medium leading-none tracking-tight">{title}</h5>
                    <div class="text-sm opacity-90">{&props.message}</div>
                </div>
            </div>
        </div>
    }
}

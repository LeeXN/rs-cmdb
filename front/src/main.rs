mod app;
mod components;
mod hooks;
mod i18n;
mod icons;
mod pages;
mod routes;
mod services;
mod stores;
mod types;
mod utils;

use app::App;
use i18n::Language;
use utils::i18n_helper;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    // 初始化国际化，默认使用中文
    i18n_helper::init_i18n(Language::ZhCn);

    yew::Renderer::<App>::new().render();
}

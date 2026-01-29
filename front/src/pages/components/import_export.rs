use common::models::{Component, ComponentStatus, ComponentType};
use rust_xlsxwriter::Workbook;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, BlobPropertyBag, FileReader, HtmlAnchorElement, Url};
use std::io::Cursor;
use calamine::{Reader, Xlsx, DataType};
use crate::services::component::update_component;
use yew::Callback;
use std::collections::HashMap;
use std::rc::Rc;
use crate::i18n::I18n;

pub fn export_components(components: &[Component]) {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Headers
    let headers = [
        "ID",
        "Serial Number",
        "Model",
        "Type",
        "Vendor",
        "Status",
        "Location",
        "Purchase Date",
        "Warranty Expiration",
    ];

    for (col, header) in headers.iter().enumerate() {
        let _ = worksheet.write_string(0, col as u16, *header);
    }

    // Write Data
    for (row, component) in components.iter().enumerate() {
        let r = (row + 1) as u32;

        let status = format!("{:?}", component.status);
        let c_type = format!("{:?}", component.component_type);

        let _ = worksheet.write_string(r, 0, &component.id);
        let _ = worksheet.write_string(r, 1, &component.serial_number);
        let _ = worksheet.write_string(r, 2, &component.model);
        let _ = worksheet.write_string(r, 3, &c_type);
        let _ = worksheet.write_string(r, 4, component.vendor.clone().unwrap_or_default());
        let _ = worksheet.write_string(r, 5, &status);
        let _ = worksheet.write_string(r, 6, component.location.clone().unwrap_or_default());
        let _ = worksheet.write_string(
            r,
            7,
            component.purchase_date.clone().unwrap_or_default(),
        );
        let _ = worksheet.write_string(
            r,
            8,
            component.warranty_expiration.clone().unwrap_or_default(),
        );
    }

    // Save to buffer
    let buf = workbook.save_to_buffer().unwrap();

    // Create Blob and Download
    let uint8_array = js_sys::Uint8Array::from(&buf[..]);
    let array = js_sys::Array::new();
    array.push(&uint8_array.buffer());

    let blob_options = BlobPropertyBag::new();
    blob_options
        .set_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_options).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let a: HtmlAnchorElement = document.create_element("a").unwrap().unchecked_into();
    a.set_href(&url);
    a.set_download("components_export.xlsx");
    document.body().unwrap().append_child(&a).unwrap();
    a.click();
    document.body().unwrap().remove_child(&a).unwrap();
    Url::revoke_object_url(&url).unwrap();
}

pub fn handle_file_import(
    file: web_sys::File,
    is_importing: yew::UseStateHandle<bool>,
    import_progress: yew::UseStateHandle<i32>,
    fetch_data: Callback<()>,
    t: Rc<I18n>,
) {
    let reader = FileReader::new().unwrap();
    let reader_clone = reader.clone();

    let onload = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let result = reader_clone.result().unwrap();
        let array_buffer = result.dyn_into::<js_sys::ArrayBuffer>().unwrap();
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let vec = uint8_array.to_vec();

        is_importing.set(true);
        import_progress.set(0);

        let cursor = Cursor::new(vec);
        let mut excel: Xlsx<_> = calamine::open_workbook_from_rs(cursor).unwrap();

        if let Ok(range) = excel.worksheet_range("Sheet1") {
            let mut updates = Vec::new();
            let _total_rows = range.height().saturating_sub(1); // Subtract header

            // Iterate rows (skip header)
            for row in range.rows().skip(1) {
                let id = row.first().and_then(|c| c.as_string()).unwrap_or_default();
                if id.is_empty() {
                    continue;
                }

                // Helper to get string from cell
                let get_str = |idx| {
                    row.get(idx)
                        .and_then(|c: &calamine::Data| c.as_string())
                        .unwrap_or_default()
                };

                let serial_number = get_str(1);
                let model = get_str(2);
                let type_str = get_str(3);
                let vendor = get_str(4);
                let status_str = get_str(5);
                let location = get_str(6);
                let purchase_date = get_str(7);
                let warranty_expiration = get_str(8);

                let component_type = match type_str.as_str() {
                    "GPU" => ComponentType::GPU,
                    "CPU" => ComponentType::CPU,
                    "Memory" => ComponentType::Memory,
                    "Disk" => ComponentType::Disk,
                    "NetworkCard" => ComponentType::NetworkCard,
                    "Motherboard" => ComponentType::Motherboard,
                    "PowerSupply" => ComponentType::PowerSupply,
                    _ => ComponentType::Other,
                };

                let status = match status_str.as_str() {
                    "InStock" => ComponentStatus::InStock,
                    "InUse" => ComponentStatus::InUse,
                    "LentOut" => ComponentStatus::LentOut,
                    "Faulty" => ComponentStatus::Faulty,
                    "Decommissioned" => ComponentStatus::Decommissioned,
                    _ => ComponentStatus::Unknown,
                };

                let component = Component {
                    id: id.clone(),
                    serial_number,
                    model,
                    component_type,
                    vendor: if vendor.is_empty() {
                        None
                    } else {
                        Some(vendor)
                    },
                    status,
                    location: if location.is_empty() {
                        None
                    } else {
                        Some(location)
                    },
                    purchase_date: if purchase_date.is_empty() {
                        None
                    } else {
                        Some(purchase_date)
                    },
                    warranty_expiration: if warranty_expiration.is_empty() {
                        None
                    } else {
                        Some(warranty_expiration)
                    },
                    client_id: None, // Not updating associations via Excel for now
                    client_hostname: None,
                    missing_since: None,
                    created_at: "".to_string(), // Will be ignored on update
                    updated_at: "".to_string(),
                };

                updates.push(component);
            }

            // Apply updates
            let fetch_data = fetch_data.clone();
            let is_importing = is_importing.clone();
            let import_progress = import_progress.clone();
            let t = t.clone();

            spawn_local(async move {
                let total = updates.len();
                let mut success_count = 0;
                for (idx, component) in updates.iter().enumerate() {
                    if let Ok(_) = update_component(&component.id, component).await {
                        success_count += 1;
                    }
                    let progress = ((idx as f64 / total as f64) * 100.0) as i32;
                    import_progress.set(progress);
                }

                fetch_data.emit(());
                is_importing.set(false);
                import_progress.set(100);
                gloo::dialogs::alert(&t.t_with_args(
                    "components.batch_update_complete",
                    &HashMap::from([
                        ("success".to_string(), success_count.to_string()),
                        ("total".to_string(), total.to_string()),
                    ]),
                ));
            });
        }
    }) as Box<dyn FnMut(_)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    reader.read_as_array_buffer(&file).unwrap();
}
